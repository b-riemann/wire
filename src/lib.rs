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

    fn astring(self_: PyRef<'_, Self>, a: u64) -> PyResult<String> {
        let mut buf = [0u8; 8];
        self_.file.read_at(&mut buf, a)?;
        let ostr = str::from_utf8(&buf)?;
        return Ok(ostr.to_string())
    }
}

#[pymodule]
fn pyre(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<FunPyre>()?;
    Ok(())
}
