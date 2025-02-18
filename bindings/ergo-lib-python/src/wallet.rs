use box_selector::{select_boxes_simple, BoxSelection};
use derivation_path::DerivationPath;
use ergo_lib::wallet::signing::TransactionContext;
use ergo_lib::wallet::Wallet as WalletInner;
use ext_secret_key::ExtSecretKey;
use mnemonic::MnemonicGenerator;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::{
    types::{PyModule, PyModuleMethods},
    wrap_pyfunction, Bound, PyResult,
};
use secret_key::SecretKey;

use crate::chain::ergo_box::ErgoBox;
use crate::chain::ergo_state_context::ErgoStateContext;
use crate::errors::WalletError;
use crate::to_value_error;
use crate::transaction::{ReducedTransaction, Transaction, UnsignedTransaction};

pub mod box_selector;
mod derivation_path;
mod ext_pub_key;
mod ext_secret_key;
mod mnemonic;
mod secret_key;

#[pyclass]
pub struct Wallet(WalletInner);
#[pymethods]
impl Wallet {
    #[new]
    fn new(secrets: Vec<SecretKey>) -> Self {
        Self(WalletInner::from_secrets(
            secrets.into_iter().map(Into::into).collect(),
        ))
    }
    fn add_secret(&mut self, secret: SecretKey) {
        self.0.add_secret(secret.into());
    }
    fn sign_transaction(
        &self,
        tx: &Bound<'_, PyAny>,
        boxes_to_spend: Vec<ErgoBox>,
        data_boxes: Vec<ErgoBox>,
        state_context: Option<ErgoStateContext>,
    ) -> PyResult<Transaction> {
        match tx.extract::<ReducedTransaction>() {
            Ok(reduced_tx) => self
                .0
                .sign_reduced_transaction(reduced_tx.into(), None)
                .map(Into::into)
                .map_err(WalletError::from)
                .map_err(Into::into),
            Err(e) => match tx.extract::<UnsignedTransaction>() {
                Ok(unsigned_tx) => {
                    let tx_context = TransactionContext::new(
                        unsigned_tx.0,
                        boxes_to_spend.into_iter().map(Into::into).collect(),
                        data_boxes.into_iter().map(Into::into).collect(),
                    )
                    .map_err(to_value_error)?;
                    let state_context = state_context
                        .ok_or_else(|| PyValueError::new_err("missing argument state_context"))?
                        .into();
                    self.0
                        .sign_transaction(tx_context, &state_context, None)
                        .map(Into::into)
                        .map_err(WalletError::from)
                        .map_err(Into::into)
                }
                Err(e) => Err(PyValueError::new_err(
                    "Expected ReducedTransaction or Transaction",
                )),
            },
        }
    }
}
// Register all classes & functions of this module. This does not create a submodule because of a python limitation that would prevent 'from ergo_lib import submodule'
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<SecretKey>()?;
    m.add_class::<MnemonicGenerator>()?;
    m.add_class::<ExtSecretKey>()?;
    m.add_class::<DerivationPath>()?;
    m.add_class::<BoxSelection>()?;
    m.add_function(wrap_pyfunction!(select_boxes_simple, m)?)?;
    m.add_function(wrap_pyfunction!(mnemonic::to_seed, m)?)?;
    Ok(())
}
