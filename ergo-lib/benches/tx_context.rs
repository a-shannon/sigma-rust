use criterion::{criterion_group, criterion_main, BatchSize};

use criterion::{BenchmarkId, Criterion, SamplingMode};
use ergo_lib::chain::ergo_box::box_builder::ErgoBoxCandidateBuilder;
use ergo_lib::{
    chain::transaction::{unsigned::UnsignedTransaction, UnsignedInput},
    wallet::{secret_key::SecretKey, signing::TransactionContext, Wallet},
};
use ergotree_ir::chain::ergo_box::box_value::BoxValue;
use ergotree_ir::chain::tx_id::TxId;
use ergotree_ir::mir::expr::Expr;
use ergotree_ir::{
    chain::{context_extension::ContextExtension, ergo_box::ErgoBox},
    ergo_tree::{ErgoTree, ErgoTreeHeader},
    mir::constant::Constant,
};
use sigma_test_util::force_any_val;

pub fn bench_tx_context(c: &mut Criterion) {
    let mut group = c.benchmark_group("TransactionContext::new scaling with inputs");
    group.sample_size(10);
    let true_tree = ErgoTree::new(ErgoTreeHeader::v0(true), &Expr::Const(true.into())).unwrap();

    for range in (1..=100).step_by(10) {
        let inputs = (0..range)
            .map(|i| {
                ErgoBox::from_box_candidate(
                    &ErgoBoxCandidateBuilder::new(BoxValue::SAFE_USER_MIN, true_tree.clone(), 1000)
                        .build()
                        .unwrap(),
                    TxId::zero(),
                    i as u16,
                )
                .unwrap()
            })
            .collect::<Vec<ErgoBox>>();
        let input_ids = inputs
            .iter()
            .map(|input| UnsignedInput::new(input.box_id(), ContextExtension::empty()))
            .collect();
        let unsigned_tx = UnsignedTransaction::new_from_vec(
            input_ids,
            vec![],
            vec![
                ErgoBoxCandidateBuilder::new(BoxValue::SAFE_USER_MIN, true_tree.clone(), 1000)
                    .build()
                    .unwrap(),
            ],
        )
        .unwrap();
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{range:4}")),
            &unsigned_tx,
            |b, tx| {
                b.iter_batched(
                    || (tx.clone(), inputs.clone(), vec![]),
                    |(tx, inputs, data_inputs)| {
                        TransactionContext::new(tx, inputs, data_inputs).unwrap()
                    },
                    BatchSize::SmallInput,
                )
            },
        );
    }
    group.finish();
}

pub fn bench_tx_sign_wallet(c: &mut Criterion) {
    let mut sign_group = c.benchmark_group("sign ProveDlog inputs");
    sign_group
        .sample_size(10)
        .sampling_mode(SamplingMode::Flat)
        .warm_up_time(std::time::Duration::from_millis(50));

    const MAX_INPUTS: usize = 100;
    let secrets = vec![SecretKey::random_dlog(); MAX_INPUTS];
    let inputs = (0..MAX_INPUTS)
        .map(|i| {
            ErgoBox::from_box_candidate(
                &ErgoBoxCandidateBuilder::new(
                    BoxValue::SAFE_USER_MIN,
                    secrets[i].get_address_from_public_image().script().unwrap(),
                    1000,
                )
                .build()
                .unwrap(),
                TxId::zero(),
                i as u16,
            )
            .unwrap()
        })
        .collect::<Vec<ErgoBox>>();
    let wallet = Wallet::from_secrets(secrets);

    for range in (1..=MAX_INPUTS).step_by(20) {
        let input_ids = inputs
            .iter()
            .take(range)
            .map(|input| UnsignedInput::new(input.box_id(), ContextExtension::empty()))
            .collect();
        let unsigned_tx = UnsignedTransaction::new_from_vec(
            input_ids,
            vec![],
            vec![ErgoBoxCandidateBuilder::new(
                BoxValue::SAFE_USER_MIN,
                ErgoTree::new(
                    ErgoTreeHeader::new(0).unwrap(),
                    &Constant::from(true).into(),
                )
                .unwrap(),
                1000,
            )
            .build()
            .unwrap()],
        )
        .unwrap();
        let tx_context =
            TransactionContext::new(unsigned_tx, inputs[0..range].to_owned(), vec![]).unwrap();
        sign_group.bench_with_input(
            BenchmarkId::from_parameter(format!("{range:4}")),
            &tx_context,
            |b, tx_context| {
                b.iter_batched(
                    || (tx_context.clone(), force_any_val()),
                    |(tx_context, state_context)| {
                        wallet
                            .sign_transaction(tx_context, &state_context, None)
                            .unwrap()
                    },
                    BatchSize::SmallInput,
                )
            },
        );
    }
    sign_group.finish();
}

criterion_group!(benches, bench_tx_context, bench_tx_sign_wallet);
criterion_main!(benches);
