use std::collections::HashMap;

use derive_more::{From, Into};
use ergo_lib::{
    chain::ergo_box::box_builder::ErgoBoxCandidateBuilder,
    ergotree_ir::chain::ergo_box::{self, box_value::BoxValue},
};
use pyo3::{exceptions::PyValueError, prelude::*, types::PyDict};
use serde_pyobject::from_pyobject;

use crate::{ergo_tree::ErgoTree, errors::JsonError, to_value_error};

use super::{address::Address, constant::Constant, token::Token};

#[pyclass(eq, frozen, hash, ord)]
#[derive(PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Copy)]
#[repr(u8)]
pub enum NonMandatoryRegisterId {
    R4 = 4,
    R5 = 5,
    R6 = 6,
    R7 = 7,
    R8 = 8,
    R9 = 9,
}

impl From<ergo_box::NonMandatoryRegisterId> for NonMandatoryRegisterId {
    fn from(value: ergo_box::NonMandatoryRegisterId) -> Self {
        match value {
            ergo_box::NonMandatoryRegisterId::R4 => Self::R4,
            ergo_box::NonMandatoryRegisterId::R5 => Self::R5,
            ergo_box::NonMandatoryRegisterId::R6 => Self::R6,
            ergo_box::NonMandatoryRegisterId::R7 => Self::R7,
            ergo_box::NonMandatoryRegisterId::R8 => Self::R8,
            ergo_box::NonMandatoryRegisterId::R9 => Self::R9,
        }
    }
}

impl From<NonMandatoryRegisterId> for ergo_box::NonMandatoryRegisterId {
    fn from(id: NonMandatoryRegisterId) -> ergo_box::NonMandatoryRegisterId {
        #[allow(clippy::unwrap_used)]
        ergo_box::NonMandatoryRegisterId::try_from(id as i8).unwrap()
    }
}

/// Identifier of an :class:`ErgoBox`
#[pyclass(str = "{0}", eq)]
#[derive(PartialEq, Eq, Clone, Copy, From, Into)]
pub struct BoxId(ergo_box::BoxId);

#[pyclass(eq)]
#[derive(Clone, PartialEq, Eq, From, Into, Debug)]
pub struct ErgoBoxCandidate(ergo_box::ErgoBoxCandidate);

#[pymethods]
impl ErgoBoxCandidate {
    #[new]
    #[pyo3(signature=(*, value, address=None, ergo_tree=None, creation_height, tokens=None, registers=None))]
    fn new(
        value: u64,
        address: Option<Address>,
        ergo_tree: Option<ErgoTree>,
        creation_height: u32,
        tokens: Option<Vec<Token>>,
        registers: Option<HashMap<NonMandatoryRegisterId, Constant>>,
    ) -> PyResult<Self> {
        // TODO: maybe take only one argument (Address | ErgoTree)
        let tree = address
            .map(|addr| addr.0.script())
            .transpose()
            .map_err(to_value_error)?
            .xor(ergo_tree.map(|tree| tree.into()))
            .ok_or_else(|| {
                PyValueError::new_err("Expected only one of address or ergo_tree arguments")
            })?;
        let mut builder = ErgoBoxCandidateBuilder::new(
            BoxValue::new(value).map_err(to_value_error)?,
            tree,
            creation_height,
        );
        for token in tokens.into_iter().flatten() {
            builder.add_token(token.into());
        }
        for (id, value) in registers.into_iter().flatten() {
            builder.set_register_value(id.into(), value.into());
        }
        builder.build().map(Self).map_err(to_value_error)
    }
    fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }
}

#[pyclass(eq)]
#[derive(PartialEq, Eq, Clone)]
pub struct ErgoBox(pub ergo_box::ErgoBox);

#[pymethods]
impl ErgoBox {
    #[new]
    fn new(dict: Bound<'_, PyDict>) -> PyResult<Self> {
        from_pyobject::<ergo_box::ErgoBox, PyDict>(dict)
            .map(Self)
            .map_err(to_value_error)
    }
    #[getter]
    fn box_id(&self) -> BoxId {
        self.0.box_id().into()
    }
    #[getter]
    fn value(&self) -> u64 {
        *self.0.value.as_u64()
    }
    #[getter]
    fn creation_height(&self) -> u32 {
        self.0.creation_height
    }
    #[getter]
    fn tokens(&self) -> Vec<Token> {
        self.0
            .tokens
            .iter()
            .flatten()
            .copied()
            .map(Into::into)
            .collect()
    }
    #[getter]
    fn registers(&self) -> PyResult<HashMap<NonMandatoryRegisterId, Constant>> {
        ergo_box::NonMandatoryRegisterId::REG_IDS
            .into_iter()
            .flat_map(|id| {
                Some((
                    NonMandatoryRegisterId::from(id),
                    self.0.additional_registers.get_constant(id).transpose()?,
                ))
            })
            .map(|(id, val)| val.map(|val| (id, val.into())))
            .collect::<Result<_, _>>()
            .map_err(to_value_error)
    }
    #[getter]
    fn ergo_tree(&self) -> ErgoTree {
        self.0.ergo_tree.clone().into()
    }
    #[pyo3(text_signature = "(self) -> str")]
    fn json(&self) -> PyResult<String> {
        serde_json::to_string(&self.0)
            .map_err(JsonError::from)
            .map_err(Into::into)
    }
    #[staticmethod]
    fn from_json(json: &str) -> PyResult<Self> {
        Ok(Self(serde_json::from_str(json).map_err(JsonError::from)?))
    }
    fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }
}
