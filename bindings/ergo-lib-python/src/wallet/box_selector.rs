use crate::{
    chain::{ergo_box::ErgoBox, token::Token},
    to_value_error,
};
use derive_more::{From, Into};
use ergo_lib::{
    ergotree_ir::chain::ergo_box::{box_value::BoxValue, ErgoBox as InnerErgoBox},
    wallet::box_selector::{BoxSelection as InnerBoxSelection, BoxSelector, SimpleBoxSelector},
};
use pyo3::prelude::*;

#[pyclass(eq)]
#[derive(Clone, PartialEq, Eq, From, Into)]
pub struct BoxSelection(InnerBoxSelection<InnerErgoBox>);

#[pymethods]
impl BoxSelection {
    // TODO: add abillity to construct BoxSelection
    fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }
}

/// Select boxes whose value and tokens sum to target_balance and target_tokens.
/// This uses a simple strategy where boxes are sorted by token amounts and selected in descending order
#[pyfunction]
pub fn select_boxes_simple(
    inputs: Vec<ErgoBox>,
    target_balance: u64,
    target_tokens: Vec<Token>,
) -> PyResult<BoxSelection> {
    // TODO: use bytemuck to convert collections of newtypes into inner type with zero-cost
    let target_tokens: Vec<_> = target_tokens.into_iter().map(|t| t.0).collect();
    SimpleBoxSelector::new()
        .select(
            inputs.into_iter().map(|p| p.0).collect(),
            BoxValue::new(target_balance).map_err(to_value_error)?,
            &target_tokens,
        )
        .map(Into::into)
        .map_err(to_value_error)
}
