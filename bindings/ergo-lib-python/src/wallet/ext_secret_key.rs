use derive_more::From;
use ergo_lib::wallet::{derivation_path::ChildIndex, ext_secret_key};
use pyo3::{pyclass, pymethods, PyResult};

use crate::to_value_error;

use super::{derivation_path::DerivationPath, ext_pub_key::ExtPubKey};

#[pyclass(eq, frozen)]
#[derive(PartialEq, Eq, From)]
pub struct ExtSecretKey(ext_secret_key::ExtSecretKey);

#[pymethods]
impl ExtSecretKey {
    /// Create new ExtSecretKey from seed
    #[staticmethod]
    pub fn derive_master(seed: &[u8]) -> PyResult<Self> {
        ext_secret_key::ExtSecretKey::derive_master(seed.try_into().map_err(to_value_error)?)
            .map(Self)
            .map_err(to_value_error)
    }
    /// Derivation path associated with this ExtSecretKey
    pub fn path(&self) -> DerivationPath {
        self.0.path().into()
    }
    /// Derive a new extended secret key from the provided index
    /// The index is in the form of soft or hardened indices
    /// For example: 4 or 4' respectively
    pub fn child(&self, index: &str) -> PyResult<ExtSecretKey> {
        let idx = index.parse::<ChildIndex>().map_err(to_value_error)?;
        Ok(self.0.child(idx).map_err(to_value_error)?.into())
    }
    /// Derive new ExtSecretKey from up_path
    #[pyo3(text_signature = "(self, up_path: DerivationPath)")]
    pub fn derive(&self, up_path: DerivationPath) -> PyResult<Self> {
        self.0.derive(up_path.0).map(Self).map_err(to_value_error)
    }

    pub fn public_key(&self) -> PyResult<ExtPubKey> {
        self.0
            .public_key()
            .map(ExtPubKey::from)
            .map_err(to_value_error)
    }
}
