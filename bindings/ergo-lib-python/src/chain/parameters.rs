use derive_more::{From, Into};
use ergo_lib::chain::parameters::Parameters as ParametersInner;
use pyo3::prelude::*;

use crate::errors::JsonError;

#[pyclass(eq)]
#[derive(Clone, PartialEq, Eq, From, Into)]
pub struct Parameters(ParametersInner);

#[pymethods]
impl Parameters {
    #[staticmethod]
    fn default() -> Self {
        Self(Default::default())
    }
    #[staticmethod]
    fn from_json(json: &str) -> PyResult<Self> {
        serde_json::from_str(json)
            .map(Self)
            .map_err(JsonError::from)
            .map_err(Into::into)
    }
    fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }
}
