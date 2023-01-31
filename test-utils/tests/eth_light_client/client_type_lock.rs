use std::path::Path;

use ibc_ckb_contracts_test_utils::{
    ckb::{
        script::ScriptVersion,
        types::{
            core::{ScriptHashType, TransactionBuilder},
            packed,
            prelude::*,
        },
    },
    misc, Context, Verifier,
};

use super::{CLIENT_TYPE_LOCK_CONTRACT, DATA_DIR};
use crate::{mock_contracts::REVERSE_ARGS_LOCK_CONTRACT, prelude::*};

#[test]
fn create_case_1_no_empty() {
    let param = CreateParameter {
        case_id: 1,
        client_filename: "client-5246088_5246151.data",
        proof_update_filename: "proof_update-5246088_5246151.data",
    };
    create(param);
}

#[test]
#[should_panic]
fn create_case_2_empty_at_the_start_of_updates() {
    let param = CreateParameter {
        case_id: 2,
        client_filename: "client-5601248_5601264.data",
        proof_update_filename: "proof_update-5601248_5601264.data",
    };
    create(param);
}

#[test]
fn create_case_2_empty_at_the_middle_of_updates() {
    let param = CreateParameter {
        case_id: 2,
        client_filename: "client-5601201_5601264.data",
        proof_update_filename: "proof_update-5601201_5601264.data",
    };
    create(param);
}

#[test]
fn create_case_2_empty_at_the_end_of_updates() {
    let param = CreateParameter {
        case_id: 2,
        client_filename: "client-5601201_5601248.data",
        proof_update_filename: "proof_update-5601201_5601248.data",
    };
    create(param);
}

#[test]
fn update_case_1_no_empty() {
    let param = UpdateParameter {
        case_id: 1,
        client_filename: "client-5246088_5246119.data",
        new_client_filename: "client-5246088_5246151.data",
        proof_update_filename: "proof_update-5246120_5246151.data",
    };
    update(param);
}

#[test]
fn update_case_2_empty_client() {
    let param = UpdateParameter {
        case_id: 2,
        client_filename: "client-5601201_5601248.data",
        new_client_filename: "client-5601201_5601264.data",
        proof_update_filename: "proof_update-5601249_5601264.data",
    };
    update(param);
}

#[test]
fn update_case_2_empty_at_the_start_of_updates() {
    let param = UpdateParameter {
        case_id: 2,
        client_filename: "client-5601201_5601247.data",
        new_client_filename: "client-5601201_5601264.data",
        proof_update_filename: "proof_update-5601248_5601264.data",
    };
    update(param);
}

#[test]
fn update_case_2_empty_at_the_middle_of_updates() {
    let param = UpdateParameter {
        case_id: 2,
        client_filename: "client-5601201_5601232.data",
        new_client_filename: "client-5601201_5601264.data",
        proof_update_filename: "proof_update-5601233_5601264.data",
    };
    update(param);
}

#[test]
fn update_case_2_empty_at_the_end_of_updates() {
    let param = UpdateParameter {
        case_id: 2,
        client_filename: "client-5601201_5601224.data",
        new_client_filename: "client-5601201_5601248.data",
        proof_update_filename: "proof_update-5601225_5601248.data",
    };
    update(param);
}

struct CreateParameter {
    case_id: usize,
    client_filename: &'static str,
    proof_update_filename: &'static str,
}

