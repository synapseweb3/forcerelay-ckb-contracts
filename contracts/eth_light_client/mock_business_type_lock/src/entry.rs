use alloc::{ffi::CString, string::ToString as _};

use ckb_std::{
    ckb_constants::Source,
    ckb_types::{core::ScriptHashType, packed::Byte32Reader},
    high_level as hl,
};
use eth_light_client_in_ckb_verification::types::prelude::*;

use crate::error::{Error, Result};

const WITNESS_INDEX: usize = 0;

pub fn main() -> Result<()> {
    debug!("{} Starting ...", module_path!());

    let args = hl::load_script()?.args();
    debug!("args = {:#x}", args);
    if args.len() != 32 * 2 {
        return Err(Error::IncorrectArgs);
    }

    let args_raw_data = args.raw_data();
    let client_cell_type_hash_slice = &args_raw_data.slice(0..32);
    let bin_cell_type_hash_slice = &args_raw_data.slice(32..64);

    let client_cell_type_hash = Byte32Reader::new_unchecked(client_cell_type_hash_slice);
    let bin_cell_type_hash = Byte32Reader::new_unchecked(bin_cell_type_hash_slice);
    debug!("client cell type hash = {:#x}", client_cell_type_hash);
    debug!("   bin cell type hash = {:#x}", bin_cell_type_hash);

    let mut client_cell_index_opt = None;
    let mut bin_cell_index_opt = None;

    for (index, type_hash_opt) in
        hl::QueryIter::new(hl::load_cell_type_hash, Source::CellDep).enumerate()
    {
        if let Some(type_hash) = type_hash_opt {
            debug!("{}-th type hash: {:#x}", index, type_hash.pack());
            if type_hash == client_cell_type_hash.as_slice() {
                client_cell_index_opt = Some(index);
            } else if type_hash == bin_cell_type_hash.as_slice() {
                bin_cell_index_opt = Some(index);
            }
        }
    }

    let client_cell_index_string = if let Some(client_cell_index) = client_cell_index_opt {
        CString::new(client_cell_index.to_string())?
    } else {
        return Err(Error::ClientCellDepIsNotExisted);
    };

    if bin_cell_index_opt.is_none() {
        return Err(Error::BinCellDepIsNotExisted);
    }

    let witness_index_string = CString::new(WITNESS_INDEX.to_string())?;

    let retcode = hl::exec_cell(
        bin_cell_type_hash.as_slice(),
        ScriptHashType::Type,
        0,
        0,
        &[&client_cell_index_string, &witness_index_string],
    )?;

    if retcode != 0 {
        return Err(Error::FailedToExecuteBinCell);
    }

    debug!("{} DONE.", module_path!());

    Ok(())
}
