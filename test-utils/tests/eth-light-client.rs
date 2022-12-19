use ibc_ckb_contracts_test_utils::{
    ckb::{
        script::ScriptVersion,
        types::{
            core::{capacity_bytes, cell::ResolvedTransaction, Capacity, TransactionBuilder},
            packed,
            prelude::*,
        },
    },
    misc, Verifier,
};

const CONTRACT_BINARY: &str = "../build/eth-light-client";

#[test]
fn test_eth_light_client() {
    let (cell, data_hash) = misc::load_cell_from_path(CONTRACT_BINARY);
    let script_version = ScriptVersion::latest();

    let script = packed::Script::new_builder()
        .hash_type(script_version.data_hash_type().into())
        .code_hash(data_hash)
        .build();
    let output = packed::CellOutput::new_builder()
        .capacity(capacity_bytes!(100).pack())
        .lock(script)
        .build();
    let input = packed::CellInput::new(packed::OutPoint::null(), 0);

    let transaction = TransactionBuilder::default().input(input).build();
    let dummy_cell = misc::create_dummy_cell(output);

    let rtx = ResolvedTransaction {
        transaction,
        resolved_cell_deps: vec![cell],
        resolved_inputs: vec![dummy_cell],
        resolved_dep_groups: vec![],
    };

    let verifier = Verifier::default();
    let result = verifier.verify_without_limit(&rtx);

    assert!(result.is_ok());
}
