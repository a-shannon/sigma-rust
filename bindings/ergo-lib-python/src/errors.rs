use derive_more::From;
use ergo_lib::ergotree_ir::serialization;
use pyo3::prelude::*;
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
create_exception!(
    ergo_lib_python,
    SigmaParsingException,
    PyException,
    "Error during sigma serialization"
);
create_exception!(
    ergo_lib_python,
    WalletException,
    PyException,
    "error during wallet-related operation"
);
create_exception!(
    ergo_lib_python,
    RegisterValueException,
    PyException,
    "error parsing register value"
);

#[derive(From)]
pub(crate) struct SigmaSerializationError(serialization::SigmaSerializationError);
impl From<SigmaSerializationError> for PyErr {
    fn from(err: SigmaSerializationError) -> Self {
        SigmaSerializationException::new_err(err.0.to_string())
    }
}

#[derive(From)]
pub(crate) struct SigmaParsingError(serialization::SigmaParsingError);
impl From<SigmaParsingError> for PyErr {
    fn from(err: SigmaParsingError) -> Self {
        SigmaParsingException::new_err(err.0.to_string())
    }
}

#[derive(From)]
pub(crate) struct JsonError(serde_json::Error);
impl From<JsonError> for PyErr {
    fn from(err: JsonError) -> Self {
        JsonException::new_err(err.0.to_string())
    }
}

#[derive(From)]
pub(crate) struct WalletError(pub ergo_lib::wallet::WalletError);
impl From<WalletError> for PyErr {
    fn from(err: WalletError) -> Self {
        WalletException::new_err(err.0.to_string())
    }
}

#[derive(From)]
pub(crate) struct RegisterValueError(
    pub ergo_lib::ergotree_ir::chain::ergo_box::RegisterValueError,
);
impl From<RegisterValueError> for PyErr {
    fn from(err: RegisterValueError) -> Self {
        RegisterValueException::new_err(err.0.to_string())
    }
}

pub(crate) fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let submodule = PyModule::new(m.py(), "exceptions")?;
    submodule.add("JsonException", m.py().get_type::<JsonException>())?;
    submodule.add(
        "SigmaSerializationException",
        m.py().get_type::<SigmaSerializationException>(),
    )?;
    submodule.add(
        "SigmaParsingException",
        m.py().get_type::<SigmaParsingException>(),
    )?;
    submodule.add("WalletException", m.py().get_type::<WalletException>())?;
    submodule.add(
        "RegisterValueException",
        m.py().get_type::<RegisterValueException>(),
    )?;
    m.add_submodule(&submodule)?;
    m.py()
        .import("sys")?
        .getattr("modules")?
        .set_item("ergo_lib_python.exceptions", submodule)?;
    Ok(())
}
