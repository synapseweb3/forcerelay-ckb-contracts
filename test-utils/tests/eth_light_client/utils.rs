use std::path::Path;

use ckb_hash::{new_blake2b, BLAKE2B_LEN};
use ibc_ckb_contracts_test_utils::{
    ckb::types::{packed, prelude::*},
    misc,
};

use super::{DATA_DIR, GENESIS_VALIDATORS_ROOT_DATA};

pub(crate) fn build_client_type_args(
    input: &packed::CellInput,
    clients_count: u8,
) -> packed::Bytes {
    let type_id = calculate_type_id(input, clients_count as usize + 3);
    let mut client_type_args = Vec::with_capacity(type_id.len() + 1);
    client_type_args.extend_from_slice(&type_id);
    client_type_args.push(clients_count);
    client_type_args.pack()
}

pub(crate) fn randomize_client_type_args(clients_count: u8) -> packed::Bytes {
    let type_id = misc::randomize_fixed_bytes::<BLAKE2B_LEN>();
    let mut client_type_args = Vec::with_capacity(type_id.len() + 1);
    client_type_args.extend_from_slice(&type_id);
    client_type_args.push(clients_count);
    client_type_args.pack()
}

pub(crate) fn load_genesis_validators_root() -> Vec<u8> {
    let root_dir = Path::new(DATA_DIR);
    misc::load_data_from_file(&root_dir, GENESIS_VALIDATORS_ROOT_DATA)
}

pub(crate) fn build_client_info(last_client_id: u8, minimal_headers_count: u8) -> packed::Bytes {
    let mut data = vec![last_client_id, minimal_headers_count];
    let mut genesis_validators_root = load_genesis_validators_root();
    data.append(&mut genesis_validators_root);
    data.pack()
}

pub(crate) fn calculate_type_id(
    input: &packed::CellInput,
    outputs_count: usize,
) -> [u8; BLAKE2B_LEN] {
    let mut hasher = new_blake2b();
    hasher.update(input.as_slice());
    hasher.update(&(outputs_count as u64).to_le_bytes());
    let mut result = [0u8; BLAKE2B_LEN];
    hasher.finalize(&mut result);
    result
}
