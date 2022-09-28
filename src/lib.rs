use pyo3::prelude::*;
use pyo3::exceptions::PyTypeError;
use std::io::prelude::*;

use std::fs::File;
use std::os::unix::prelude::FileExt;
use std::io::BufReader;
use std::io::SeekFrom;
use std::str;

use regex::bytes::Regex;

#[pyclass]
#[derive(Clone)]
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

//fn is_sentence(segment: &Vec<u8>) -> bool {
//    //single-character based classification of segments.
//    // i.e. checks if standard english sentence
//    for be in segment {
//        match be {
//          0x41..=0x5a => continue, //uppercase
//          0x61..=0x7a => continue, //lowercase
//          b'.' => continue,
//          b',' => continue,
//          b' ' => continue,
//          _ => return false
//        }
//    }
//    return true
//}

fn asplit_buf<T>(buf: &mut T, split_at: &Vec<u8>, split_left: bool, mut iterations: usize) -> Vec<u64>
    where T: BufRead + Seek {
    let mut out = Vec::with_capacity(iterations+1);
    out.push( buf.stream_position().unwrap() );

    let mut tmp = Vec::with_capacity(512);
    let split = split_at.last().unwrap().clone();
    let splen = split_at.len();

    'iters: loop {
        let readlen = buf.read_until(split, &mut tmp).unwrap();
        if readlen == 0 {
            return out
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
        out.push( buf.stream_position().unwrap() - if split_left { splen as u64 } else { 0 } );
        iterations -= 1;
        if iterations == 0 {
            return out
        }

        //if split_left {
        //    tmp = split_at.clone();
        tmp.clear();
    }
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

#[pyclass]
struct FunPyre {
    file: File,
}



impl FunPyre {
    fn rfrom_index(&self, start_index: u64, n_bytes: usize) -> Vec<u8> {
        let mut buf = vec![0; n_bytes];
        self.file.read_exact_at(&mut buf, start_index).unwrap();
        buf
    }

    fn rfrom_segment(&self, seg: &Segment) -> Vec<u8> {
        let n_bytes = (seg.end - seg.start) as usize;
        self.rfrom_index(seg.start, n_bytes)
    }

    fn ascan(&self, split_at: &Vec<u8>, split_left: bool, iterations: usize, start: u64) -> Result<Segments, &str>
    {
        let mut buf_reader = BufReader::new(&self.file);
        buf_reader.seek(SeekFrom::Start(start)).unwrap();
        let indices = asplit_buf(&mut buf_reader, split_at, split_left, iterations);
        Ok( Segments{ indices } )
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
        let buf = self_.rfrom_segment(segment);
        match str::from_utf8(&buf) {
            Ok(ostr) => Ok(ostr.to_string()),
            Err(_) => Err(PyTypeError::new_err("utf-8 problem"))
        }
    }

    fn find_sentences(self_: PyRef<'_, Self>, iterations: usize) -> PyResult<Segments> {
        Ok( self_.ascan(&b".".to_vec(), false, iterations, 0).unwrap() )
    }

    fn find_pages(self_: PyRef<'_, Self>, iterations: usize) -> PyResult<Segments> {
        Ok( self_.ascan(&b"<page".to_vec(), true, iterations, 0).unwrap() )
    }

}

enum Token {
    AsciiWord{ wrd: Vec<u8> },
    PageMacro {
        id: u8,
        rev_id: u32,
        tstamp_2k: u32,
    }
} 


#[pyclass]
struct Page {
    #[pyo3(get)]
    head: Segment,
    #[pyo3(get)]
    content: Segment
}

const PAGEBYTE : u8 = b'\x00';


#[pymethods]
impl Page {
    #[new]
    fn new(seg: &Segment, f: &FunPyre) -> Page {
        let head = f.ascan(&b"</contributor>\n".to_vec(), false, 1, seg.start).unwrap().getitem(0);
        let content = Segment{ start: head.end, end: seg.end };
        Page { head, content }
    }

    fn tokenize_head(&self, f: &FunPyre) -> Vec<u8> { // ToDo: Buffer instead of Vec
        let re = Regex::new(r"^<page>\n    <title>.*</title>\n    <id>(\d*)</id>\n    [\s\S]*<revision>\n      <id>(\d*)</id>\n      <timestamp>20(\d\d)-(\d\d)-(\d\d)T(\d\d):(\d\d):(\d\d)Z</timestamp>\n      <contributor>\n        [\s\S]+\n      </contributor>\n[\s\S]+$").unwrap();

        let input = f.rfrom_segment(&self.head);
        //let caps = re.captures(  ).unwrap();

        let mut out = Vec::with_capacity(256);
        //let id = caps[2];
        //let rev_id = caps[3];
        //let tstamp_2k : u32 = caps[9] +60*(caps[8] + 60*(caps[7] + 24*(caps[6] + 31*(caps[5] + 12*caps[4]))));

        out.push(PAGEBYTE);
        out
    }
}


#[pymodule]
fn pyre(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<FunPyre>()?;
    m.add_class::<Page>()?;
    Ok(())
}
