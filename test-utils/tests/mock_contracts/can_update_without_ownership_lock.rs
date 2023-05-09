use ibc_ckb_contracts_test_utils::{
    ckb::{
        script::ScriptVersion,
        types::{
            core::{Capacity, TransactionBuilder},
            packed,
            prelude::*,
        },
    },
    misc, Context, Verifier,
};

use super::CAN_UPDATE_WITHOUT_OWNERSHIP_LOCK_CONTRACT;
use crate::prelude::*;

#[test]
fn lost_capacity_with_1_input() {
    crate::setup();

    let args_bytes = misc::randomize_bytes();
    let args: packed::Bytes = args_bytes.pack();
    let args_rev: packed::Bytes = args_bytes.into_iter().rev().collect::<Vec<_>>().pack();

    let mut context = Context::new();
    let script_version = ScriptVersion::latest();

    let deployed_contract = {
        let contract_data =
            misc::load_contract_from_file(CAN_UPDATE_WITHOUT_OWNERSHIP_LOCK_CONTRACT);
        let data = contract_data.into();
        let lock_script = packed::Script::default();
        context.deploy(data, lock_script, None, None)
    };

    let lock_script = packed::Script::new_builder()
        .hash_type(script_version.data_hash_type().into())
        .code_hash(deployed_contract.data_hash())
        .args(args.clone())
        .build();

    let deployed_cell = context.deploy(
        Default::default(),
        lock_script.clone(),
        None,
        Some(Capacity::shannons(1)),
    );

    let output = packed::CellOutput::new_builder()
        .lock(lock_script)
        .build_exact_capacity(Capacity::zero())
        .unwrap();

    let witness = {
        let lock_args = packed::BytesOpt::new_builder()
            .set(Some(args_rev.clone()))
            .build();
        let witness_args = packed::WitnessArgs::new_builder().lock(lock_args).build();
        witness_args.as_bytes()
    }
    .pack();

    let witness_empty: packed::Bytes = Default::default();

    let witness_no_args = {
        let lock_args = packed::BytesOpt::new_builder().set(None).build();
        let witness_args = packed::WitnessArgs::new_builder().lock(lock_args).build();
        witness_args.as_bytes()
    }
    .pack();

    let witness_bad_args = {
        let mut args_bad: packed::Bytes = misc::randomize_bytes().pack();
        while args_rev.as_slice() == args_bad.as_slice() {
            args_bad = misc::randomize_bytes().pack();
        }

        let lock_args = packed::BytesOpt::new_builder().set(Some(args_bad)).build();
        let witness_args = packed::WitnessArgs::new_builder().lock(lock_args).build();
        witness_args.as_bytes()
    }
    .pack();

    let verifier = Verifier::default();

    let tx_without_witness = TransactionBuilder::default()
        .cell_dep(deployed_contract.as_cell_dep())
        .input(deployed_cell.as_input())
        .output(output)
        .output_data(Default::default())
        .build();

    let tx = tx_without_witness
        .clone()
        .as_advanced_builder()
        .witness(witness)
        .build();
    let rtx = context.resolve(tx);
    verifier.verify_without_limit(&rtx).should_be_ok();

    let tx = tx_without_witness
        .clone()
        .as_advanced_builder()
        .witness(witness_empty)
        .build();
    let rtx = context.resolve(tx);
    verifier.verify_without_limit(&rtx).should_be_err();

    let tx = tx_without_witness
        .clone()
        .as_advanced_builder()
        .witness(witness_no_args)
        .build();
    let rtx = context.resolve(tx);
    verifier.verify_without_limit(&rtx).should_be_err();

    let tx = tx_without_witness
        .clone()
        .as_advanced_builder()
        .witness(witness_bad_args)
        .build();
    let rtx = context.resolve(tx);
    verifier.verify_without_limit(&rtx).should_be_err();

    let rtx = context.resolve(tx_without_witness);
    verifier.verify_without_limit(&rtx).should_be_err();
}

