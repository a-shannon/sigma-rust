use box_selector::{select_boxes_simple, BoxSelection, ErgoBoxAssetsData};
use derivation_path::DerivationPath;
use ergo_lib::ergotree_ir::sigma_protocol::sigma_boolean::SigmaBoolean;
use ergo_lib::wallet::signing::TransactionContext;
use ergo_lib::wallet::Wallet as WalletInner;
use ext_pub_key::ExtPubKey;
use ext_secret_key::ExtSecretKey;
use mnemonic::MnemonicGenerator;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::{
    types::{PyModule, PyModuleMethods},
    wrap_pyfunction, Bound, PyResult,
};
use secret_key::SecretKey;

use crate::chain::address::Address;
use crate::chain::ergo_box::ErgoBox;
use crate::chain::ergo_state_context::ErgoStateContext;
use crate::errors::WalletError;
use crate::multi_sig::hints_bag::TransactionHintsBag;
use crate::to_value_error;
use crate::transaction::input::Input;
use crate::transaction::{ReducedTransaction, Transaction, UnsignedTransaction};

pub mod box_selector;
mod derivation_path;
mod ext_pub_key;
mod ext_secret_key;
mod mnemonic;
mod secret_key;

#[pyclass]
pub(crate) struct Wallet(WalletInner);
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
    #[pyo3(signature = (tx, boxes_to_spend=vec![], data_boxes=vec![], state_context=None, *, hints_bag=None))]
    fn sign_transaction(
        &self,
        tx: &Bound<'_, PyAny>,
        boxes_to_spend: Vec<ErgoBox>,
        data_boxes: Vec<ErgoBox>,
        state_context: Option<&ErgoStateContext>,
        hints_bag: Option<&TransactionHintsBag>,
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
                    let tx_context = build_tx_context(unsigned_tx, boxes_to_spend, data_boxes)?;
                    let state_context = state_context
                        .map(AsRef::as_ref)
                        .ok_or_else(|| PyValueError::new_err("missing argument state_context"))?;
                    self.0
                        .sign_transaction(tx_context, state_context, hints_bag.map(AsRef::as_ref))
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
    #[pyo3(signature = (tx, boxes_to_spend=vec![], data_boxes=vec![], state_context=None))]
    fn generate_commitments(
        &self,
        tx: &Bound<'_, PyAny>,
        boxes_to_spend: Vec<ErgoBox>,
        data_boxes: Vec<ErgoBox>,
        state_context: Option<&ErgoStateContext>,
    ) -> PyResult<TransactionHintsBag> {
        match tx.extract::<ReducedTransaction>() {
            Ok(reduced_tx) => self
                .0
                .generate_commitments_for_reduced_transaction(reduced_tx.into())
                .map(Into::into)
                .map_err(Into::into)
                .map_err(WalletError)
                .map_err(Into::into),
            Err(e) => match tx.extract::<UnsignedTransaction>() {
                Ok(unsigned_tx) => {
                    let tx_context = build_tx_context(unsigned_tx, boxes_to_spend, data_boxes)?;
                    let state_context = state_context
                        .map(AsRef::as_ref)
                        .ok_or_else(|| PyValueError::new_err("missing argument state_context"))?;
                    self.0
                        .generate_commitments(tx_context, state_context)
                        .map(Into::into)
                        .map_err(Into::into)
                        .map_err(WalletError)
                        .map_err(Into::into)
                }
                Err(e) => Err(PyValueError::new_err(
                    "Expected ReducedTransaction or Transaction",
                )),
            },
        }
    }

    #[pyo3(signature = (tx, input_idx, boxes_to_spend, data_boxes, state_context, *, hints_bag=None))]
    fn sign_tx_input(
        &self,
        tx: UnsignedTransaction,
        input_idx: usize,
        boxes_to_spend: Vec<ErgoBox>,
        data_boxes: Vec<ErgoBox>,
        state_context: &ErgoStateContext,
        hints_bag: Option<&TransactionHintsBag>,
    ) -> PyResult<Input> {
        let tx_context = build_tx_context(tx, boxes_to_spend, data_boxes)?;
        self.0
            .sign_tx_input(
                input_idx,
                tx_context,
                state_context.as_ref(),
                hints_bag.map(AsRef::as_ref),
            )
            .map(Into::into)
            .map_err(WalletError::from)
            .map_err(Into::into)
    }
    fn sign_message_using_p2pk(&self, address: &Address, message: &[u8]) -> PyResult<Vec<u8>> {
        if let Address(ergo_lib::ergotree_ir::chain::address::Address::P2Pk(d)) = address.clone() {
            let sb = SigmaBoolean::from(d);
            self.0
                .sign_message(sb, message)
                .map_err(WalletError::from)
                .map_err(Into::into)
        } else {
            Err(PyValueError::new_err(
                "wallet::sign_message_using_p2pk: Address:P2Pk expected",
            ))
        }
    }
}

fn build_tx_context(
    unsigned_tx: UnsignedTransaction,
    boxes_to_spend: Vec<ErgoBox>,
    data_boxes: Vec<ErgoBox>,
) -> PyResult<TransactionContext<ergo_lib::chain::transaction::unsigned::UnsignedTransaction>> {
    TransactionContext::new(
        unsigned_tx.0,
        boxes_to_spend.into_iter().map(Into::into).collect(),
        data_boxes.into_iter().map(Into::into).collect(),
    )
    .map_err(to_value_error)
}
// Register all classes & functions of this module. This does not create a submodule because of a python limitation that would prevent 'from ergo_lib import submodule'
pub(crate) fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let submodule = PyModule::new(m.py(), "wallet")?;
    submodule.add_class::<SecretKey>()?;
    submodule.add_class::<MnemonicGenerator>()?;
    submodule.add_class::<ExtSecretKey>()?;
    submodule.add_class::<ExtPubKey>()?;
    submodule.add_class::<DerivationPath>()?;
    submodule.add_class::<BoxSelection>()?;
    submodule.add_class::<Wallet>()?;
    submodule.add_class::<ErgoBoxAssetsData>()?;
    submodule.add_function(wrap_pyfunction!(select_boxes_simple, m)?)?;
    submodule.add_function(wrap_pyfunction!(mnemonic::to_seed, m)?)?;
    m.add_submodule(&submodule)?;
    m.py()
        .import("sys")?
        .getattr("modules")?
        .set_item("ergo_lib_python.wallet", submodule)?;
    Ok(())
}
