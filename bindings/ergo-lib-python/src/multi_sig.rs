use ergo_lib::{
    ergotree_ir::sigma_protocol::sigma_boolean::{SigmaBoolean, SigmaProofOfKnowledgeTree},
    wallet::{multi_sig, signing::TransactionContext},
};
use hints::{RealCommitment, RealSecretProof, SimulatedCommitment, SimulatedSecretProof};
use hints_bag::{HintsBag, TransactionHintsBag};
use pyo3::prelude::*;

use crate::{
    chain::{ergo_box::ErgoBox, ergo_state_context::ErgoStateContext},
    sigma_protocol::ProveDlog,
    to_value_error,
    transaction::Transaction,
};

pub mod hints;
pub mod hints_bag;
#[pyfunction]
fn extract_hints(
    tx: Transaction,
    boxes_to_spend: Vec<ErgoBox>,
    data_boxes: Vec<ErgoBox>,
    state_context: &ErgoStateContext,
    real_propositions: Vec<ProveDlog>,
    simulated_propositions: Vec<ProveDlog>,
) -> PyResult<TransactionHintsBag> {
    let real_propositions = real_propositions
        .into_iter()
        .map(|prove_dlog| prove_dlog.0)
        .map(SigmaProofOfKnowledgeTree::from)
        .map(SigmaBoolean::from)
        .collect();
    let simulated_propositions = simulated_propositions
        .into_iter()
        .map(|prove_dlog| prove_dlog.0)
        .map(SigmaProofOfKnowledgeTree::from)
        .map(SigmaBoolean::from)
        .collect();
    let tx_context = TransactionContext::new(
        tx.into(),
        boxes_to_spend.into_iter().map(Into::into).collect(),
        data_boxes.into_iter().map(Into::into).collect(),
    )
    .map_err(to_value_error)?;
    multi_sig::extract_hints(
        &tx_context,
        &state_context.0,
        real_propositions,
        simulated_propositions,
    )
    .map(Into::into)
    .map_err(to_value_error)
}

pub(crate) fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let submodule = PyModule::new(m.py(), "multi_sig")?;
    submodule.add_class::<HintsBag>()?;
    submodule.add_class::<TransactionHintsBag>()?;
    submodule.add_class::<RealCommitment>()?;
    submodule.add_class::<SimulatedCommitment>()?;
    submodule.add_class::<RealSecretProof>()?;
    submodule.add_class::<SimulatedSecretProof>()?;
    submodule.add_function(wrap_pyfunction!(extract_hints, m)?)?;
    m.add_submodule(&submodule)?;
    m.py()
        .import("sys")?
        .getattr("modules")?
        .set_item("ergo_lib_python.multi_sig", submodule)?;
    Ok(())
}
