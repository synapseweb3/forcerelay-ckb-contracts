use std::fs;

use ckb_types::{
    bytes::Bytes,
    core::{
        cell::{CellMeta, CellMetaBuilder},
        Capacity, TransactionInfo,
    },
    packed,
    prelude::*,
};

pub fn load_cell_from_path(path_str: &str) -> (CellMeta, packed::Byte32) {
    let cell_data = fs::read(path_str).unwrap();
    load_cell_from_slice(&cell_data)
}

pub fn load_cell_from_slice(slice: &[u8]) -> (CellMeta, packed::Byte32) {
    let cell_data = Bytes::copy_from_slice(slice);
    let cell_output = packed::CellOutput::new_builder()
        .capacity(Capacity::bytes(cell_data.len()).unwrap().pack())
        .build();
    let cell_meta = CellMetaBuilder::from_cell_output(cell_output, cell_data)
        .transaction_info(default_transaction_info())
        .build();
    let data_hash = cell_meta.mem_cell_data_hash.as_ref().unwrap().to_owned();
    (cell_meta, data_hash)
}

pub fn default_transaction_info() -> TransactionInfo {
    packed::TransactionInfo::new_builder()
        .block_number(1u64.pack())
        .block_epoch(0u64.pack())
        .key(
            packed::TransactionKey::new_builder()
                .block_hash(packed::Byte32::zero())
                .index(1u32.pack())
                .build(),
        )
        .build()
        .unpack()
}

pub fn create_dummy_cell(output: packed::CellOutput) -> CellMeta {
    CellMetaBuilder::from_cell_output(output, Bytes::new())
        .transaction_info(default_transaction_info())
        .build()
}
