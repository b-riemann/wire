use pyo3::prelude::*;
use std::io::prelude::*;

use std::fs::File;
use std::os::unix::prelude::FileExt;
use std::io::BufReader;
use std::io::SeekFrom;
use std::str;

use regex::bytes::Regex;

#[pyclass]
#[derive(Debug, Clone)]
enum SegmentKind {
    Unknown,
    AsciiSentence,
    Page
}

#[pyclass]
struct Segment {
    #[pyo3(get)]
    start: u64,
    #[pyo3(get)]
    end: u64,
    #[pyo3(get)]
    kind: SegmentKind
}

#[pymethods]
impl Segment {
    fn __repr__(&self) -> String {
        format!("Segment: {}--{} ({:?})", self.start, self.end, self.kind)
    }
}

fn sentence_classifier(segment: &Vec<u8>) -> SegmentKind {
    //single-character based classification of segments.
    // i.e. checks if standard english sentence
    for be in segment {
        match be {
          0x41..=0x5a => continue, //uppercase
          0x61..=0x7a => continue, //lowercase
          b'.' => continue,
          b',' => continue,
          b' ' => continue,
          _ => return SegmentKind::Unknown
        }
    }
    return SegmentKind::AsciiSentence
}


fn page_classifier(segment: &Vec<u8>) -> SegmentKind {
    let re = Regex::new(r"^<page>\n    <title>.*</title>\n    <id>(\d*)</id>\n    [\s\S]*<revision>\n      <id>(\d*)</id>\n      <timestamp>20(\d\d)-(\d\d)-(\d\d)T(\d\d):(\d\d):(\d\d)Z</timestamp>\n      <contributor>\n        [\s\S]+\n      </contributor>\n[\s\S]+$").unwrap();
    
    if re.is_match(segment) {
        return SegmentKind::Page
    } else {
        return SegmentKind::Unknown
    }
}


#[pyclass]
struct FunPyre {
    file : File,
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

    fn ascan<F>(&self, split_at: &Vec<u8>, scanner: F, split_left: bool, iterations: usize) -> Result<Vec<Segment>, &str>
        where F: Fn(&Vec<u8>) -> SegmentKind
    {
        let mut buf_reader = BufReader::new(&self.file);
        buf_reader.seek(SeekFrom::Start(0)).unwrap();
        
        let mut idx = 0;
        let mut tmp = Vec::with_capacity(512);
        let mut out = Vec::with_capacity(iterations);

        let split = split_at.last().unwrap().clone();
        let splen = split_at.len();

        'iters: loop {
            let readlen = buf_reader.read_until(split, &mut tmp).unwrap();
            if readlen == 0 {
                return Ok(out)
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

            if split_left {
                tmp.truncate(trunclen);
            }

            let x = Segment {
                start: idx,
                end: buf_reader.stream_position().unwrap() - if split_left { splen as u64 } else { 0 },
                kind: scanner(&tmp)
            };
            idx = x.end;
            out.push(x);
            if out.len() == iterations {
                return Ok(out)
            }

            if split_left {
                tmp = split_at.clone();
            } else {
                tmp.clear(); // as read_until only appends
            }
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
        }
    }

    fn from_segment(self_: PyRef<'_, Self>, segment: &Segment) -> PyResult<String> {
        Ok(self_.rfrom_segment(segment).unwrap())
    }

    fn find_sentences(self_: PyRef<'_, Self>, iterations: usize) -> PyResult<Vec<Segment>> {
        Ok( self_.ascan(&b".".to_vec(), sentence_classifier, false, iterations).unwrap() )
    }

    fn find_pages(self_: PyRef<'_, Self>, iterations: usize) -> PyResult<Vec<Segment>> {
        Ok( self_.ascan(&b"<page".to_vec(), page_classifier, true, iterations).unwrap() )
    }

}

#[pymodule]
fn pyre(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<FunPyre>()?;
    Ok(())
}
