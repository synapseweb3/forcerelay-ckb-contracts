use ibc_ckb_contracts_test_utils::{
    ckb::{
        script::ScriptVersion,
        types::{core::TransactionBuilder, packed, prelude::*},
    },
    misc, Context, Verifier,
};

use super::REVERSE_ARGS_LOCK_CONTRACT;
use crate::setup;

#[test]
fn test() {
    setup();

    let args_bytes = misc::random_bytes();
    let args: packed::Bytes = args_bytes.pack();

    let mut context = Context::new();
    let script_version = ScriptVersion::latest();

    let deployed_contract = {
        let contract_data = misc::load_contract_from_file(REVERSE_ARGS_LOCK_CONTRACT);
        let data = contract_data.into();
        let lock_script = packed::Script::default();
        context.deploy(data, lock_script, None)
    };

    let deployed_cell = {
        let data = Default::default();
        let lock_script = packed::Script::new_builder()
            .hash_type(script_version.data_hash_type().into())
            .code_hash(deployed_contract.data_hash())
            .args(args.clone())
            .build();
        context.deploy(data, lock_script, None)
    };

    let transaction = {
        let lock_script = deployed_cell.cell_output().lock();
        let output1 = packed::CellOutput::new_builder().lock(lock_script).build();
        let output2 = packed::CellOutput::new_builder().build();
        let witness2 = {
            let args_rev: packed::Bytes = args_bytes.into_iter().rev().collect::<Vec<_>>().pack();
            let lock_args = packed::BytesOpt::new_builder().set(Some(args_rev)).build();
            let witness_args = packed::WitnessArgs::new_builder().lock(lock_args).build();
            witness_args.as_bytes()
        };
        let output3 = output1.clone();
        TransactionBuilder::default()
            .cell_dep(deployed_contract.as_cell_dep())
            .input(deployed_cell.as_input())
            .output(output1)
            .output_data(Default::default())
            .witness(Default::default())
            .output(output2)
            .output_data(Default::default())
            .witness(witness2.pack())
            .output(output3)
            .output_data(Default::default())
            .build()
    };

    let rtx = context.resolve(transaction);

    let verifier = Verifier::default();
    let result = verifier.verify_without_limit(&rtx);

    assert!(result.is_ok());
}
