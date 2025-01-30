//! Ergo Python bindings
// Coding conventions
#![forbid(unsafe_code)]
#![deny(non_upper_case_globals)]
#![deny(non_camel_case_types)]
#![deny(non_snake_case)]
#![deny(unused_mut)]
#![deny(dead_code)]
#![deny(unused_imports)]
#![deny(missing_docs)]
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

mod errors;
mod wallet;
use pyo3::{exceptions::PyValueError, prelude::*};

// Create python ValueError from generic error
pub(crate) fn to_value_error<E: std::error::Error>(e: E) -> PyErr {
    PyValueError::new_err(e.to_string())
}
#[pymodule]
fn ergo_lib_python(m: &Bound<'_, PyModule>) -> PyResult<()> {
    wallet::register(m)?;
    Ok(())
}
