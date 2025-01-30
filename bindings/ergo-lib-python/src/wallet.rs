use derivation_path::DerivationPath;
use ext_secret_key::ExtSecretKey;
use mnemonic::MnemonicGenerator;
use pyo3::{
    types::{PyModule, PyModuleMethods},
    wrap_pyfunction, Bound, PyResult,
};
use secret_key::SecretKey;

mod derivation_path;
mod ext_secret_key;
mod mnemonic;
mod secret_key;
mod ext_pub_key;

// Register all classes & functions of this module. This does not create a submodule because of a python limitation that would prevent 'from ergo_lib import submodule'
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<SecretKey>()?;
    m.add_class::<MnemonicGenerator>()?;
    m.add_class::<ExtSecretKey>()?;
    m.add_class::<DerivationPath>()?;
    m.add_function(wrap_pyfunction!(mnemonic::to_seed, m)?)?;
    Ok(())
}
