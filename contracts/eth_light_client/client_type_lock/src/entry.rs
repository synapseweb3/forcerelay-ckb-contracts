use alloc::vec::Vec;

#[cfg(feature = "debugging")]
use ckb_std::ckb_types::prelude::*;
use ckb_std::{ckb_constants::Source, high_level as hl};

use crate::{
    error::{Error, Result},
    operations,
};

pub fn main() -> Result<()> {
    debug!("{} Starting ...", module_path!());

    let script_hash = hl::load_script_hash()?;
    debug!("script hash = {:#x}", script_hash.pack());

    let mut old_client_cell_indexes = Vec::new();
    let mut new_client_cell_indexes = Vec::new();

    // Find all input cells which use current script.
    for (index, type_hash_opt) in
        hl::QueryIter::new(hl::load_cell_type_hash, Source::Input).enumerate()
    {
        if let Some(type_hash) = type_hash_opt {
            debug!("{index}-th type hash of inputs: {:#x}", type_hash.pack());
            if type_hash == script_hash.as_slice() {
                debug!("found client cell: inputs[{index}]");
                old_client_cell_indexes.push(index);
            }
        }
    }

    // Find all output cells which use current script.
    for (index, type_hash_opt) in
        hl::QueryIter::new(hl::load_cell_type_hash, Source::Output).enumerate()
    {
        if let Some(type_hash) = type_hash_opt {
            debug!("{index}-th type hash of outputs: {:#x}", type_hash.pack());
            if type_hash == script_hash.as_slice() {
                debug!("found client cell: outputs[index]");
                new_client_cell_indexes.push(index);
            }
        }
    }

    debug!("client cell in  inputs: {old_client_cell_indexes:?}");
    debug!("client cell in outputs: {new_client_cell_indexes:?}");

    if new_client_cell_indexes.is_empty() {
        debug!("destroy all client cells");
        operations::destroy_client_cells(&old_client_cell_indexes)?;
    } else if old_client_cell_indexes.is_empty() {
        debug!("create all client cells");
        operations::create_client_cells(&new_client_cell_indexes)?;
    } else if old_client_cell_indexes.len() == 2 && new_client_cell_indexes.len() == 2 {
        debug!("update a client cell");
        let input_indexes = (old_client_cell_indexes[0], old_client_cell_indexes[1]);
        let output_indexes = (new_client_cell_indexes[0], new_client_cell_indexes[1]);
        operations::update_client_cells(input_indexes, output_indexes, script_hash.as_slice())?;
    } else {
        debug!("unknown operation: throw an error");
        return Err(Error::UnknownOperation);
    }

    debug!("{} DONE.", module_path!());

    Ok(())
}
