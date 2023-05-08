use ckb_std::{ckb_constants::Source, error::SysError, high_level as hl};
use eth_light_client_in_ckb_verification::types::{
    core::Client,
    packed::{ClientInfoReader, ClientReader, ClientTypeArgsReader, ProofUpdateReader},
    prelude::*,
};

use crate::{
    error::{Error, Result},
    utils,
};

pub(crate) fn create_client_cells(indexes: &[usize]) -> Result<()> {
    if indexes.len() < 2 {
        return Err(Error::CreateNotEnoughCells);
    }
    if indexes.iter().enumerate().any(|(i, val)| i != *val) {
        return Err(Error::CreateShouldBeOrdered);
    }
    let client_type_args = {
        let script = hl::load_script()?;
        let script_args = script.args();
        let script_args_slice = script_args.as_reader().raw_data();
        ClientTypeArgsReader::from_slice(script_args_slice)
            .map_err(|_| SysError::Encoding)?
            .unpack()
    };
    if indexes.len() != client_type_args.cells_count as usize {
        return Err(Error::CreateCellsCountNotMatched);
    }
    let type_id = utils::calculate_type_id(indexes.len())?;
    if type_id != client_type_args.type_id.as_bytes() {
        return Err(Error::CreateIncorrectUniqueId);
    }

    let client_cells_count = indexes.len() - 1;

    let output_data = hl::load_cell_data(client_cells_count, Source::Output)?;
    let packed_actual_info = ClientInfoReader::from_slice(&output_data)
        .map_err(|_| Error::CreateBadClientInfoCellData)?;
    debug!("actual new client info {packed_actual_info}");
    let actual_info = packed_actual_info.unpack();
    if actual_info.last_id != 0 {
        return Err(Error::CreateClientInfoIndexShouldBeZero);
    }
    if actual_info.minimal_updates_count == 0 {
        return Err(Error::CreateClientInfoMinimalUpdatesCountShouldNotBeZero);
    }

    let witness_args = hl::load_witness_args(0, Source::Output)?;
    let mut expected_client = if let Some(args) = witness_args.input_type().to_opt() {
        let data = args.raw_data();
        let proof_update = ProofUpdateReader::from_slice(&data).map_err(|_| SysError::Encoding)?;
        debug!(
            "packed proof update size = {}",
            proof_update.as_slice().len()
        );
        Client::new_from_packed_proof_update(proof_update)?
    } else {
        return Err(Error::CreateWitnessIsNotExisted);
    };
    debug!("expected new client (id=0) = {}", expected_client.pack());

    let actual_updates_count = expected_client.maximal_slot - expected_client.minimal_slot + 1;
    if actual_updates_count < u64::from(actual_info.minimal_updates_count) {
        return Err(Error::CreateUpdatesIsNotEnough);
    }

    for index in 0..client_cells_count {
        debug!("check client cell (id={index})");
        let output_data = hl::load_cell_data(index, Source::Output)?;
        let actual =
            ClientReader::from_slice(&output_data).map_err(|_| Error::CreateBadClientCellData)?;
        debug!("actual new client {actual}");
        let expected = expected_client.pack();
        if actual.as_slice() != expected.as_slice() {
            return Err(Error::CreateNewClientIsIncorrect);
        }
        expected_client.id += 1;
    }

    Ok(())
}
