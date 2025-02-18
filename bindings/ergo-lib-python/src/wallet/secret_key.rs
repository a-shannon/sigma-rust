use derive_more::{From, Into};
use ergo_lib::wallet::secret_key;
use pyo3::{pyclass, pymethods, PyResult};

use crate::{errors::JsonError, to_value_error};

/// Secret Key
#[pyclass(eq, frozen, str = "{0:?}")]
#[derive(PartialEq, Eq, Clone, From, Into)]
pub struct SecretKey(secret_key::SecretKey);

#[pymethods]
impl SecretKey {
    /// Create a random ProveDlog SecretKey
    #[staticmethod]
    fn random_dlog() -> Self {
        Self(secret_key::SecretKey::random_dlog())
    }
    /// Create a random ProveDHTuple SecretKey
    #[staticmethod]
    fn random_dht() -> Self {
        Self(secret_key::SecretKey::random_dht())
    }
    /// Deserialize SecretKey from json
    #[staticmethod]
    #[pyo3(text_signature = "(s: str) -> SecretKey")]
    fn from_json(s: &str) -> Result<Self, JsonError> {
        Ok(Self(serde_json::from_str(s)?))
    }
    #[pyo3(text_signature = "(self) -> str")]
    fn json(&self) -> PyResult<String> {
        serde_json::to_string(&self.0)
            .map_err(JsonError::from)
            .map_err(Into::into)
    }
    fn __bytes__(&self) -> Vec<u8> {
        self.0.to_bytes()
    }
    #[staticmethod]
    #[pyo3(text_signature = "(bytes: bytes) -> SecretKey")]
    fn from_bytes(bytes: &[u8]) -> PyResult<SecretKey> {
        secret_key::SecretKey::from_bytes(bytes)
            .map(Self)
            .map_err(to_value_error)
    }
    fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }
}
