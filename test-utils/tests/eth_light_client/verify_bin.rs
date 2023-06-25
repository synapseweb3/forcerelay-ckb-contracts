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

use super::{
    utils, CLIENT_TYPE_LOCK_CONTRACT, DATA_DIR, MOCK_BUSINESS_TYPE_LOCK_CONTRACT,
    VERIFY_BIN_CONTRACT,
};
use crate::{mock_contracts::CAN_UPDATE_WITHOUT_OWNERSHIP_LOCK_CONTRACT, prelude::*};

#[test]
fn mainnet_testcase_in_capella() {
    let bootstrap_slot = 6632736;
    let finalized_slots = vec![6632864, 6632896, 6632928];
    mainnet_testcase_for_transaction(bootstrap_slot, finalized_slots, 6632854, 42);
}

fn mainnet_testcase_for_transaction(
    bootstrap_slot: u64,
    finalized_slots: Vec<u64>,
    tx_in_slot: u64,
    tx_index: usize,
) {
    let client_and_tx_proof_filenames = finalized_slots
        .into_iter()
        .map(|slot| {
            (
                format!("client-{bootstrap_slot:09}_{slot:09}.data"),
                format!("tx_proof-{tx_in_slot:09}_{tx_index:03}-{slot:09}.data"),
            )
        })
        .collect();
    let param = Parameter {
        client_and_tx_proof_filenames,
        tx_payload_filename: format!("tx_payload-{tx_in_slot:09}_{tx_index:03}.data"),
    };
    verify(param);
}

struct Parameter {
    client_and_tx_proof_filenames: Vec<(String, String)>,
    tx_payload_filename: String,
}

fn verify(param: Parameter) {
    crate::setup();

    let clients_count = 10;

    let root_dir = Path::new(DATA_DIR).join("verify_bin");
    let tx_payload = misc::load_data_from_file(&root_dir, &param.tx_payload_filename);

    for (client_filename, tx_proof_filename) in &param.client_and_tx_proof_filenames {
        let client = misc::load_data_from_file(&root_dir, client_filename);
        let tx_proof = misc::load_data_from_file(&root_dir, tx_proof_filename);

        let mut context = Context::new();
        let script_version = ScriptVersion::latest();

        let deployed_lock_contract = {
            let contract_data =
                misc::load_contract_from_file(CAN_UPDATE_WITHOUT_OWNERSHIP_LOCK_CONTRACT);
            let data = contract_data.into();
            let lock_script = packed::Script::default();
            context.deploy(data, lock_script, None, None)
        };

        let deployed_client_cell = {
            let deployed_type_contract = {
                let contract_data = misc::load_contract_from_file(CLIENT_TYPE_LOCK_CONTRACT);
                let data = contract_data.into();
                let lock_script = packed::Script::default();
                let type_script = packed::Script::new_builder().args(vec![0u8].pack()).build();
                context.deploy(data, lock_script, Some(type_script), None)
            };

            {
                let data = client.into();

                let lock_args = misc::randomize_bytes();
                let lock_script = packed::Script::new_builder()
                    .hash_type(script_version.data_hash_type().into())
                    .code_hash(deployed_lock_contract.data_hash())
                    .args(lock_args.pack())
                    .build();

                let client_type_args = utils::randomize_client_type_args(clients_count);
                let type_script = packed::Script::new_builder()
                    .hash_type(ScriptHashType::Type.into())
                    .code_hash(deployed_type_contract.type_hash().unwrap())
                    .args(client_type_args)
                    .build();

                context.deploy(data, lock_script, Some(type_script), None)
            }
        };

        let deployed_bin_cell = {
            let contract_data = misc::load_contract_from_file(VERIFY_BIN_CONTRACT);
            let data = contract_data.into();
            let lock_script = packed::Script::default();
            let type_script = packed::Script::new_builder().args(vec![1u8].pack()).build();
            context.deploy(data, lock_script, Some(type_script), None)
        };

        let deployed_business_type_lock = {
            let contract_data = misc::load_contract_from_file(MOCK_BUSINESS_TYPE_LOCK_CONTRACT);
            let data = contract_data.into();
            let lock_script = packed::Script::default();
            context.deploy(data, lock_script, None, None)
        };

        let deployed_cell = {
            let data = vec![0u8].into();
            let lock_args = misc::randomize_bytes();
            let lock_script = packed::Script::new_builder()
                .hash_type(script_version.data_hash_type().into())
                .code_hash(deployed_lock_contract.data_hash())
                .args(lock_args.pack())
                .build();
            let mut type_args = Vec::<u8>::with_capacity(32 * 2);
            let client_cell_type_hash = deployed_client_cell.type_hash().unwrap().raw_data();
            let bin_cell_type_hash = deployed_bin_cell.type_hash().unwrap().raw_data();
            type_args.extend_from_slice(client_cell_type_hash.as_ref());
            type_args.extend_from_slice(bin_cell_type_hash.as_ref());
            assert_eq!(type_args.len(), 64);
            let type_script = packed::Script::new_builder()
                .hash_type(script_version.data_hash_type().into())
                .code_hash(deployed_business_type_lock.data_hash())
                .args(type_args.pack())
                .build();
            context.deploy(data, lock_script, Some(type_script), None)
        };

        let transaction = {
            let output = deployed_cell.cell_output();
            let output_data = vec![1u8];
            let witness = {
                let input_type_args = packed::BytesOpt::new_builder()
                    .set(Some(tx_proof.pack()))
                    .build();
                let output_type_args = packed::BytesOpt::new_builder()
                    .set(Some(tx_payload.pack()))
                    .build();
                let witness_args = packed::WitnessArgs::new_builder()
                    .input_type(input_type_args)
                    .output_type(output_type_args)
                    .build();
                witness_args.as_bytes()
            };
            TransactionBuilder::default()
                .cell_dep(deployed_lock_contract.as_cell_dep())
                .cell_dep(deployed_business_type_lock.as_cell_dep())
                .cell_dep(deployed_client_cell.as_cell_dep())
                .cell_dep(deployed_bin_cell.as_cell_dep())
                .input(deployed_cell.as_input())
                .output(output)
                .output_data(output_data.pack())
                .witness(witness.pack())
                .build()
        };

        let rtx = context.resolve(transaction);

        let verifier = Verifier::default();
        let result = verifier.verify_without_limit(&rtx);

        result.should_be_ok();
    }
}
