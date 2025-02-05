use derive_more::{From, Into};
use ergo_lib::ergotree_ir::{ergo_tree, serialization::SigmaSerializable};
use pyo3::prelude::*;

use crate::{errors::SigmaSerializationError, to_value_error};

use super::chain::constant::Constant;

#[pyclass(eq)]
#[derive(PartialEq, Eq, Clone, From, Into)]
pub struct ErgoTree(pub ergo_tree::ErgoTree);

#[pymethods]
impl ErgoTree {
    fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }
    fn constants(&self) -> PyResult<Vec<Constant>> {
        self.0
            .get_constants()
            .map_err(to_value_error)
            .map(|constants| constants.into_iter().map(Into::into).collect())
    }
    /// Set constant at index to constant. Returns an exception if ErgoTree was not parsed or constant tpe does not match
    /// Returns a new ErgoTree
    fn with_constant(&self, index: usize, constant: Constant) -> PyResult<Self> {
        self.0
            .clone()
            .with_constant(index, constant.into())
            .map(Self)
            .map_err(to_value_error)
    }
    #[staticmethod]
    fn from_bytes(bytes: &[u8]) -> PyResult<Self> {
        ergo_tree::ErgoTree::sigma_parse_bytes(bytes)
            .map(Self)
            .map_err(to_value_error)
    }
    fn __bytes__(&self) -> PyResult<Vec<u8>> {
        self.0
            .sigma_serialize_bytes()
            .map_err(SigmaSerializationError::from)
            .map_err(Into::into)
    }
}
