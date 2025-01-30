use derive_more::From;
use pyo3::{create_exception, exceptions::PyException, PyErr};

create_exception!(
    ergo_lib_python,
    JsonException,
    PyException,
    "Error during JSON deserialization"
);
#[derive(From)]
pub struct JsonError(serde_json::Error);
impl From<JsonError> for PyErr {
    fn from(err: JsonError) -> Self {
        JsonException::new_err(err.0.to_string())
    }
}
