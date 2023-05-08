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
fn create_case_1() {
    let param = CreateParameter {
        case_id: 1,
        clients_count: 3,
        minimal_updates_count: 64,
        client_filename: "client-6268480_6268543.data",
        proof_update_filename: "proof_update-6268480_6268543.data",
    };
    create(param);
}

struct CreateParameter {
    case_id: usize,
    clients_count: u8,
    minimal_updates_count: u8,
    client_filename: &'static str,
    proof_update_filename: &'static str,
}

fn create(param: CreateParameter) {
    crate::setup();

    let case_dir = format!("case-{}", param.case_id);
    let root_dir = Path::new(DATA_DIR).join("client_type_lock").join(case_dir);
    let mut client = misc::load_data_from_file(&root_dir, param.client_filename);
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

    let deployed_cell = {
        let data = Default::default();
        let args = misc::randomize_bytes();
        let lock_script = packed::Script::new_builder()
            .hash_type(script_version.data_hash_type().into())
            .code_hash(deployed_lock_contract.data_hash())
            .args(args.pack())
            .build();
        context.deploy(data, lock_script, None, None)
    };

    let transaction = {
        let client_type_args = {
            let cells_count = param.clients_count + 1;
            utils::build_client_type_args(&deployed_cell.as_input(), cells_count)
        };
        let client_info = utils::build_client_info(0, param.minimal_updates_count);
        let output = {
            let type_script = packed::Script::new_builder()
                .hash_type(ScriptHashType::Type.into())
                .code_hash(deployed_type_contract.type_hash().unwrap())
                .args(client_type_args)
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
        let mut tx_builder = TransactionBuilder::default()
            .cell_dep(deployed_lock_contract.as_cell_dep())
            .cell_dep(deployed_type_contract.as_cell_dep())
            .input(deployed_cell.as_input())
            .witness(witness.pack());
        for id in 0..param.clients_count {
            client[0] = id;
            tx_builder = tx_builder.output(output.clone()).output_data(client.pack());
        }
        tx_builder = tx_builder.output(output).output_data(client_info);
        tx_builder.build()
    };

    let rtx = context.resolve(transaction);

    let verifier = Verifier::default();
    let result = verifier.verify_without_limit(&rtx);

    result.should_be_ok();
}
