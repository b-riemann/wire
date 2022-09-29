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
pub struct Segment {
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
    pub fn getitem(&self, n: usize) -> Segment {
        Segment {
            start: self.indices[n],
            end: self.indices[n+1]
        }
    }
}

#[pyclass]
pub struct FunPyre {
    file: File,
    pages: Segments
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

    pub fn fetch_page(&self, n: usize) -> Vec<u8> {
        self.rfrom_segment( &self.pages.getitem(n) )
    }
}

fn uvec_to_string(uvec: &Vec<u8>) -> PyResult<String> {
    //Ok( str::from_utf8_unchecked(&uvec) )
    match str::from_utf8(&uvec) {
        Ok(ostr) => Ok(ostr.to_string()),
        Err(_) => Err(PyTypeError::new_err("utf-8 problem"))
    }
}

impl FunPyre {
}


#[pymethods]
impl FunPyre {
    #[new]
    pub fn new(filename: String, n_pages: usize) -> Self {
        let file = File::open(filename).unwrap();
        let mut buf_reader = BufReader::new(&file);
        buf_reader.seek(SeekFrom::Start(0)).unwrap();
        let indices = asplit_buf(&mut buf_reader, &b"<page".to_vec(), true, n_pages);
        FunPyre {
            file,
            pages: Segments { indices }
        }
    }

    fn from_segment(self_: PyRef<'_, Self>, segment: &Segment) -> PyResult<String> {
        uvec_to_string( &self_.rfrom_segment(segment) )
    }
}

// these bytes should not be part of the input file (they aren't for enwik9),
// and preferably still be part of the ascii table (for easier utf8 output for conversion if required
const MACROBYTE : u8 = b'\x05'; // x05 is nice, as its the enquiry symbol in ascii
const ANTISPACE : u8 = b'\x15'; // x08 would be nice, as its backspace in ascii

pub fn tokenize_page(invec: &Vec<u8>) -> Vec<u8> {
    let re = Regex::new(r"^<page>\n    <title>(.*)</title>\n    <id>(\d*)</id>\n    [\s\S]*<revision>\n      <id>(\d*)</id>\n      <timestamp>20(\d\d)-(\d\d)-(\d\d)T(\d\d):(\d\d):(\d\d)Z</timestamp>\n      <contributor>\n        [\s\S]+\n      </contributor>\n([\s\S]+)$").unwrap();
    let caps = re.captures( &invec ).unwrap();
    let mut ot = vec![MACROBYTE, b'p', b'a', b'g', b'e', b' '];
    ot.extend_from_slice( &caps[1] );
    ot.push(b' ');
    ot.extend_from_slice( &caps[2] );
    ot.push(b' ');
    ot.extend_from_slice( &caps[4] );
    ot.push(b' ');
    ot.extend_from_slice( &caps[5] );
    ot.extend_from_slice( &caps[6] );
    ot.extend_from_slice( &caps[7] );
    ot.extend_from_slice( &caps[8] );
    ot.extend_from_slice( &caps[9] );
    ot.extend(b" \n ");
    //main text
    ot.extend_from_slice( &caps[10] );
    ot.push(ANTISPACE); //just for testing
    ot
}

#[pyfunction]
fn tokenize_page_string(seg: &Segment, f: &FunPyre) -> PyResult<String> {
    uvec_to_string( &tokenize_page( &f.rfrom_segment(seg) ) )
}

#[pymodule]
fn pyre(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<FunPyre>()?;
    m.add_wrapped(wrap_pyfunction!(tokenize_page_string)).unwrap();
    Ok(())
}
