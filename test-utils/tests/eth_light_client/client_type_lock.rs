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
use crate::{mock_contracts::REVERSE_ARGS_LOCK_CONTRACT, setup};

#[test]
fn test() {
    setup();

    let client_id = misc::random_hash().raw_data().to_vec();

    let root_dir = Path::new(DATA_DIR).join("client_type_lock");
    let client = misc::load_data_from_file(&root_dir, "client.data");
    let new_client = misc::load_data_from_file(&root_dir, "new_client.data");
    let proof_update = misc::load_data_from_file(&root_dir, "proof_update.data");

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

    assert!(result.is_ok());
}
