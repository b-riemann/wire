use pyo3::prelude::*;
use pyo3::exceptions::PyTypeError;
use std::io::prelude::*;

use std::fs::File;
use std::os::unix::prelude::FileExt;
use std::io::BufReader;
use std::io::SeekFrom;
use std::str;

use regex::bytes::Regex;
use regex::bytes::Captures;

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


struct Segments {
   indices: Vec<u64>
}

impl Segments {
    pub fn getitem(&self, n: usize) -> Segment {
        Segment {
            start: self.indices[n],
            end: self.indices[n+1]
        }
    }
}

// for enwik9, available escape bytes are:
//fb fd c0 0f f2 1d 0e 1c 7f 1b 08 1f fc 04 1e df 00 05 0c 19 f9 f8 18 10 0b 16 1a fa f3 \r f6 f7 12 f4 17 14 01 15 f1 f5 11 dd 07 13 c1 06 02 03
// of which the ascii control characters (in range 00-1f) are 
// 00 01 02 03 04 05 06 07 08 \r(0a) 0b 0c 0e 0f 10 11 12 13 14 15 16 17 18 19 1a 1b 1c 1d 1e 1f

// these escape bytes should not be part of the input file
// and preferably still be part of the ascii table (for easier utf8 output for conversion if required
const MACROBYTE : u8 = b'\x05'; // x05 is nice, as its the enquiry symbol in ascii
const ANTISPACE : u8 = b'\x15';
// idea: glue links and macros together with gluespace as a first step (macrobyte+macrotype single+glued-arg), late analyse in detail when text-parsing is implemented.
//const GLUESPACE : u8 = b'\x16';

//TODO: make upcase part of the parsing process and not of the regex procedure
//const UPCASE : u8 = b'\x07';

pub struct PageRegexer {
    re: Regex,
    beg_wrd: Regex,
    end_wrd: Regex,
    openduo: Regex,
    clseduo: Regex,
}

//fn glue(inv: &[u8]) -> Vec<u8> {
//    let mut x : Vec<u8> = inv.to_vec();
//    for n in 0..x.len() {
//        if x[n] == b' ' {
//            x[n] = GLUESPACE;
//        }
//    }
//    x
//}

impl PageRegexer {
    fn new() -> Self {
        PageRegexer {
            re: Regex::new(r#"^<page>\n    <title>(.*)</title>\n    <id>(\d*)</id>\n    ([\s\S]*)<revision>\n      <id>(\d*)</id>\n      <timestamp>20(\d\d)-(\d\d)-(\d\d)T(\d\d):(\d\d):(\d\d)Z</timestamp>\n      <contributor>\n        ([\s\S]+)\n      </contributor>\n      ([\s\S]*)<text xml:space="preserve"(.*)>([\s\S]+)$"#).unwrap(),
            beg_wrd: Regex::new(r"(\n|;|\||\(|-)([A-Za-z])").unwrap(), //note: weird to include - on one side but could help
            end_wrd: Regex::new(r"([A-Za-z])(,|\.|&|\||\))").unwrap(),
            openduo: Regex::new(r"(\[\[|\{\{|'')(\w)").unwrap(),
            clseduo: Regex::new(r"(\w)(\]\]|\}\}|'')").unwrap(),
        }
    }

    pub fn matches(&self, invec: &Vec<u8>) -> bool {
        self.re.is_match(&invec)
    }

    fn rex_content(&self, inslice: &[u8]) -> Vec<u8> {
        let a = self.beg_wrd.replace_all(inslice, |caps: &Captures| { 
            [caps[1][0], ANTISPACE, b' ', caps[2][0]]
            }).into_owned();
        let b = self.end_wrd.replace_all(&a, |caps: &Captures| { 
            [caps[1][0], b' ', ANTISPACE, caps[2][0]]
            }).into_owned();
        let c = self.openduo.replace_all(&b, |caps: &Captures| { 
            [caps[1][0], caps[1][1], ANTISPACE, b' ', caps[2][0]]
            }).into_owned();
        self.clseduo.replace_all(&c, |caps: &Captures| { 
            [caps[1][0], b' ', ANTISPACE, caps[2][0], caps[2][1]]
            }).into_owned()
    }

    pub fn rex(&self, invec: &Vec<u8>) -> Vec<u8> {
        let caps = self.re.captures( &invec ).unwrap();
        let mut ot = Vec::with_capacity(1024);
        ot.push(MACROBYTE); ot.extend(b"page ");
        ot.extend_from_slice( &caps[2] ); ot.push(b' ');
        ot.extend_from_slice( &caps[4] ); ot.push(b' ');
        ot.extend_from_slice( &caps[5] ); ot.extend_from_slice( &caps[6] ); ot.extend_from_slice( &caps[7] );
        ot.extend_from_slice( &caps[8] ); ot.extend_from_slice( &caps[9] ); ot.extend_from_slice( &caps[10] );
        ot.extend(b" revision:"); ot.extend_from_slice( &caps[3] );
        ot.extend(b": contributor:"); ot.extend_from_slice( &caps[11] );
        ot.extend(b": comment:"); ot.extend_from_slice( &caps[12] );
        ot.extend(b": texml:"); ot.extend_from_slice( &caps[13] );
        ot.extend(b": title:"); ot.extend_from_slice( &caps[1] );
        ot.extend(&[b' ', MACROBYTE, b' ']);
 
        ot.append( &mut self.rex_content( &caps[14] ));
        ot
    }

    //pub fn unrex(&self, ev: &Vec<u8>)
}

#[pyclass]
pub struct FunPyre {
    file: File,
    pages: Segments,
    pub pagere: PageRegexer
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
            pages: Segments { indices },
            pagere: PageRegexer::new()
        }
    }

    fn from_segment(self_: PyRef<'_, Self>, segment: &Segment) -> PyResult<String> {
        uvec_to_string( &self_.rfrom_segment(segment) )
    }
}

#[pymodule]
fn pyre(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<FunPyre>()?;
    Ok(())
}
