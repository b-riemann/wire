use pyo3::prelude::*;
use std::io::prelude::*;

use std::fs::File;
use std::os::unix::prelude::FileExt;
use std::io::BufReader;
use std::io::SeekFrom;
use std::str;

#[pyclass]
#[derive(Debug, Clone)]
enum SegmentKind {
    GeneralSegment,
    AsciiSentence,
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

fn scan(segment: &Vec<u8>) -> SegmentKind {
    //single-character based classification of segments.
    // i.e. checks if standard english sentence
    for be in segment {
        match be {
          0x41..=0x5a => continue, //uppercase
          0x61..=0x7a => continue, //lowercase
          b'.' => continue,
          b',' => continue,
          b' ' => continue,
          _ => return SegmentKind::GeneralSegment
        }
    }
    return SegmentKind::AsciiSentence
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

    fn ascan(&self, split_at: &Vec<u8>, iterations: usize) -> Result<Vec<Segment>, &str> {
        let mut buf_reader = BufReader::new(&self.file);
        buf_reader.seek(SeekFrom::Start(0)).unwrap();
        
        let mut idx = 0;
        let mut tmp = Vec::with_capacity(256);
        let mut out = Vec::with_capacity(iterations);

        let split = split_at.last().unwrap().clone();
        let splen = split_at.len();

        'iters: for _ in 0..iterations {
            buf_reader.read_until(split, &mut tmp).unwrap();

            let tmplen = tmp.len();
            if tmplen < splen {
                continue;
            }
            for m in 0..splen {
                if tmp[tmplen-splen+m] != split_at[m] {
                    continue 'iters;
                }
            }

            let x = Segment {
                start: idx,
                end: buf_reader.stream_position().unwrap(),
                kind: scan(&tmp)
            };
            tmp.clear(); // as read_until only appends
            idx = x.end;
            out.push(x);
        }
        Ok(out)
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

    fn scanner(self_: PyRef<'_, Self>, iterations: usize) -> PyResult<Vec<Segment>> {
        Ok( self_.ascan(&b".".to_vec(), iterations).unwrap() )
    }
}

#[pymodule]
fn pyre(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<FunPyre>()?;
    Ok(())
}
