use pyo3::{pyclass, pymethods, PyResult};

#[pyclass]
pub struct SecretKey(ergo_lib::wallet::secret_key::SecretKey);

#[pymethods]
impl SecretKey {
    #[staticmethod]
    fn random_dlog() -> Self {
        Self(ergo_lib::wallet::secret_key::SecretKey::random_dlog())
    }
    #[staticmethod]
    fn from_json(s: &str) -> PyResult<Self> {
        Ok(Self(serde_json::from_str(s)?))
    }
    fn json(&self) -> String {
        serde_json::to_string(&self.0).unwrap()
    }
}