#[test]
fn lost_capacity_with_2_inputs() {
    crate::setup();

    let args_bytes = misc::randomize_bytes();
    let args: packed::Bytes = args_bytes.pack();
    let args_rev: packed::Bytes = args_bytes.into_iter().rev().collect::<Vec<_>>().pack();

    let mut context = Context::new();
    let script_version = ScriptVersion::latest();

    let deployed_contract = {
        let contract_data =
            misc::load_contract_from_file(CAN_UPDATE_WITHOUT_OWNERSHIP_LOCK_CONTRACT);
        let data = contract_data.into();
        let lock_script = packed::Script::default();
        context.deploy(data, lock_script, None, None)
    };

    let lock_script = packed::Script::new_builder()
        .hash_type(script_version.data_hash_type().into())
        .code_hash(deployed_contract.data_hash())
        .args(args.clone())
        .build();

    let deployed_cells = (0..2)
        .into_iter()
        .map(|_| context.deploy(Default::default(), lock_script.clone(), None, None))
        .collect::<Vec<_>>();

    let output = packed::CellOutput::new_builder()
        .lock(lock_script)
        .build_exact_capacity(Capacity::zero())
        .unwrap();

    let witness = {
        let lock_args = packed::BytesOpt::new_builder()
            .set(Some(args_rev.clone()))
            .build();
        let witness_args = packed::WitnessArgs::new_builder().lock(lock_args).build();
        witness_args.as_bytes()
    }
    .pack();

    let witness_empty: packed::Bytes = Default::default();

    let witness_no_args = {
        let lock_args = packed::BytesOpt::new_builder().set(None).build();
        let witness_args = packed::WitnessArgs::new_builder().lock(lock_args).build();
        witness_args.as_bytes()
    }
    .pack();

    let witness_bad_args = {
        let mut args_bad: packed::Bytes = misc::randomize_bytes().pack();
        while args_rev.as_slice() == args_bad.as_slice() {
            args_bad = misc::randomize_bytes().pack();
        }

        let lock_args = packed::BytesOpt::new_builder().set(Some(args_bad)).build();
        let witness_args = packed::WitnessArgs::new_builder().lock(lock_args).build();
        witness_args.as_bytes()
    }
    .pack();

    let verifier = Verifier::default();

    let tx_without_witness = TransactionBuilder::default()
        .cell_dep(deployed_contract.as_cell_dep())
        .input(deployed_cells[0].as_input())
        .input(deployed_cells[1].as_input())
        .output(output)
        .output_data(Default::default())
        .build();

    for x in 0..4 {
        for y in 0..4 {
            let mut tx_builder = tx_without_witness.clone().as_advanced_builder();
            let mut flag = None;
            for i in &[x, y] {
                match *i {
                    0 => {
                        tx_builder = tx_builder.witness(witness.clone());
                        if flag.is_none() {
                            flag = Some(true);
                        }
                    }
                    1 => {
                        tx_builder = tx_builder.witness(witness_empty.clone());
                    }
                    2 => {
                        tx_builder = tx_builder.witness(witness_no_args.clone());
                    }
                    _ => {
                        tx_builder = tx_builder.witness(witness_bad_args.clone());
                        flag = Some(false);
                    }
                };
            }
            let tx = tx_builder.build();
            let rtx = context.resolve(tx);
            if flag.unwrap_or(false) {
                verifier.verify_without_limit(&rtx).should_be_ok();
            } else {
                verifier.verify_without_limit(&rtx).should_be_err();
            }
        }
    }

    let tx = tx_without_witness
        .clone()
        .as_advanced_builder()
        .witness(witness)
        .build();
    let rtx = context.resolve(tx);
    verifier.verify_without_limit(&rtx).should_be_ok();

    let tx = tx_without_witness
        .clone()
        .as_advanced_builder()
        .witness(witness_empty)
        .build();
    let rtx = context.resolve(tx);
    verifier.verify_without_limit(&rtx).should_be_err();

    let tx = tx_without_witness
        .clone()
        .as_advanced_builder()
        .witness(witness_no_args)
        .build();
    let rtx = context.resolve(tx);
    verifier.verify_without_limit(&rtx).should_be_err();

    let tx = tx_without_witness
        .clone()
        .as_advanced_builder()
        .witness(witness_bad_args)
        .build();
    let rtx = context.resolve(tx);
    verifier.verify_without_limit(&rtx).should_be_err();

    let rtx = context.resolve(tx_without_witness);
    verifier.verify_without_limit(&rtx).should_be_err();
}

