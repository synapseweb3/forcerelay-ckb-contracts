use ckb_std::{error::SysError, high_level as hl};
use eth_light_client_in_ckb_verification::types::{packed::ClientTypeArgsReader, prelude::*};

use crate::error::{Error, Result};

pub(crate) fn destroy_client_cells(indexes: &[usize]) -> Result<()> {
    let client_type_args = {
        let script = hl::load_script()?;
        let script_args = script.args();
        let script_args_slice = script_args.as_reader().raw_data();
        ClientTypeArgsReader::from_slice(script_args_slice)
            .map_err(|_| SysError::Encoding)?
            .unpack()
    };
    if indexes.len() != client_type_args.cells_count as usize {
        return Err(Error::DestroyNotEnoughCells);
    }
    Ok(())
}
