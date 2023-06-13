use alloc::vec::Vec;

#[cfg(feature = "debugging")]
use ckb_std::ckb_types::prelude::Pack as StdPack;
use ckb_std::{ckb_constants::Source, error::SysError, high_level as hl};
use eth_light_client_in_ckb_verification::types::{
    core::{Client, ClientInfo},
    packed::{
        ClientInfoReader, ClientReader, ClientSyncCommitteeReader, ClientTypeArgsReader,
        ClientUpdateReader,
    },
    prelude::*,
};

use crate::error::{InternalError, Result};

const EXPECTED_CELL_DEPS_COUNT: usize = 2;

pub(crate) fn update_client(
    inputs: (usize, usize),
    outputs: (usize, usize),
    script_hash: &[u8],
) -> Result<()> {
    // Checks the id of the input client cell, then returns
    // - expected output info cell base on the input info cell,
    // - the last client id.
    let (expected_info, last_client_id, expected_client_id) = {
        let (mut input_info, last_client_id, expected_client_id) = load_inputs(inputs)?;
        input_info.last_client_id = expected_client_id;
        (input_info, last_client_id, expected_client_id)
    };
    // Checks the output info cell, then returns the output client cell and its index,
    let (output_client, output_client_index) = load_outputs(outputs, &expected_info)?;
    // Finds the indexes of cell deps which use current script.
    let cell_deps = find_cell_deps(script_hash)?;
    // Checks the id of the cell-dep client cell, then returns:
    // - expected input client cell base on the cell-dep client cell,
    // - the cell-dep sync committee cell.
    let (expected_client, sync_committee_data) = {
        let (mut cell_dep_client, cell_dep_sync_committee_data) =
            load_cell_deps(cell_deps, last_client_id)?;
        cell_dep_client.id = expected_client_id;
        (cell_dep_client, cell_dep_sync_committee_data)
    };
    let packed_sync_committee = ClientSyncCommitteeReader::new_unchecked(&sync_committee_data);
    // Gets the client update from the witness.
    let client_update = {
        let witness_args = hl::load_witness_args(output_client_index, Source::Output)?;
        if let Some(args) = witness_args.input_type().to_opt() {
            ClientUpdateReader::from_slice(&args.raw_data())
                .map_err(|_| SysError::Encoding)?
                .unpack()
        } else {
            return Err(InternalError::UpdateClientWitnessIsNotExisted.into());
        }
    };

    if client_update.headers.len() < usize::from(expected_info.minimal_headers_count) {
        return Err(InternalError::UpdateClientHeadersNotEnough.into());
    }

    client_update.verify_client_update(
        expected_client,
        expected_info.genesis_validators_root,
        packed_sync_committee,
        output_client,
    )?;

    Ok(())
}

fn load_inputs(inputs: (usize, usize)) -> Result<(ClientInfo, u8, u8)> {
    debug!("load cell data of inputs[{}]", inputs.0);
    let input_data_0 = hl::load_cell_data(inputs.0, Source::Input)?;
    debug!("load cell data of inputs[{}]", inputs.1);
    let input_data_1 = hl::load_cell_data(inputs.1, Source::Input)?;

    let (packed_input_info, packed_input_client) =
        if let Ok(input_info) = ClientInfoReader::from_slice(&input_data_0) {
            debug!("input info = {input_info} (index={})", inputs.0);
            if let Ok(input_client) = ClientReader::from_slice(&input_data_1) {
                debug!("input client = {input_client} (index={})", inputs.1);
                (input_info, input_client)
            } else {
                return Err(InternalError::UpdateClientInputClientNotFound.into());
            }
        } else if let Ok(input_info) = ClientInfoReader::from_slice(&input_data_1) {
            debug!("input info = {input_info} (index={})", inputs.1);
            if let Ok(input_client) = ClientReader::from_slice(&input_data_0) {
                debug!("input client = {input_client} (index={})", inputs.0);
                (input_info, input_client)
            } else {
                return Err(InternalError::UpdateClientInputClientNotFound.into());
            }
        } else {
            return Err(InternalError::UpdateClientInputInfoNotFound.into());
        };

    let input_info: ClientInfo = packed_input_info.unpack();
    let last_client_id = input_info.last_client_id;
    debug!("last client id = {last_client_id}");
    let input_client_id: u8 = packed_input_client.id().into();
    debug!("input client id = {input_client_id}");

    let clients_count: u8 = {
        let script = hl::load_script()?;
        let script_args = script.args();
        let script_args_slice = script_args.as_reader().raw_data();
        ClientTypeArgsReader::from_slice(script_args_slice)
            .map_err(|_| SysError::Encoding)?
            .clients_count()
            .into()
    };
    debug!("clients count: {clients_count}");
    let expected_client_id = if input_info.last_client_id + 1 < clients_count {
        input_info.last_client_id + 1
    } else {
        0
    };
    debug!("expected client id = {expected_client_id}");
    if input_client_id != expected_client_id {
        return Err(InternalError::UpdateClientInputClientIdIsMismatch.into());
    }

    Ok((input_info, last_client_id, expected_client_id))
}

