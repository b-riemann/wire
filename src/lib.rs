use pyo3::prelude::*;

use std::fs::File;
use std::os::unix::prelude::FileExt;
use std::str;

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

    fn from_index(self_: PyRef<'_, Self>, start_index: u64, n_bytes: usize) -> PyResult<String> {
        let mut buf = vec![0; n_bytes];
        self_.file.read_exact_at(&mut buf, start_index)?;
        let ostr = str::from_utf8(&buf)?;
        return Ok(ostr.to_string())
    }
}

#[pymodule]
fn pyre(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<FunPyre>()?;
    Ok(())
}
