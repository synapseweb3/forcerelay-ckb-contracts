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

const SLOTS_IN_ONE_PERIOD: u64 = 8192;

#[test]
fn mainnet_testcase_altair_to_bellatrix() {
    mainnet_testcase_since_slot(4612096, 6);
}

#[test]
fn mainnet_testcase_bellatrix_to_capella() {
    mainnet_testcase_since_slot(6184960, 6);
}

fn mainnet_testcase_since_slot(bootstrap_slot: u64, count: usize) {
    let sync_committee_update_and_new_sync_committee_filenames = (1..=count)
        .into_iter()
        .map(|index| {
            (
                format!("sync_committee_update-{bootstrap_slot:09}_{index:02}.data"),
                format!("sync_committee-{bootstrap_slot:09}_{index:02}.data"),
            )
        })
        .collect();
    let param = Parameter {
        clients_count: 3,
        minimal_headers_count: 22,
        client_filename: format!("client-{bootstrap_slot:09}_{bootstrap_slot:09}.data"),
        sync_committee_filename: format!("sync_committee-{bootstrap_slot:09}.data"),
        sync_committee_update_and_new_sync_committee_filenames,
    };
    update_sync_committee(param);
}

struct Parameter {
    clients_count: u8,
    minimal_headers_count: u8,
    client_filename: String,
    sync_committee_filename: String,
    sync_committee_update_and_new_sync_committee_filenames: Vec<(String, String)>,
}

fn update_sync_committee(param: Parameter) {
    crate::setup();

    let bootstrap_dir = Path::new(DATA_DIR)
        .join("client_type_lock")
        .join("bootstrap");
    let mut client = misc::load_data_from_file(&bootstrap_dir, &param.client_filename);
    let sync_committee = misc::load_data_from_file(&bootstrap_dir, &param.sync_committee_filename);

    let update_dir = Path::new(DATA_DIR)
        .join("client_type_lock")
        .join("sync_committee_update");

    let mut first_is_latest = true;
    let mut sync_committees = vec![sync_committee; 2];
    let mut maximal_slot = {
        let mut tmp = [0u8; 8];
        tmp.copy_from_slice(&client[9..17]);
        u64::from_le_bytes(tmp)
    };

    for (sync_committee_update_filename, new_sync_committee_filename) in
        &param.sync_committee_update_and_new_sync_committee_filenames
    {
        let mut context = Context::new();
        let script_version = ScriptVersion::latest();

        let sync_committee_update =
            misc::load_data_from_file(&update_dir, sync_committee_update_filename);
        let new_sync_committee =
            misc::load_data_from_file(&update_dir, new_sync_committee_filename);

        let update_index = if first_is_latest { 1 } else { 0 };
        let latest_index = if first_is_latest { 0 } else { 1 };

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

        let client_type_args = utils::randomize_client_type_args(param.clients_count);
        let type_script = packed::Script::new_builder()
            .hash_type(ScriptHashType::Type.into())
            .code_hash(deployed_type_contract.type_hash().unwrap())
            .args(client_type_args)
            .build();

        let cell_dep_client_info = {
            let client_info = utils::build_client_info(0, param.minimal_headers_count);
            let data = client_info.unpack();
            context.deploy(data, lock_script.clone(), Some(type_script.clone()), None)
        };

        let cell_dep_client = {
            let tmp = maximal_slot.to_le_bytes();
            client[9..17].copy_from_slice(&tmp);
            let data = client.clone().into();
            context.deploy(data, lock_script.clone(), Some(type_script.clone()), None)
        };

        let cell_dep_sync_committee = {
            let data = sync_committees[latest_index].clone().into();
            context.deploy(data, lock_script.clone(), Some(type_script.clone()), None)
        };

        let input_sync_committee = {
            let data = sync_committees[update_index].clone().into();
            context.deploy(data, lock_script, Some(type_script), None)
        };

        let transaction = {
            let output_sync_committee = input_sync_committee.cell_output();
            let witness = {
                let input_type_args = packed::BytesOpt::new_builder()
                    .set(Some(sync_committee_update.pack()))
                    .build();
                let witness_args = packed::WitnessArgs::new_builder()
                    .input_type(input_type_args)
                    .build();
                witness_args.as_bytes()
            };
            TransactionBuilder::default()
                .cell_dep(deployed_lock_contract.as_cell_dep())
                .cell_dep(deployed_type_contract.as_cell_dep())
                .cell_dep(cell_dep_client_info.as_cell_dep())
                .cell_dep(cell_dep_client.as_cell_dep())
                .cell_dep(cell_dep_sync_committee.as_cell_dep())
                .input(input_sync_committee.as_input())
                .output(output_sync_committee)
                .output_data(new_sync_committee.pack())
                .witness(witness.pack())
                .build()
        };

        let rtx = context.resolve(transaction);

        let verifier = Verifier::default();
        let result = verifier.verify_without_limit(&rtx);

        result.should_be_ok();

        maximal_slot += SLOTS_IN_ONE_PERIOD;
        sync_committees[update_index] = new_sync_committee;
        first_is_latest = !first_is_latest;
    }
}
