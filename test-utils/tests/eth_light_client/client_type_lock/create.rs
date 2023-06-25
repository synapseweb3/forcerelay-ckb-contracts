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
fn mainnet_testcase_for_altair() {
    for slot in [3745600, 4628480, 4636640] {
        mainnet_testcase_for_slot(slot);
    }
}

#[test]
fn mainnet_testcase_for_bellatrix() {
    for slot in [4636672, 4636704, 4644864, 6201343, 6209504] {
        mainnet_testcase_for_slot(slot);
    }
}

#[test]
fn mainnet_testcase_for_capella() {
    for slot in [6209535, 6209568, 6217728] {
        mainnet_testcase_for_slot(slot);
    }
}

#[test]
fn mainnet_testcase_for_others() {
    // The test data for client update.
    for slot in [6632736] {
        mainnet_testcase_for_slot(slot);
    }
    // The test data for sync committee update.
    for slot in [4612096, 6184960] {
        mainnet_testcase_for_slot(slot);
    }
}

fn mainnet_testcase_for_slot(slot: u64) {
    let param = Parameter {
        clients_count: 3,
        minimal_headers_count: 22,
        client_filename: format!("client-{slot:09}_{slot:09}.data"),
        sync_committee_filename: format!("sync_committee-{slot:09}.data"),
        client_bootstrap_filename: format!("client_bootstrap-{slot:09}.data"),
    };
    create(param);
}

struct Parameter {
    clients_count: u8,
    minimal_headers_count: u8,
    client_filename: String,
    sync_committee_filename: String,
    client_bootstrap_filename: String,
}

fn create(param: Parameter) {
    crate::setup();

    let bootstrap_dir = Path::new(DATA_DIR)
        .join("client_type_lock")
        .join("bootstrap");
    let mut client = misc::load_data_from_file(&bootstrap_dir, &param.client_filename);
    let sync_committee = misc::load_data_from_file(&bootstrap_dir, &param.sync_committee_filename);
    let client_bootstrap =
        misc::load_data_from_file(&bootstrap_dir, &param.client_bootstrap_filename);

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
        let client_type_args =
            utils::build_client_type_args(&deployed_cell.as_input(), param.clients_count);
        let client_info = utils::build_client_info(0, param.minimal_headers_count);
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
                .set(Some(client_bootstrap.pack()))
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
            .output(output.clone())
            .output_data(client_info)
            .witness(witness.pack());
        for id in 0..param.clients_count {
            client[0] = id;
            tx_builder = tx_builder.output(output.clone()).output_data(client.pack());
        }
        tx_builder
            .output(output.clone())
            .output(output)
            .output_data(sync_committee.pack())
            .output_data(sync_committee.pack())
            .build()
    };

    let rtx = context.resolve(transaction);

    let verifier = Verifier::default();
    let result = verifier.verify_without_limit(&rtx);

    result.should_be_ok();
}
