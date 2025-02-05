use derive_more::From;
use ergo_lib::ergotree_ir::serialization;
use pyo3::{create_exception, exceptions::PyException, PyErr};

create_exception!(
    ergo_lib_python,
    JsonException,
    PyException,
    "Error during JSON deserialization"
);
create_exception!(
    ergo_lib_python,
    SigmaSerializationException,
    PyException,
    "Error during sigma serialization"
);

#[derive(From)]
pub struct SigmaSerializationError(serialization::SigmaSerializationError);
impl From<SigmaSerializationError> for PyErr {
    fn from(err: SigmaSerializationError) -> Self {
        SigmaSerializationException::new_err(err.0.to_string())
    }
}

#[derive(From)]
pub struct JsonError(serde_json::Error);
impl From<JsonError> for PyErr {
    fn from(err: JsonError) -> Self {
        JsonException::new_err(err.0.to_string())
    }
}