fn load_outputs(outputs: (usize, usize), expected_info: &ClientInfo) -> Result<(Client, usize)> {
    debug!("load cell data of outputs[{}]", outputs.0);
    let output_data_0 = hl::load_cell_data(outputs.0, Source::Output)?;
    debug!("load cell data of outputs[{}]", outputs.1);
    let output_data_1 = hl::load_cell_data(outputs.1, Source::Output)?;

    let (packed_output_info, packed_output_client, output_client_index) =
        if let Ok(output_info) = ClientInfoReader::from_slice(&output_data_0) {
            debug!("output info = {output_info} (index={})", outputs.0);
            if let Ok(output_client) = ClientReader::from_slice(&output_data_1) {
                debug!("output client = {output_client} (index={})", outputs.1);
                (output_info, output_client, outputs.1)
            } else {
                return Err(InternalError::UpdateClientOutputClientNotFound.into());
            }
        } else if let Ok(output_info) = ClientInfoReader::from_slice(&output_data_1) {
            debug!("output info = {output_info} (index={})", outputs.1);
            if let Ok(output_client) = ClientReader::from_slice(&output_data_0) {
                debug!("output client = {output_client} (index={})", outputs.0);
                (output_info, output_client, outputs.0)
            } else {
                return Err(InternalError::UpdateClientOutputClientNotFound.into());
            }
        } else {
            return Err(InternalError::UpdateClientOutputInfoNotFound.into());
        };

    let packed_expected_info = expected_info.pack();
    debug!("expected info = {packed_expected_info}");
    if packed_output_info.as_slice() != packed_expected_info.as_slice() {
        return Err(InternalError::UpdateClientInfoChanged.into());
    }
    let output_client: Client = packed_output_client.unpack();

    Ok((output_client, output_client_index))
}

fn find_cell_deps(script_hash: &[u8]) -> Result<(usize, usize)> {
    let mut indexes = Vec::new();
    for (index, type_hash_opt) in
        hl::QueryIter::new(hl::load_cell_type_hash, Source::CellDep).enumerate()
    {
        if let Some(type_hash) = type_hash_opt {
            debug!(
                "{index}-th type hash of cell-deps: {:#x}",
                StdPack::pack(&type_hash)
            );
            if type_hash == script_hash {
                if indexes.len() < EXPECTED_CELL_DEPS_COUNT {
                    indexes.push(index);
                } else {
                    return Err(InternalError::UpdateClientCellDepsTooMany.into());
                }
            }
        }
    }
    if indexes.len() != EXPECTED_CELL_DEPS_COUNT {
        return Err(InternalError::UpdateClientCellDepsNotEnough.into());
    }
    Ok((indexes[0], indexes[1]))
}

fn load_cell_deps(cell_deps: (usize, usize), last_client_id: u8) -> Result<(Client, Vec<u8>)> {
    debug!("load cell data of cell deps[{}]", cell_deps.0);
    let cell_dep_data_0 = hl::load_cell_data(cell_deps.0, Source::CellDep)?;
    debug!("load cell data of cell deps[{}]", cell_deps.1);
    let cell_dep_data_1 = hl::load_cell_data(cell_deps.1, Source::CellDep)?;

    let (packed_cell_dep_client, cell_dep_sync_committee_data) =
        if let Ok(cell_dep_client) = ClientReader::from_slice(&cell_dep_data_0) {
            debug!(
                "cell-dep client = {cell_dep_client} (index={})",
                cell_deps.0
            );
            if let Ok(_cell_dep_sync_committee) =
                ClientSyncCommitteeReader::from_slice(&cell_dep_data_1)
            {
                debug!(
                    "cell-dep sync committee (index={}): \
                    {{ period: {}, pubkeys-length: {}, aggregate_pubkey: {} }}",
                    cell_deps.1,
                    _cell_dep_sync_committee.period(),
                    _cell_dep_sync_committee.data().pubkeys().len(),
                    _cell_dep_sync_committee.data().aggregate_pubkey()
                );
                (cell_dep_client, cell_dep_data_1)
            } else {
                return Err(InternalError::UpdateClientCellDepSyncCommitteeNotFound.into());
            }
        } else if let Ok(cell_dep_client) = ClientReader::from_slice(&cell_dep_data_1) {
            debug!(
                "cell-dep client = {cell_dep_client} (index={})",
                cell_deps.1
            );
            if let Ok(_cell_dep_sync_committee) =
                ClientSyncCommitteeReader::from_slice(&cell_dep_data_0)
            {
                debug!(
                    "cell-dep sync committee (index={}): \
                    {{ period: {}, pubkeys-length: {}, aggregate_pubkey: {} }}",
                    cell_deps.0,
                    _cell_dep_sync_committee.period(),
                    _cell_dep_sync_committee.data().pubkeys().len(),
                    _cell_dep_sync_committee.data().aggregate_pubkey()
                );
                (cell_dep_client, cell_dep_data_0)
            } else {
                return Err(InternalError::UpdateClientCellDepSyncCommitteeNotFound.into());
            }
        } else {
            return Err(InternalError::UpdateClientCellDepClientNotFound.into());
        };

    let cell_dep_client: Client = packed_cell_dep_client.unpack();
    debug!("cell-dep client id = {}", cell_dep_client.id);
    if cell_dep_client.id != last_client_id {
        return Err(InternalError::UpdateClientCellDepClientIdIsMismatch.into());
    }

    Ok((cell_dep_client, cell_dep_sync_committee_data))
}
