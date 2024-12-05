use pyo3::prelude::*;

#[pymodule]
fn ergo_lib_python(m: &Bound<'_, PyModule>) -> PyResult<()> {
    Ok(())
}
