use ergo_lib::wallet::ext_pub_key;
use pyo3::{pyclass, pymethods};
#[pyclass(frozen)]
#[allow(dead_code)]
pub struct ExtPubKey(ext_pub_key::ExtPubKey);

#[pymethods]
impl ExtPubKey {}
