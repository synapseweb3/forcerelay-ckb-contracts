use std::path::Path;

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

use super::super::{utils, CLIENT_TYPE_LOCK_CONTRACT, DATA_DIR};
use crate::{mock_contracts::CAN_UPDATE_WITHOUT_OWNERSHIP_LOCK_CONTRACT, prelude::*};

#[test]
fn destroy_case_1() {
    let param = DestroyParameter {
        case_id: 1,
        clients_count: 3,
        client_filename: "client-6268480_6268543.data",
    };
    destroy(param);
}

struct DestroyParameter {
    case_id: usize,
    clients_count: u8,
    client_filename: &'static str,
}

fn destroy(param: DestroyParameter) {
    crate::setup();

    let minimal_updates_count = 64;

    let case_dir = format!("case-{}", param.case_id);
    let root_dir = Path::new(DATA_DIR).join("client_type_lock").join(case_dir);
    let mut client = misc::load_data_from_file(&root_dir, param.client_filename);

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

    let deployed_clients = (0..param.clients_count)
        .map(|id| {
            client[0] = id;
            let data = client.clone().into();
            context.deploy(data, lock_script.clone(), Some(type_script.clone()), None)
        })
        .collect::<Vec<_>>();

    let deployed_client_info = {
        let client_info = utils::build_client_info(0, minimal_updates_count);
        context.deploy(client_info.unpack(), lock_script, Some(type_script), None)
    };

    let transaction = {
        let info_capacity: Capacity = deployed_client_info.cell_output().capacity().unpack();
        let total_capacity = deployed_clients
            .iter()
            .fold(info_capacity, |total, client| {
                let added: Capacity = client.cell_output().capacity().unpack();
                total.safe_add(added).unwrap()
            });
        let output = deployed_client_info
            .cell_output()
            .as_builder()
            .type_(None::<packed::Script>.pack())
            .capacity(total_capacity.pack())
            .build();
        let mut tx_builder = TransactionBuilder::default()
            .cell_dep(deployed_lock_contract.as_cell_dep())
            .cell_dep(deployed_type_contract.as_cell_dep());
        for client in deployed_clients {
            tx_builder = tx_builder.input(client.as_input());
        }
        tx_builder
            .input(deployed_client_info.as_input())
            .output(output)
            .output_data(Default::default())
            .build()
    };

    let rtx = context.resolve(transaction);

    let verifier = Verifier::default();
    let result = verifier.verify_without_limit(&rtx);

    result.should_be_ok();
}
