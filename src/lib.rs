use pyo3::prelude::*;
use std::io::prelude::*;

use std::fs::File;
use std::os::unix::prelude::FileExt;
use std::io::BufReader;
use std::io::SeekFrom;
use std::str;

#[pyclass]
struct FunPyre {
    file : File,
}

fn scan(segment: Vec<u8>) -> bool {
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
    true
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

    fn ascan(&self, split_at: u8, iterations: usize) -> Result<Vec<u64>, &str> {
        let mut buf_reader = BufReader::new(&self.file);
        buf_reader.seek(SeekFrom::Start(0)).unwrap();
        
        let mut x = Vec::with_capacity(iterations);
        x.push(0);
        let mut tmp = Vec::with_capacity(256);

	for n in 1..iterations {
            buf_reader.read_until(split_at, &mut tmp).unwrap();
            // possibly classsify tmp string here..
            tmp.clear(); // as read_until only appends
            if n>=iterations {
                break;
            }
            x.push( buf_reader.stream_position().unwrap() )
        }
        Ok(x)
    }

    fn bscan(&self, split_at: u8, iterations: usize) -> Result<Vec<bool>, &str> {
        let mut buf_reader = BufReader::new(&self.file);
        buf_reader.seek(SeekFrom::Start(0)).unwrap();
        let mut x = Vec::with_capacity(iterations);
        for (n, segmentread) in buf_reader.split(split_at).enumerate() {
	    if n>=iterations {
                break;
            }
            match segmentread {
                Ok(segment) => x.push( scan(segment) ),
                Err(_) => return Err("bla")
            }
        }
        Ok(x)
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

    fn from_index(self_: PyRef<'_, Self>, start_index: u64, n_bytes: usize) -> PyResult<String> {
        Ok(self_.rfrom_index(start_index, n_bytes).unwrap())
    }

    fn scan(self_: PyRef<'_, Self>, iterations: usize) -> PyResult<Vec<bool>> {
        let x = self_.bscan(b'.', iterations).unwrap();
        Ok(x)
    }

    fn sentencer(self_: PyRef<'_, Self>, iterations: usize) -> PyResult<Vec<u64>> {
        Ok( self_.ascan(b'.', iterations).unwrap() )
    }
}

#[pymodule]
fn pyre(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<FunPyre>()?;
    Ok(())
}
