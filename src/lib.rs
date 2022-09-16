use pyo3::prelude::*;
use std::io::prelude::*;

use std::fs::File;
use std::os::unix::prelude::FileExt;
use std::io::BufReader;
use std::str;

#[pyclass]
struct FunPyre {
    file : File,
}

fn scan(segment: Vec<u8>) -> bool {
    //checks if standard english sentence
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

    fn bscan(&self, split_at: u8, iterations: usize) -> Result<Vec<bool>, &str> {
        let buf_reader = BufReader::new(&self.file);
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

}

#[pymodule]
fn pyre(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<FunPyre>()?;
    Ok(())
}
