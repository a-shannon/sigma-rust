use ergo_lib::ergotree_ir::sigma_protocol::sigma_boolean::SigmaBoolean;
use pyo3::{exceptions::PyValueError, prelude::*};

use crate::{chain::address::Address, to_value_error};

#[pyfunction]
fn verify_signature(address: &Address, message: &[u8], signature: &[u8]) -> PyResult<bool> {
    if let Address(ergo_lib::ergotree_ir::chain::address::Address::P2Pk(d)) = address.clone() {
        let sb = SigmaBoolean::from(d);
        ergo_lib::ergotree_interpreter::sigma_protocol::verifier::verify_signature(
            sb, message, signature,
        )
        .map_err(to_value_error)
    } else {
        Err(PyValueError::new_err(
            "wallet::verify_signature: Address:P2Pk expected",
        ))
    }
}

pub(crate) fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let submodule = PyModule::new(m.py(), "verifier")?;
    submodule.add_function(wrap_pyfunction!(verify_signature, m)?)?;
    m.add_submodule(&submodule)?;
    m.py()
        .import("sys")?
        .getattr("modules")?
        .set_item("ergo_lib_python.verifier", submodule)?;
    Ok(())
}