#[test]
fn add_capacity_without_ownership() {
    crate::setup();

    let args_bytes_extra = misc::randomize_bytes();
    let mut args_bytes = misc::randomize_bytes();
    while args_bytes_extra == args_bytes {
        args_bytes = misc::randomize_bytes();
    }

    let args_extra: packed::Bytes = args_bytes_extra.pack();
    let args_rev_extra: packed::Bytes = args_bytes_extra
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .pack();

    let args: packed::Bytes = args_bytes.pack();
    let args_rev: packed::Bytes = args_bytes.into_iter().rev().collect::<Vec<_>>().pack();

    let mut context = Context::new();
    let script_version = ScriptVersion::latest();

    let deployed_contract = {
        let contract_data =
            misc::load_contract_from_file(CAN_UPDATE_WITHOUT_OWNERSHIP_LOCK_CONTRACT);
        let data = contract_data.into();
        let lock_script = packed::Script::default();
        context.deploy(data, lock_script, None, None)
    };

    let lock_script_extra = packed::Script::new_builder()
        .hash_type(script_version.data_hash_type().into())
        .code_hash(deployed_contract.data_hash())
        .args(args_extra.clone())
        .build();

    let lock_script = packed::Script::new_builder()
        .hash_type(script_version.data_hash_type().into())
        .code_hash(deployed_contract.data_hash())
        .args(args.clone())
        .build();

    let deployed_cell_extra =
        context.deploy(Default::default(), lock_script_extra.clone(), None, None);
    let deployed_cell = context.deploy(Default::default(), lock_script.clone(), None, None);

    let output = packed::CellOutput::new_builder()
        .lock(lock_script)
        .build_exact_capacity(Capacity::shannons(1))
        .unwrap();

    let witness_extra = {
        let lock_args = packed::BytesOpt::new_builder()
            .set(Some(args_rev_extra.clone()))
            .build();
        let witness_args = packed::WitnessArgs::new_builder().lock(lock_args).build();
        witness_args.as_bytes()
    }
    .pack();

    let witness = {
        let lock_args = packed::BytesOpt::new_builder()
            .set(Some(args_rev.clone()))
            .build();
        let witness_args = packed::WitnessArgs::new_builder().lock(lock_args).build();
        witness_args.as_bytes()
    }
    .pack();

    let witness_empty: packed::Bytes = Default::default();

    let witness_no_args = {
        let lock_args = packed::BytesOpt::new_builder().set(None).build();
        let witness_args = packed::WitnessArgs::new_builder().lock(lock_args).build();
        witness_args.as_bytes()
    }
    .pack();

    let witness_bad_args = {
        let mut args_bad: packed::Bytes = misc::randomize_bytes().pack();
        while args_rev_extra.as_slice() == args_bad.as_slice()
            || args_rev.as_slice() == args_bad.as_slice()
        {
            args_bad = misc::randomize_bytes().pack();
        }

        let lock_args = packed::BytesOpt::new_builder().set(Some(args_bad)).build();
        let witness_args = packed::WitnessArgs::new_builder().lock(lock_args).build();
        witness_args.as_bytes()
    }
    .pack();

    let verifier = Verifier::default();

    let tx_without_witness = TransactionBuilder::default()
        .cell_dep(deployed_contract.as_cell_dep())
        .input(deployed_cell_extra.as_input())
        .input(deployed_cell.as_input())
        .output(output)
        .output_data(Default::default())
        .witness(witness_extra)
        .build();

    let tx = tx_without_witness
        .clone()
        .as_advanced_builder()
        .witness(witness)
        .build();
    let rtx = context.resolve(tx);
    verifier.verify_without_limit(&rtx).should_be_ok();

    let tx = tx_without_witness
        .clone()
        .as_advanced_builder()
        .witness(witness_empty)
        .build();
    let rtx = context.resolve(tx);
    verifier.verify_without_limit(&rtx).should_be_ok();

    let tx = tx_without_witness
        .clone()
        .as_advanced_builder()
        .witness(witness_no_args)
        .build();
    let rtx = context.resolve(tx);
    verifier.verify_without_limit(&rtx).should_be_ok();

    let tx = tx_without_witness
        .clone()
        .as_advanced_builder()
        .witness(witness_bad_args)
        .build();
    let rtx = context.resolve(tx);
    verifier.verify_without_limit(&rtx).should_be_err();

    let rtx = context.resolve(tx_without_witness);
    verifier.verify_without_limit(&rtx).should_be_ok();
}
