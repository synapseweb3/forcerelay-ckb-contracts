use ibc_ckb_contracts_test_utils::{
    ckb::{
        script::ScriptVersion,
        types::{
            core::{Capacity, ScriptHashType, TransactionBuilder},
            packed,
            prelude::*,
        },
    },
    misc, Context, Verifier,
};

use super::super::{utils, CLIENT_TYPE_LOCK_CONTRACT};
use crate::{mock_contracts::CAN_UPDATE_WITHOUT_OWNERSHIP_LOCK_CONTRACT, prelude::*};

#[test]
fn testcase_just_right() {
    let param = Parameter {
        clients_count: 3,
        actual_cells_count: 6,
    };
    destroy(param);
}

#[test]
#[should_panic]
fn testcase_one_less() {
    let param = Parameter {
        clients_count: 3,
        actual_cells_count: 7,
    };
    destroy(param);
}

#[test]
#[should_panic]
fn testcase_one_more() {
    let param = Parameter {
        clients_count: 3,
        actual_cells_count: 5,
    };
    destroy(param);
}

struct Parameter {
    clients_count: u8,
    actual_cells_count: u8,
}

fn destroy(param: Parameter) {
    crate::setup();

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

    let client_type_args = utils::randomize_client_type_args(param.clients_count);
    let type_script = packed::Script::new_builder()
        .hash_type(ScriptHashType::Type.into())
        .code_hash(deployed_type_contract.type_hash().unwrap())
        .args(client_type_args)
        .build();

    let deployed_cells = (0..param.actual_cells_count)
        .map(|byte| {
            let data = vec![byte].into();
            context.deploy(data, lock_script.clone(), Some(type_script.clone()), None)
        })
        .collect::<Vec<_>>();

    let transaction = {
        let total_capacity = deployed_cells
            .iter()
            .fold(Capacity::shannons(0), |total, cell| {
                let added: Capacity = cell.cell_output().capacity().unpack();
                total.safe_add(added).unwrap()
            });
        let output = deployed_cells[0]
            .cell_output()
            .as_builder()
            .type_(None::<packed::Script>.pack())
            .capacity(total_capacity.pack())
            .build();
        let mut tx_builder = TransactionBuilder::default()
            .cell_dep(deployed_lock_contract.as_cell_dep())
            .cell_dep(deployed_type_contract.as_cell_dep());
        for cell in deployed_cells {
            tx_builder = tx_builder.input(cell.as_input());
        }
        tx_builder
            .output(output)
            .output_data(Default::default())
            .build()
    };

    let rtx = context.resolve(transaction);

    let verifier = Verifier::default();
    let result = verifier.verify_without_limit(&rtx);

    result.should_be_ok();
}
