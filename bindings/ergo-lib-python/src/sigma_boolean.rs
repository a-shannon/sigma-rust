use derive_more::{From, Into};
use ergo_lib::ergotree_ir::sigma_protocol::sigma_boolean::ProveDlog as ProveDlogInner;
use pyo3::prelude::*;

use crate::chain::ec_point::EcPoint;

#[pyclass(eq, frozen)]
#[derive(PartialEq, Eq, From, Into, Clone)]
struct ProveDlog(ProveDlogInner);

#[pymethods]
impl ProveDlog {
    #[new]
    fn new(ec_point: EcPoint) -> Self {
        ProveDlogInner::new(ec_point.into()).into()
    }
    fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }
}

pub(crate) fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<ProveDlog>()?;
    Ok(())
}
