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
fn mainnet_testcase_in_capella() {
    let bootstrap_slot = 6632736;
    let finalized_slots = vec![6632768, 6632800, 6632832, 6632864, 6632896, 6632928];
    mainnet_testcase_for_slots(bootstrap_slot, finalized_slots);
}

fn mainnet_testcase_for_slots(bootstrap_slot: u64, finalized_slots: Vec<u64>) {
    let client_update_and_new_client_filenames = finalized_slots
        .into_iter()
        .map(|slot| {
            (
                format!("client_update-{slot:09}.data"),
                format!("client-{bootstrap_slot:09}_{slot:09}.data"),
            )
        })
        .collect();
    let param = Parameter {
        clients_count: 3,
        minimal_headers_count: 22,
        client_filename: format!("client-{bootstrap_slot:09}_{bootstrap_slot:09}.data"),
        sync_committee_filename: format!("sync_committee-{bootstrap_slot:09}.data"),
        client_update_and_new_client_filenames,
    };
    update_client(param);
}

struct Parameter {
    clients_count: u8,
    minimal_headers_count: u8,
    client_filename: String,
    sync_committee_filename: String,
    client_update_and_new_client_filenames: Vec<(String, String)>,
}

fn update_client(param: Parameter) {
    crate::setup();

    let bootstrap_dir = Path::new(DATA_DIR)
        .join("client_type_lock")
        .join("bootstrap");
    let client = misc::load_data_from_file(&bootstrap_dir, &param.client_filename);
    let sync_committee = misc::load_data_from_file(&bootstrap_dir, &param.sync_committee_filename);

    let update_dir = Path::new(DATA_DIR)
        .join("client_type_lock")
        .join("client_update");

    let mut last_client_id = 0;
    let mut clients = (0..param.clients_count)
        .into_iter()
        .map(|id| {
            let mut client_copied = client.clone();
            client_copied[0] = id;
            client_copied
        })
        .collect::<Vec<_>>();

    for (client_update_filename, new_client_filename) in
        &param.client_update_and_new_client_filenames
    {
        let mut context = Context::new();
        let script_version = ScriptVersion::latest();

        let client_update = misc::load_data_from_file(&update_dir, client_update_filename);
        let new_client = misc::load_data_from_file(&update_dir, new_client_filename);

        let next_client_id = if last_client_id + 1 < param.clients_count {
            last_client_id + 1
        } else {
            0
        };

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

        let input_client_info = {
            let client_info = utils::build_client_info(last_client_id, param.minimal_headers_count);
            let data = client_info.unpack();
            context.deploy(data, lock_script.clone(), Some(type_script.clone()), None)
        };

        let input_client = {
            let data = clients[usize::from(next_client_id)].clone().into();
            context.deploy(data, lock_script.clone(), Some(type_script.clone()), None)
        };

        let cell_dep_client = {
            let data = clients[usize::from(last_client_id)].clone().into();
            context.deploy(data, lock_script.clone(), Some(type_script.clone()), None)
        };

        let cell_dep_sync_committee = {
            let data = sync_committee.clone().into();
            context.deploy(data, lock_script, Some(type_script), None)
        };

        let transaction = {
            let output_client = input_client.cell_output();
            let output_client_info = input_client_info.cell_output();
            let output_client_info_data =
                utils::build_client_info(next_client_id, param.minimal_headers_count);
            let witness = {
                let input_type_args = packed::BytesOpt::new_builder()
                    .set(Some(client_update.pack()))
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
                .cell_dep(cell_dep_sync_committee.as_cell_dep())
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

        clients[usize::from(next_client_id)] = new_client;
        last_client_id = next_client_id;
    }
}
