use pyo3::{
    types::{PyModule, PyModuleMethods},
    Bound, PyResult,
};
use secret_key::SecretKey;

mod secret_key;

// Register all classes & functions of this module. This does not create a submodule because of a python limitation that would prevent 'from ergo_lib import submodule'
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<SecretKey>()?;
    Ok(())
}
