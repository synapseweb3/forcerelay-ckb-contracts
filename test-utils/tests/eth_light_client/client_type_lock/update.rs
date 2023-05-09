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

use super::super::{utils, CLIENT_TYPE_LOCK_CONTRACT, DATA_DIR};
use crate::{mock_contracts::CAN_UPDATE_WITHOUT_OWNERSHIP_LOCK_CONTRACT, prelude::*};

#[test]
fn update_case_1() {
    let param = UpdateParameter {
        case_id: 1,
        clients_count: 3,
        minimal_updates_count: 32,
        cell_dep_id: 2,
        input_id: 0,
        client_filename: "client-6268480_6268511.data",
        new_client_filename: "client-6268480_6268543.data",
        proof_update_filename: "proof_update-6268512_6268543.data",
    };
    update(param);
}

struct UpdateParameter {
    case_id: usize,
    clients_count: u8,
    minimal_updates_count: u8,
    cell_dep_id: u8,
    input_id: u8,
    client_filename: &'static str,
    new_client_filename: &'static str,
    proof_update_filename: &'static str,
}

fn update(param: UpdateParameter) {
    crate::setup();

    let case_dir = format!("case-{}", param.case_id);
    let root_dir = Path::new(DATA_DIR).join("client_type_lock").join(case_dir);
    let mut client = misc::load_data_from_file(&root_dir, param.client_filename);
    let mut new_client = misc::load_data_from_file(&root_dir, param.new_client_filename);
    let proof_update = misc::load_data_from_file(&root_dir, param.proof_update_filename);

    let mut context = Context::new();
    let script_version = ScriptVersion::latest();

    let deployed_lock_contract = {
        let contract_data =
            misc::load_contract_from_file(CAN_UPDATE_WITHOUT_OWNERSHIP_LOCK_CONTRACT);
        let data = contract_data.into();
        let lock_script = packed::Script::default();
        context.deploy(data, lock_script, None, None)
    };

    let deployed_type_contract = {
        let contract_data = misc::load_contract_from_file(CLIENT_TYPE_LOCK_CONTRACT);
        let data = contract_data.into();
        let lock_script = packed::Script::default();
        let type_script = packed::Script::new_builder().args(vec![0u8].pack()).build();
        context.deploy(data, lock_script, Some(type_script), None)
    };

    let lock_args = misc::randomize_bytes();
    let lock_script = packed::Script::new_builder()
        .hash_type(script_version.data_hash_type().into())
        .code_hash(deployed_lock_contract.data_hash())
        .args(lock_args.pack())
        .build();

    let client_type_args = {
        let cells_count = param.clients_count + 1;
        utils::randomize_client_type_args(cells_count)
    };
    let type_script = packed::Script::new_builder()
        .hash_type(ScriptHashType::Type.into())
        .code_hash(deployed_type_contract.type_hash().unwrap())
        .args(client_type_args)
        .build();

    let input_client = {
        client[0] = param.input_id;
        let data = client.clone().into();
        context.deploy(data, lock_script.clone(), Some(type_script.clone()), None)
    };

    let cell_dep_client = {
        client[0] = param.cell_dep_id;
        let data = client.clone().into();
        context.deploy(data, lock_script.clone(), Some(type_script.clone()), None)
    };

    let input_client_info = {
        let client_info = utils::build_client_info(param.cell_dep_id, param.minimal_updates_count);
        context.deploy(client_info.unpack(), lock_script, Some(type_script), None)
    };

    let transaction = {
        new_client[0] = param.input_id;
        let output_client = input_client.cell_output();
        let output_client_info = input_client_info.cell_output();
        let output_client_info_data =
            utils::build_client_info(param.input_id, param.minimal_updates_count);
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
            .cell_dep(cell_dep_client.as_cell_dep())
            .input(input_client.as_input())
            .input(input_client_info.as_input())
            .output(output_client)
            .output_data(new_client.pack())
            .output(output_client_info)
            .output_data(output_client_info_data)
            .witness(witness.pack())
            .build()
    };

    let rtx = context.resolve(transaction);

    let verifier = Verifier::default();
    let result = verifier.verify_without_limit(&rtx);

    result.should_be_ok();
}
