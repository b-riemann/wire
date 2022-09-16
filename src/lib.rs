use pyo3::prelude::*;

use std::fs::File;
use std::os::unix::prelude::FileExt;
use std::str;

#[pyfunction]
fn astring(a: u64) -> PyResult<String> {
    let file = File::open("demo.txt")?;
    let mut buf = [0u8; 8];
    file.read_at(&mut buf, a)?;
    let ostr = str::from_utf8(&buf)?;
    return Ok(ostr.to_string())
}

#[pyclass]
struct FunPyre {
    file : File,
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
}

#[pymodule]
fn pyre(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(astring, m)?)?;
    Ok(())
}
