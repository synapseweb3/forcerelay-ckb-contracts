use alloc::vec::Vec;

#[cfg(feature = "debugging")]
use ckb_std::ckb_types::prelude::Pack as StdPack;
use ckb_std::{ckb_constants::Source, error::SysError, high_level as hl};
use eth_light_client_in_ckb_verification::types::{
    core::Hash,
    packed::{
        ClientInfoReader, ClientReader, ClientSyncCommitteeReader, SyncCommitteeUpdateReader,
    },
    prelude::*,
};

use crate::error::{InternalError, Result};

const EXPECTED_CELL_DEPS_COUNT: usize = 3;

pub(crate) fn update_sync_committee(input: usize, output: usize, script_hash: &[u8]) -> Result<()> {
    // Gets the period of the input sync committee.
    let period_of_input_sync_committee = load_input(input)?;
    // Gets the data of output sync committee.
    let output_sync_committee_data = load_output(output)?;
    // Finds the indexes of cell deps which use current script.
    let cell_deps = find_cell_deps(script_hash)?;
    // Checks the id of the cell-dep client cell and the period of the current sync committee,
    // then returns:
    // - maximal slot in the cell-dep client cell,
    // - genesis validators root in the info cell,
    // - the data of the current sync committee.
    let (maximal_slot_in_last_client, genesis_validators_root, current_sync_committee_data) =
        load_cell_deps(cell_deps, period_of_input_sync_committee)?;
    // Gets the sync comittee update from the witness.
    let sync_committee_update = {
        let witness_args = hl::load_witness_args(output, Source::Output)?;
        if let Some(args) = witness_args.input_type().to_opt() {
            SyncCommitteeUpdateReader::from_slice(&args.raw_data())
                .map_err(|_| SysError::Encoding)?
                .unpack()
        } else {
            return Err(InternalError::UpdateSyncCommitteeWitnessIsNotExisted.into());
        }
    };

    let packed_next_sync_committee =
        ClientSyncCommitteeReader::new_unchecked(&output_sync_committee_data);
    let packed_current_sync_committee =
        ClientSyncCommitteeReader::new_unchecked(&current_sync_committee_data);
    sync_committee_update.verify_packed_client_sync_committee(
        maximal_slot_in_last_client,
        genesis_validators_root,
        packed_current_sync_committee,
        packed_next_sync_committee,
    )?;

    Ok(())
}

fn load_input(input: usize) -> Result<u64> {
    debug!("load cell data of inputs[{}]", input);
    let input_data = hl::load_cell_data(input, Source::Input)?;

    let period_of_input_sync_committee: u64 =
        if let Ok(input_sync_committee) = ClientSyncCommitteeReader::from_slice(&input_data) {
            debug!(
                "input sync committee (index={input}): \
                {{ period: {}, pubkeys-length: {}, aggregate_pubkey: {} }}",
                input_sync_committee.period(),
                input_sync_committee.data().pubkeys().len(),
                input_sync_committee.data().aggregate_pubkey()
            );
            input_sync_committee.period().unpack()
        } else {
            return Err(InternalError::UpdateSyncCommitteeInputSyncCommitteeNotFound.into());
        };
    debug!("period of input sync committee = {period_of_input_sync_committee}");

    Ok(period_of_input_sync_committee)
}

fn load_output(output: usize) -> Result<Vec<u8>> {
    debug!("load cell data of outputs[{}]", output);
    let output_data = hl::load_cell_data(output, Source::Output)?;

    if let Ok(_output_sync_committee) = ClientSyncCommitteeReader::from_slice(&output_data) {
        debug!(
            "output sync committee (index={output}): \
            {{ period: {}, pubkeys-length: {}, aggregate_pubkey: {} }}",
            _output_sync_committee.period(),
            _output_sync_committee.data().pubkeys().len(),
            _output_sync_committee.data().aggregate_pubkey()
        );
        Ok(output_data)
    } else {
        Err(InternalError::UpdateSyncCommitteeOutputSyncCommitteeNotFound.into())
    }
}

