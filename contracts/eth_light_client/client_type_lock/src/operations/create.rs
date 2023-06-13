use ckb_std::{ckb_constants::Source, error::SysError, high_level as hl};
use eth_light_client_in_ckb_verification::types::{
    packed::{
        ClientBootstrapReader, ClientInfoReader, ClientReader, ClientSyncCommitteeReader,
        ClientTypeArgsReader,
    },
    prelude::*,
};

use crate::{
    error::{InternalError, Result},
    utils,
};

pub(crate) fn create_cells(indexes: &[usize]) -> Result<()> {
    if indexes.len() < 1 + 1 + 2 {
        return Err(InternalError::CreateNotEnoughCells.into());
    }
    if indexes.windows(2).any(|pair| pair[0] + 1 != pair[1]) {
        return Err(InternalError::CreateShouldBeOrdered.into());
    }
    // Checks args of the client type script, then returns the clients count;
    let clients_count = {
        let client_type_args = {
            let script = hl::load_script()?;
            let script_args = script.args();
            let script_args_slice = script_args.as_reader().raw_data();
            ClientTypeArgsReader::from_slice(script_args_slice)
                .map_err(|_| SysError::Encoding)?
                .unpack()
        };
        let clients_count = usize::from(client_type_args.clients_count);
        let cells_count = 1 + clients_count + 2;
        if indexes.len() != cells_count {
            return Err(InternalError::CreateCellsCountNotMatched.into());
        }
        let type_id = utils::calculate_type_id(indexes.len())?;
        if type_id != client_type_args.type_id.as_bytes() {
            return Err(InternalError::CreateIncorrectUniqueId.into());
        }
        clients_count
    };
    // First cell is the client info cell.
    let mut index = indexes[0];
    {
        debug!("check client info cell (index={index})");
        let output_data = hl::load_cell_data(index, Source::Output)?;
        let packed_info = ClientInfoReader::from_slice(&output_data)
            .map_err(|_| InternalError::CreateBadClientInfoCellData)?;
        debug!("actual client info cell: {packed_info}");
        let info = packed_info.unpack();
        if info.last_client_id != 0 {
            return Err(InternalError::CreateClientInfoIndexShouldBeZero.into());
        }
        if info.minimal_headers_count == 0 {
            return Err(InternalError::CreateClientInfoMinimalHeadersCountShouldNotBeZero.into());
        }
    }
    // Gets the client bootstrap from the witness.
    let client_bootstrap = {
        let witness_args = hl::load_witness_args(index, Source::Output)?;
        if let Some(args) = witness_args.input_type().to_opt() {
            ClientBootstrapReader::from_slice(&args.raw_data())
                .map_err(|_| SysError::Encoding)?
                .unpack()
        } else {
            return Err(InternalError::CreateWitnessIsNotExisted.into());
        }
    };
    // Gets the new client from the client bootstrap.
    let mut expected_client = client_bootstrap.header.initialize_client();
    debug!("expected client cell (id=0): {}", expected_client.pack());
    // Next `clients_count` cells are the client cells;
    index += 1;
    for _id in 0..clients_count {
        debug!("check client cell (index={index}, id={_id})");
        let output_data = hl::load_cell_data(index, Source::Output)?;
        let actual = ClientReader::from_slice(&output_data)
            .map_err(|_| InternalError::CreateBadClientCellData)?;
        debug!("actual client cell: {actual}");
        let expected = expected_client.pack();
        if actual.as_slice() != expected.as_slice() {
            return Err(InternalError::CreateNewClientIsIncorrect.into());
        }
        expected_client.id += 1;
        index += 1;
    }
    // Last 2 cells are the client sync committee cells.
    debug!("check 1st sync committee cell (index={index})");
    let output_data = hl::load_cell_data(index, Source::Output)?;
    let packed_sync_committee = ClientSyncCommitteeReader::from_slice(&output_data)
        .map_err(|_| InternalError::CreateBadClientSyncCommitteeCellData)?;
    debug!(
        "actual 1st sync committee: period: {}, pubkeys-length: {}, aggregate_pubkey: {}",
        packed_sync_committee.period(),
        packed_sync_committee.data().pubkeys().len(),
        packed_sync_committee.data().aggregate_pubkey()
    );
    {
        // The two client sync committee cells should be the same as each other.
        index += 1;
        debug!("check 2nd sync committee cell (index={index})");
        let output_data = hl::load_cell_data(index, Source::Output)?;
        let packed_sync_committee_copied = ClientSyncCommitteeReader::from_slice(&output_data)
            .map_err(|_| InternalError::CreateBadClientSyncCommitteeCellData)?;
        debug!(
            "actual 2nd sync committee: {{ period: {}, pubkeys-length: {}, aggregate_pubkey: {} }}",
            packed_sync_committee_copied.period(),
            packed_sync_committee_copied.data().pubkeys().len(),
            packed_sync_committee_copied.data().aggregate_pubkey()
        );
        if packed_sync_committee.as_slice() != packed_sync_committee_copied.as_slice() {
            return Err(InternalError::CreateBadClientSyncCommitteeCellData.into());
        }
    }
    client_bootstrap.verify_packed_client_sync_committee(packed_sync_committee)?;

    Ok(())
}