fn create(param: CreateParameter) {
    crate::setup();

    let client_id = misc::random_hash().raw_data().to_vec();

    let case_dir = format!("case-{}", param.case_id);
    let root_dir = Path::new(DATA_DIR).join("client_type_lock").join(case_dir);
    let client = misc::load_data_from_file(&root_dir, param.client_filename);
    let proof_update = misc::load_data_from_file(&root_dir, param.proof_update_filename);

    let mut context = Context::new();
    let script_version = ScriptVersion::latest();

    let deployed_lock_contract = {
        let contract_data = misc::load_contract_from_file(REVERSE_ARGS_LOCK_CONTRACT);
        let data = contract_data.into();
        let lock_script = packed::Script::default();
        context.deploy(data, lock_script, None)
    };

    let deployed_type_contract = {
        let contract_data = misc::load_contract_from_file(CLIENT_TYPE_LOCK_CONTRACT);
        let data = contract_data.into();
        let lock_script = packed::Script::default();
        let type_script = packed::Script::new_builder().args(vec![0u8].pack()).build();
        context.deploy(data, lock_script, Some(type_script))
    };

    let deployed_cell = {
        let data = Default::default();
        let args = misc::random_bytes();
        let lock_script = packed::Script::new_builder()
            .hash_type(script_version.data_hash_type().into())
            .code_hash(deployed_lock_contract.data_hash())
            .args(args.pack())
            .build();
        context.deploy(data, lock_script, None)
    };

    let transaction = {
        let output = {
            let type_script = packed::Script::new_builder()
                .hash_type(ScriptHashType::Type.into())
                .code_hash(deployed_type_contract.type_hash().unwrap())
                .args(client_id.pack())
                .build();
            deployed_cell
                .cell_output()
                .as_builder()
                .type_(Some(type_script).pack())
                .build()
        };
        let witness = {
            let input_type_args = packed::BytesOpt::new_builder()
                .set(Some(proof_update.pack()))
                .build();
            let witness_args = packed::WitnessArgs::new_builder()
                .input_type(input_type_args)
                .build();
            witness_args.as_bytes()
        };
        TransactionBuilder::default()
            .cell_dep(deployed_lock_contract.as_cell_dep())
            .cell_dep(deployed_type_contract.as_cell_dep())
            .input(deployed_cell.as_input())
            .output(output)
            .output_data(client.pack())
            .witness(witness.pack())
            .build()
    };

    let rtx = context.resolve(transaction);

    let verifier = Verifier::default();
    let result = verifier.verify_without_limit(&rtx);

    result.should_be_ok();
}

struct UpdateParameter {
    case_id: usize,
    client_filename: &'static str,
    new_client_filename: &'static str,
    proof_update_filename: &'static str,
}

fn update(param: UpdateParameter) {
    crate::setup();

    let client_id = misc::random_hash().raw_data().to_vec();

    let case_dir = format!("case-{}", param.case_id);
    let root_dir = Path::new(DATA_DIR).join("client_type_lock").join(case_dir);
    let client = misc::load_data_from_file(&root_dir, param.client_filename);
    let new_client = misc::load_data_from_file(&root_dir, param.new_client_filename);
    let proof_update = misc::load_data_from_file(&root_dir, param.proof_update_filename);

    let mut context = Context::new();
    let script_version = ScriptVersion::latest();

    let deployed_lock_contract = {
        let contract_data = misc::load_contract_from_file(REVERSE_ARGS_LOCK_CONTRACT);
        let data = contract_data.into();
        let lock_script = packed::Script::default();
        context.deploy(data, lock_script, None)
    };

    let deployed_type_contract = {
        let contract_data = misc::load_contract_from_file(CLIENT_TYPE_LOCK_CONTRACT);
        let data = contract_data.into();
        let lock_script = packed::Script::default();
        let type_script = packed::Script::new_builder().args(vec![0u8].pack()).build();
        context.deploy(data, lock_script, Some(type_script))
    };

    let deployed_cell = {
        let data = client.into();
        let args = misc::random_bytes();
        let lock_script = packed::Script::new_builder()
            .hash_type(script_version.data_hash_type().into())
            .code_hash(deployed_lock_contract.data_hash())
            .args(args.pack())
            .build();
        let type_script = packed::Script::new_builder()
            .hash_type(ScriptHashType::Type.into())
            .code_hash(deployed_type_contract.type_hash().unwrap())
            .args(client_id.pack())
            .build();
        context.deploy(data, lock_script, Some(type_script))
    };

    let transaction = {
        let output = deployed_cell.cell_output();
        let witness = {
            let input_type_args = packed::BytesOpt::new_builder()
                .set(Some(proof_update.pack()))
                .build();
            let witness_args = packed::WitnessArgs::new_builder()
                .input_type(input_type_args)
                .build();
            witness_args.as_bytes()
        };
        TransactionBuilder::default()
            .cell_dep(deployed_lock_contract.as_cell_dep())
            .cell_dep(deployed_type_contract.as_cell_dep())
            .input(deployed_cell.as_input())
            .output(output)
            .output_data(new_client.pack())
            .witness(witness.pack())
            .build()
    };

    let rtx = context.resolve(transaction);

    let verifier = Verifier::default();
    let result = verifier.verify_without_limit(&rtx);

    result.should_be_ok();
}