fn find_cell_deps(script_hash: &[u8]) -> Result<(usize, usize, usize)> {
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
                    return Err(InternalError::UpdateSyncCommitteeCellDepsTooMany.into());
                }
            }
        }
    }
    if indexes.len() != EXPECTED_CELL_DEPS_COUNT {
        return Err(InternalError::UpdateSyncCommitteeCellDepsNotEnough.into());
    }
    Ok((indexes[0], indexes[1], indexes[2]))
}

fn load_cell_deps(
    cell_deps: (usize, usize, usize),
    period_of_input_sync_committee: u64,
) -> Result<(u64, Hash, Vec<u8>)> {
    let (info_data, client_data, sync_committee_data) = {
        debug!("load cell data of cell deps[{}]", cell_deps.0);
        let cell_dep_data_0 = hl::load_cell_data(cell_deps.0, Source::CellDep)?;
        debug!("load cell data of cell deps[{}]", cell_deps.1);
        let cell_dep_data_1 = hl::load_cell_data(cell_deps.1, Source::CellDep)?;
        debug!("load cell data of cell deps[{}]", cell_deps.2);
        let cell_dep_data_2 = hl::load_cell_data(cell_deps.2, Source::CellDep)?;

        let mut info_data_opt = None;
        let mut client_data_opt = None;
        let mut sync_committee_data_opt = None;

        for data in [cell_dep_data_0, cell_dep_data_1, cell_dep_data_2] {
            if info_data_opt.is_none() && ClientInfoReader::verify(&data, false).is_ok() {
                info_data_opt = Some(data);
                continue;
            }
            if client_data_opt.is_none() && ClientReader::verify(&data, false).is_ok() {
                client_data_opt = Some(data);
                continue;
            }
            if sync_committee_data_opt.is_none()
                && ClientSyncCommitteeReader::verify(&data, false).is_ok()
            {
                sync_committee_data_opt = Some(data);
                continue;
            }
        }
        if info_data_opt.is_none() {
            return Err(InternalError::UpdateSyncCommitteeCellDepInfoNotFound.into());
        }
        if client_data_opt.is_none() {
            return Err(InternalError::UpdateSyncCommitteeCellDepClientNotFound.into());
        }
        if sync_committee_data_opt.is_none() {
            return Err(InternalError::UpdateSyncCommitteeCellDepSyncCommitteeNotFound.into());
        }

        (
            info_data_opt.unwrap(),
            client_data_opt.unwrap(),
            sync_committee_data_opt.unwrap(),
        )
    };

    let packed_info = ClientInfoReader::new_unchecked(&info_data);
    let packed_client = ClientReader::new_unchecked(&client_data);
    let packed_sync_committee = ClientSyncCommitteeReader::new_unchecked(&sync_committee_data);
    debug!("cell-dep info = {packed_info}");
    debug!("cell-dep client = {packed_client}");
    debug!(
        "cell-dep sync committee: \
        {{ period: {}, pubkeys-length: {}, aggregate_pubkey: {} }}",
        packed_sync_committee.period(),
        packed_sync_committee.data().pubkeys().len(),
        packed_sync_committee.data().aggregate_pubkey()
    );

    let last_client_id: u8 = packed_info.last_client_id().into();
    let client_id: u8 = packed_client.id().into();
    debug!("cell-dep info.last_client_id = {last_client_id}");
    debug!("cell-dep client.id = {client_id}");

    if client_id != last_client_id {
        return Err(InternalError::UpdateSyncCommitteeCellDepClientIsNotLatest.into());
    }

    let period_of_current_sync_committee: u64 = packed_sync_committee.period().unpack();
    debug!("period of current sync committee = {period_of_current_sync_committee}");
    if period_of_current_sync_committee < period_of_input_sync_committee {
        return Err(InternalError::UpdateSyncCommitteeCellDepSyncCommitteeIsNotOldest.into());
    }

    let maximal_slot_in_last_client: u64 = packed_client.maximal_slot().unpack();
    let genesis_validators_root: Hash = packed_info.genesis_validators_root().unpack();
    debug!("maximal_slot_in_last_client = {maximal_slot_in_last_client}");
    debug!("genesis_validators_root = {genesis_validators_root:#x}");

    Ok((
        maximal_slot_in_last_client,
        genesis_validators_root,
        sync_committee_data,
    ))
}
