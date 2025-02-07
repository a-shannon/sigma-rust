use ergo_lib::chain::ergo_state_context::ErgoStateContext as ErgoStateContextInner;
use pyo3::prelude::*;

use super::{
    header::{Header, PreHeader},
    parameters::Parameters,
};

#[pyclass(eq)]
#[derive(Clone, PartialEq, Eq)]
pub struct ErgoStateContext(ErgoStateContextInner);

#[pymethods]
impl ErgoStateContext {
    #[new]
    fn new(pre_header: PreHeader, headers: [Header; 10], parameters: Parameters) -> Self {
        Self(ErgoStateContextInner::new(
            pre_header.into(),
            headers.map(Into::into),
            parameters.into(),
        ))
    }
}
