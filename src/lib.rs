use pyo3::prelude::*;
use std::io::prelude::*;

use std::fs::File;
use std::os::unix::prelude::FileExt;
use std::io::BufReader;
use std::io::SeekFrom;
use std::str;

use regex::bytes::Regex;

#[pyclass]
struct Segment {
    #[pyo3(get)]
    start: u64,
    #[pyo3(get)]
    end: u64,
}

#[pymethods]
impl Segment {
    fn __repr__(&self) -> String {
        format!("Segment: {}--{}", self.start, self.end)
    }
}

fn is_sentence(segment: &Vec<u8>) -> bool {
    //single-character based classification of segments.
    // i.e. checks if standard english sentence
    for be in segment {
        match be {
          0x41..=0x5a => continue, //uppercase
          0x61..=0x7a => continue, //lowercase
          b'.' => continue,
          b',' => continue,
          b' ' => continue,
          _ => return false
        }
    }
    return true
}

#[pyclass]
struct Segments {
   indices: Vec<u64>
}

#[pymethods]
impl Segments {
    fn getitem(&self, n: usize) -> Segment {
        Segment {
            start: self.indices[n],
            end: self.indices[n+1]
        }
    }
}

fn is_page(segment: &Vec<u8>) -> bool {
    let re = Regex::new(r"^<page>\n    <title>.*</title>\n    <id>(\d*)</id>\n    [\s\S]*<revision>\n      <id>(\d*)</id>\n      <timestamp>20(\d\d)-(\d\d)-(\d\d)T(\d\d):(\d\d):(\d\d)Z</timestamp>\n      <contributor>\n        [\s\S]+\n      </contributor>\n[\s\S]+$").unwrap();
    re.is_match(segment)
}


#[pyclass]
struct FunPyre {
    file: File,
    //pages: Segments
}

impl FunPyre {
    fn rfrom_index(&self, start_index: u64, n_bytes: usize) -> Result<String, &str> {
        let mut buf = vec![0; n_bytes];
        self.file.read_exact_at(&mut buf, start_index).unwrap();
        match str::from_utf8(&buf) {
            Ok(ostr) => Ok(ostr.to_string()),
            Err(_) => Err("utf-8 problem")
        }
    }

    fn rfrom_segment(&self, seg: &Segment) -> Result<String, &str> {
        let n_bytes = (seg.end - seg.start) as usize;
        self.rfrom_index(seg.start, n_bytes)
    }

    fn ascan(&self, split_at: &Vec<u8>, split_left: bool, iterations: usize) -> Result<Segments, &str>
    {
        let mut buf_reader = BufReader::new(&self.file);
        buf_reader.seek(SeekFrom::Start(0)).unwrap();
        
        let mut tmp = Vec::with_capacity(512);
        let mut out = Vec::with_capacity(iterations);
        out.push(0);

        let split = split_at.last().unwrap().clone();
        let splen = split_at.len();

        'iters: loop {
            let readlen = buf_reader.read_until(split, &mut tmp).unwrap();
            if readlen == 0 {
                return Ok(Segments{ indices: out })
            }
            if readlen < splen {
                continue
            }

            let trunclen = tmp.len() - splen;
            for m in 0..splen-1 {
                if tmp[trunclen+m] != split_at[m] {
                    continue 'iters
                }
            }

            // if split_left {
            //     tmp.truncate(trunclen); //tmp is not used at the moment
            // }

            out.push( buf_reader.stream_position().unwrap() - if split_left { splen as u64 } else { 0 } );
            if out.len() == iterations {
                return Ok(Segments{ indices: out })
            }

            //if split_left {
            //    tmp = split_at.clone();
            //} else {
                tmp.clear();
            //}
        }
    }
}

#[pymethods]
impl FunPyre {
    #[new]
    fn new(filename: String) -> Self {
        let file = File::open(filename).unwrap();
        FunPyre {
            file,
            //pages: Segments { indices: vec![] }
        }
    }

    fn from_segment(self_: PyRef<'_, Self>, segment: &Segment) -> PyResult<String> {
        Ok(self_.rfrom_segment(segment).unwrap())
    }

    fn find_sentences(self_: PyRef<'_, Self>, iterations: usize) -> PyResult<Segments> {
        Ok( self_.ascan(&b".".to_vec(), false, iterations).unwrap() )
    }

    fn find_pages(self_: PyRef<'_, Self>, iterations: usize) -> PyResult<Segments> {
        Ok( self_.ascan(&b"<page".to_vec(), true, iterations).unwrap() )
    }

}

#[pymodule]
fn pyre(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<FunPyre>()?;
    Ok(())
}
