//! Ergo Python bindings
// Coding conventions
#![forbid(unsafe_code)]
#![deny(non_upper_case_globals)]
#![deny(non_camel_case_types)]
#![deny(non_snake_case)]
#![deny(unused_mut)]
#![deny(dead_code)]
#![deny(unused_imports)]
#![allow(unused_variables)]
// Clippy warnings
#![allow(clippy::new_without_default)]
#![allow(clippy::len_without_is_empty)]
#![allow(clippy::unused_unit)]
#![deny(clippy::wildcard_enum_match_arm)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]

pub mod chain;
mod ergo_tree;
mod errors;
pub mod multi_sig;
pub mod sigma_boolean;
pub mod transaction;
mod verifier;
pub mod wallet;
use ergo_tree::ErgoTree;
use errors::{JsonException, SigmaParsingException, SigmaSerializationException, WalletException};
use pyo3::{exceptions::PyValueError, prelude::*};

// Create python ValueError from generic error
pub(crate) fn to_value_error<E: std::error::Error>(e: E) -> PyErr {
    PyValueError::new_err(e.to_string())
}
#[pymodule]
fn ergo_lib_python(m: &Bound<'_, PyModule>) -> PyResult<()> {
    wallet::register(m)?;
    chain::register(m)?;
    transaction::register(m)?;
    sigma_boolean::register(m)?;
    multi_sig::register(m)?;
    verifier::register(m)?;
    m.add("JsonException", m.py().get_type::<JsonException>())?;
    m.add(
        "SigmaSerializationException",
        m.py().get_type::<SigmaSerializationException>(),
    )?;
    m.add(
        "SigmaSerializationException",
        m.py().get_type::<SigmaParsingException>(),
    )?;
    m.add("WalletException", m.py().get_type::<WalletException>())?;
    m.add_class::<ErgoTree>()?;
    Ok(())
}
