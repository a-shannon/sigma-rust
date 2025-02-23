use std::{collections::HashMap, str::FromStr};

use derive_more::{From, Into};
use ergo_lib::{
    chain::ergo_box::box_builder::ErgoBoxCandidateBuilder,
    ergotree_ir::{
        chain::ergo_box::{self, box_value::BoxValue, NonMandatoryRegisters},
        serialization::SigmaSerializable,
    },
};
use pyo3::{exceptions::PyValueError, prelude::*, types::PyDict};
use serde_pyobject::from_pyobject;

use crate::{
    ergo_tree::ErgoTree,
    errors::{JsonError, SigmaParsingError, SigmaSerializationError},
    to_value_error,
    transaction::TxId,
};

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
pub struct BoxId(pub ergo_box::BoxId);

#[pymethods]
impl BoxId {
    #[new]
    fn new(val: &Bound<'_, PyAny>) -> PyResult<Self> {
        match val.extract::<&str>() {
            Ok(s) => ergo_box::BoxId::from_str(s)
                .map_err(to_value_error)
                .map(Self),
            Err(_) => match val.extract::<&[u8]>() {
                Ok(bytes) => ergo_box::BoxId::sigma_parse_bytes(bytes)
                    .map_err(to_value_error)
                    .map(Self),
                Err(_) => Err(PyValueError::new_err(
                    "TokenId.new: missing bytes or str argument",
                )),
            },
        }
    }
    fn __bytes__(&self) -> Vec<u8> {
        #[allow(clippy::unwrap_used)]
        self.0.sigma_serialize_bytes().unwrap()
    }
    fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }
}

#[pyclass(eq)]
#[derive(Clone, PartialEq, Eq, From, Into, Debug)]
pub struct ErgoBoxCandidate(ergo_box::ErgoBoxCandidate);

#[pymethods]
impl ErgoBoxCandidate {
    #[allow(clippy::too_many_arguments)]
    #[new]
    #[pyo3(signature=(*, value, address=None, ergo_tree=None, creation_height, tokens=None, registers=None, mint_token= None, mint_token_name = None, mint_token_desc=None, mint_token_decimals=None))]
    fn new(
        value: u64,
        address: Option<Address>,
        ergo_tree: Option<ErgoTree>,
        creation_height: u32,
        tokens: Option<Vec<Token>>,
        registers: Option<HashMap<NonMandatoryRegisterId, Constant>>,
        mint_token: Option<Token>,
        mint_token_name: Option<&str>,
        mint_token_desc: Option<&str>,
        mint_token_decimals: Option<usize>,
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
        if let Some(mint_token) = mint_token {
            (|| {
                builder.mint_token(
                    mint_token.into(),
                    mint_token_name?.into(),
                    mint_token_desc?.into(),
                    mint_token_decimals?,
                );
                Some(())
            })()
            .ok_or_else(|| {
                PyValueError::new_err(
                    "Expected mint_token_name, mint_token_desc, mint_token_decimals",
                )
            })?;
        }
        builder.build().map(Self).map_err(to_value_error)
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
        extract_registers(&self.0.additional_registers)
    }
    #[getter]
    fn ergo_tree(&self) -> ErgoTree {
        self.0.ergo_tree.clone().into()
    }
    fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }
}

#[pyclass(eq)]
#[derive(PartialEq, Eq, Clone, From, Into)]
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
        extract_registers(&self.0.additional_registers)
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
    fn from_box_candidate(candidate: ErgoBoxCandidate, tx_id: TxId, index: u16) -> PyResult<Self> {
        ergo_box::ErgoBox::from_box_candidate(&candidate.into(), tx_id.into(), index)
            .map(Into::into)
            .map_err(SigmaSerializationError::from)
            .map_err(Into::into)
    }
    #[staticmethod]
    fn from_json(json: &str) -> PyResult<Self> {
        Ok(Self(serde_json::from_str(json).map_err(JsonError::from)?))
    }
    #[staticmethod]
    fn from_bytes(bytes: &[u8]) -> PyResult<Self> {
        ergo_box::ErgoBox::sigma_parse_bytes(bytes)
            .map(Self)
            .map_err(SigmaParsingError::from)
            .map_err(Into::into)
    }
    fn __bytes__(&self) -> PyResult<Vec<u8>> {
        self.0
            .sigma_serialize_bytes()
            .map_err(SigmaSerializationError::from)
            .map_err(Into::into)
    }
    fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }
}

fn extract_registers(
    additional_registers: &NonMandatoryRegisters,
) -> PyResult<HashMap<NonMandatoryRegisterId, Constant>> {
    ergo_box::NonMandatoryRegisterId::REG_IDS
        .into_iter()
        .flat_map(|id| {
            Some((
                NonMandatoryRegisterId::from(id),
                additional_registers.get_constant(id).transpose()?,
            ))
        })
        .map(|(id, val)| val.map(|val| (id, val.into())))
        .collect::<Result<_, _>>()
        .map_err(to_value_error)
}
