pub mod address;

use address::{Address, NetworkPrefix};
use pyo3::prelude::*;
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<NetworkPrefix>()?;
    m.add_class::<Address>()?;
    Ok(())
}
