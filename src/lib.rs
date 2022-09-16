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

fn scan(segment: Vec<u8>) -> u8 {
    let mut n = 0;
    for be in segment {
        if be > 60 {
            n += 1
        }
    }
    n
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

    fn bscan(&mut self, iterations: usize) -> Result<u8, &str> {
        let buf_reader = BufReader::new(&mut self.file);
        let mut x;
        for (n, segmentread) in buf_reader.split(b' ').enumerate() {
            match segmentread {
                Ok(segment) => x = scan(segment),
                Err(_) => return Err("bla")
            }
            if n>iterations {
                return Ok(x)
            }
        }
        Ok(42)
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

    fn scan(mut self_: PyRefMut<'_, Self>, iterations: usize) -> PyResult<[u8;2]> {
        let x = self_.bscan(iterations).unwrap();

        // plan is to also return flags if is english sentence etc.
        let mut arr = [0;2];
        arr[1] = x;
        return Ok(arr)
    }

}

#[pymodule]
fn pyre(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<FunPyre>()?;
    Ok(())
}
