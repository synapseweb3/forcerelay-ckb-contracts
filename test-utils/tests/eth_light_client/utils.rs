use ckb_hash::{new_blake2b, BLAKE2B_LEN};
use ibc_ckb_contracts_test_utils::{
    ckb::types::{packed, prelude::*},
    misc,
};

pub(crate) fn build_client_type_args(input: &packed::CellInput, cells_count: u8) -> packed::Bytes {
    let type_id = calculate_type_id(input, cells_count as usize);
    let mut client_type_args = Vec::with_capacity(type_id.len() + 1);
    client_type_args.extend_from_slice(&type_id);
    client_type_args.push(cells_count);
    client_type_args.pack()
}

pub(crate) fn randomize_client_type_args(cells_count: u8) -> packed::Bytes {
    let type_id = misc::randomize_fixed_bytes::<BLAKE2B_LEN>();
    let mut client_type_args = Vec::with_capacity(type_id.len() + 1);
    client_type_args.extend_from_slice(&type_id);
    client_type_args.push(cells_count);
    client_type_args.pack()
}

pub(crate) fn build_client_info(last_id: u8, minimal_updates_count: u8) -> packed::Bytes {
    [last_id, minimal_updates_count].pack()
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
