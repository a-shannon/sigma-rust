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

pub(crate) mod chain;
mod ergo_tree;
mod errors;
pub(crate) mod multi_sig;
mod nipopow;
pub(crate) mod sigma_protocol;
pub(crate) mod transaction;
mod verifier;
pub mod wallet;
use errors::JsonError;
use pyo3::{exceptions::PyValueError, prelude::*, types::PyDict};
use serde::de::DeserializeOwned;
use serde_pyobject::from_pyobject;

// Create python ValueError from generic error
pub(crate) fn to_value_error<E: std::error::Error>(e: E) -> PyErr {
    PyValueError::new_err(e.to_string())
}

pub(crate) fn from_json<T: DeserializeOwned>(json: Bound<'_, PyAny>) -> PyResult<T> {
    let res = match json.downcast_into::<PyDict>() {
        Ok(dict) => from_pyobject::<T, PyDict>(dict).map_err(to_value_error)?,
        Err(json) => {
            serde_json::from_str(json.into_inner().extract::<&str>()?).map_err(JsonError::from)?
        }
    };
    Ok(res)
}

#[pymodule]
fn ergo_lib_python(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    wallet::register(m)?;
    chain::register(m)?;
    transaction::register(m)?;
    sigma_protocol::register(m)?;
    multi_sig::register(m)?;
    verifier::register(m)?;
    errors::register(m)?;
    ergo_tree::register(m)?;
    nipopow::register(m)?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
