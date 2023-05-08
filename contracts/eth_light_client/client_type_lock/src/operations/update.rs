#[cfg(feature = "debugging")]
use ckb_std::ckb_types::prelude::Pack as StdPack;
use ckb_std::{ckb_constants::Source, error::SysError, high_level as hl};
use eth_light_client_in_ckb_verification::types::{
    core::{Client, ClientInfo},
    packed::{ClientInfoReader, ClientReader, ClientTypeArgsReader, ProofUpdateReader},
    prelude::*,
};

use crate::error::{Error, Result};

pub(crate) fn update_client_cells(
    inputs: (usize, usize),
    outputs: (usize, usize),
    script_hash: &[u8],
) -> Result<()> {
    debug!("load cell data of inputs[{}]", inputs.0);
    let input_data_0 = hl::load_cell_data(inputs.0, Source::Input)?;
    debug!("load cell data of inputs[{}]", inputs.1);
    let input_data_1 = hl::load_cell_data(inputs.1, Source::Input)?;

    let (input_info, packed_input_client) =
        if let Ok(input_info) = ClientInfoReader::from_slice(&input_data_0) {
            if let Ok(input_client) = ClientReader::from_slice(&input_data_1) {
                (input_info, input_client)
            } else {
                return Err(Error::UpdateInputClientCellNotFound);
            }
        } else if let Ok(input_info) = ClientInfoReader::from_slice(&input_data_1) {
            if let Ok(input_client) = ClientReader::from_slice(&input_data_0) {
                (input_info, input_client)
            } else {
                return Err(Error::UpdateInputClientCellNotFound);
            }
        } else {
            return Err(Error::UpdateInputClientInfoCellNotFound);
        };
    debug!("packed input info = {input_info}");
    debug!("packed input client = {packed_input_client}");

    debug!("load cell data of outputs[{}]", outputs.0);
    let output_data_0 = hl::load_cell_data(outputs.0, Source::Output)?;
    debug!("load cell data of outputs[{}]", outputs.1);
    let output_data_1 = hl::load_cell_data(outputs.1, Source::Output)?;

    let (output_info, output_client, output_client_index) =
        if let Ok(output_info) = ClientInfoReader::from_slice(&output_data_0) {
            if let Ok(output_client) = ClientReader::from_slice(&output_data_1) {
                (output_info, output_client, outputs.1)
            } else {
                return Err(Error::UpdateOutputClientCellNotFound);
            }
        } else if let Ok(output_info) = ClientInfoReader::from_slice(&output_data_1) {
            if let Ok(output_client) = ClientReader::from_slice(&output_data_0) {
                (output_info, output_client, outputs.0)
            } else {
                return Err(Error::UpdateOutputClientCellNotFound);
            }
        } else {
            return Err(Error::UpdateOutputClientInfoCellNotFound);
        };
    debug!("packed output info = {output_info}");
    debug!("packed output client = {output_client}");

    let input_info: ClientInfo = input_info.unpack();
    let output_info: ClientInfo = output_info.unpack();

    if input_info.minimal_updates_count != output_info.minimal_updates_count {
        return Err(Error::UpdateClientInfoMinimalUpdatesCountChanged);
    }

    let client_type_args = {
        let script = hl::load_script()?;
        let script_args = script.args();
        let script_args_slice = script_args.as_reader().raw_data();
        ClientTypeArgsReader::from_slice(script_args_slice)
            .map_err(|_| SysError::Encoding)?
            .unpack()
    };

    let expected_last_id = if input_info.last_id + 2 < client_type_args.cells_count {
        input_info.last_id + 1
    } else {
        0
    };
    debug!("expected last id = {expected_last_id}");

    if output_info.last_id != expected_last_id {
        return Err(Error::UpdateClientInfoNewLastIdIsIncorrect);
    }

    let input_client = packed_input_client.unpack();
    debug!("input client id = {}", input_client.id);
    if input_client.id != expected_last_id {
        return Err(Error::UpdateClientInputLastIdIsIncorrect);
    }

    // Find the cell dep which use current script.
    let cell_dep_index = {
        let mut cell_dep_index_opt = None;
        for (index, type_hash_opt) in
            hl::QueryIter::new(hl::load_cell_type_hash, Source::CellDep).enumerate()
        {
            if let Some(type_hash) = type_hash_opt {
                debug!(
                    "{index}-th type hash of cell-deps: {:#x}",
                    StdPack::pack(&type_hash)
                );
                if type_hash == script_hash {
                    if cell_dep_index_opt.is_none() {
                        cell_dep_index_opt = Some(index);
                    } else {
                        return Err(Error::UpdateMoreThanOneCellDepsWithCurrentType);
                    }
                }
            }
        }
        if let Some(cell_dep_index) = cell_dep_index_opt {
            cell_dep_index
        } else {
            return Err(Error::UpdateCellDepClientCellNotFound);
        }
    };

    debug!("load cell data of cell_dep[{cell_dep_index}]");
    let cell_dep_data = hl::load_cell_data(cell_dep_index, Source::CellDep)?;
    let cell_dep_client = if let Ok(cell_dep_client) = ClientReader::from_slice(&cell_dep_data) {
        cell_dep_client
    } else {
        return Err(Error::UpdateCellDepClientCellNotFound);
    };
    debug!("packed cell-dep client = {cell_dep_client}");

    let cell_dep_client_id = cell_dep_client.id().as_slice()[0];
    debug!("cell-dep client id = {cell_dep_client_id}");
    if input_info.last_id != cell_dep_client_id {
        return Err(Error::UpdateCellDepLastIdIsIncorrect);
    }

    let witness_args = hl::load_witness_args(output_client_index, Source::Output)?;
    let expected_client = if let Some(args) = witness_args.input_type().to_opt() {
        let data = args.raw_data();
        let proof_update = ProofUpdateReader::from_slice(&data).map_err(|_| SysError::Encoding)?;
        debug!(
            "packed proof update size = {}",
            proof_update.as_slice().len()
        );

        let mut client: Client = cell_dep_client.unpack();
        client.id = input_client.id;
        client.try_apply_packed_proof_update(proof_update)?
    } else {
        return Err(Error::UpdateWitnessIsNotExisted);
    };

    let actual_updates_count = expected_client.maximal_slot - input_client.maximal_slot + 1;
    if actual_updates_count < u64::from(input_info.minimal_updates_count) {
        return Err(Error::UpdateUpdatesIsNotEnough);
    }

    let packed_expected_client = expected_client.pack();
    debug!("packed expected client = {packed_expected_client}");

    if output_client.as_slice() != packed_expected_client.as_slice() {
        return Err(Error::UpdateNewClientIsIncorrect);
    }

    Ok(())
}
