use derive_more::{From, Into};
use ergo_lib::{
    chain::ergo_box::box_builder::ErgoBoxCandidateBuilder,
    ergotree_ir::chain::ergo_box::{self, box_value::BoxValue},
};
use pyo3::{exceptions::PyValueError, prelude::*};

use crate::to_value_error;

use super::{address::Address, token::Token};
#[pyclass(eq)]
#[derive(Clone, PartialEq, Eq, From, Into, Debug)]
pub struct ErgoBoxCandidate(ergo_box::ErgoBoxCandidate);

#[pymethods]
impl ErgoBoxCandidate {
    #[new]
    #[pyo3(signature=(*, value, address, creation_height, tokens))]
    fn new(
        value: u64,
        address: Option<Address>,
        creation_height: u32,
        tokens: Vec<Token>,
    ) -> PyResult<Self> {
        let mut builder = ErgoBoxCandidateBuilder::new(
            BoxValue::new(value).map_err(to_value_error)?,
            address
                .ok_or_else(|| PyValueError::new_err("address argument not found"))?
                .0
                .script()
                .map_err(to_value_error)?,
            creation_height,
        );
        for token in tokens {
            builder.add_token(token.into());
        }
        builder.build().map(Self).map_err(to_value_error)
    }
    fn __repr__(&self) -> String {
        format!("{self:?}")
    }
}
