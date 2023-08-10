use ckb_ics_axon::handler::{IbcChannel, IbcConnections};
use ckb_ics_axon::message::Envelope;
use ckb_ics_axon::{ChannelArgs, ConnectionArgs};
use ckb_std::{ckb_constants::Source, high_level as hl};
use rlp::decode;
use tiny_keccak::{Hasher as _, Keccak};

use crate::axon_client::AxonClient;
use crate::error::{Error, Result};

pub fn keccak256(slice: &[u8]) -> [u8; 32] {
    let mut hasher = Keccak::v256();
    hasher.update(slice);
    let mut output = [0u8; 32];
    hasher.finalize(&mut output);
    output
}

pub fn load_connection_cell(
    idx: usize,
    source: Source,
) -> Result<(IbcConnections, ConnectionArgs)> {
    let lock = hl::load_cell_lock(idx, source).map_err(|_| Error::ConnectionLock)?;
    let lock_args = lock.args().raw_data();
    let connection_args =
        ConnectionArgs::from_slice(&lock_args).map_err(|_| Error::ConnectionLock)?;

    let witness_args = hl::load_witness_args(idx, source)?;
    let witness_data = if source == Source::Input {
        witness_args.input_type()
    } else {
        witness_args.output_type()
    };

    let cell_data = hl::load_cell_data(idx, source)?;
    let expected_hash: [u8; 32] = cell_data.try_into().map_err(|_| Error::CellDataUnmatch)?;

    if witness_data.is_none() {
        return Err(Error::WitnessInputOrOutputIsNone);
    }

    let witness_bytes = witness_data.to_opt().unwrap();
    let witness_slice = witness_bytes.raw_data();

    if keccak256(&witness_slice) != expected_hash {
        return Err(Error::ConnectionHashUnmatch);
    }

    let connection: IbcConnections =
        decode(&witness_slice).map_err(|_| Error::ConnectionEncoding)?;
    Ok((connection, connection_args))
}

pub fn load_channel_cell(idx: usize, source: Source) -> Result<(IbcChannel, ChannelArgs)> {
    let lock = hl::load_cell_lock(idx, source).map_err(|_| Error::ChannelLock)?;
    let lock_args = lock.args().raw_data();
    let channel_args = ChannelArgs::from_slice(&lock_args).map_err(|_| Error::ChannelLock)?;

    let witness_args = hl::load_witness_args(idx, source)?;
    let witness_data = if source == Source::Input {
        witness_args.input_type()
    } else {
        witness_args.output_type()
    };

    let cell_data = hl::load_cell_data(idx, source)?;
    let expected_hash: [u8; 32] = cell_data.try_into().map_err(|_| Error::CellDataUnmatch)?;

    if witness_data.is_none() {
        return Err(Error::WitnessInputOrOutputIsNone);
    }

    let witness_bytes = witness_data.to_opt().unwrap();
    let witness_slice = witness_bytes.raw_data();

    if keccak256(&witness_slice) != expected_hash {
        return Err(Error::ChannelHashUnmatch);
    }

    let channel: IbcChannel = decode(&witness_slice).map_err(|_| Error::ChannelEncoding)?;
    Ok((channel, channel_args))
}

pub fn load_envelope() -> Result<Envelope> {
    let witness_len = hl::load_transaction()?.witnesses().len();
    let last_witness = hl::load_witness_args(witness_len - 1, Source::Input)?;
    let envelope_data = last_witness.output_type();
    if envelope_data.is_none() {
        return Err(Error::WitnessIsIncorrect);
    }
    let envelope_bytes = envelope_data.to_opt().unwrap();
    let envelope_slice = &envelope_bytes.raw_data();
    decode::<Envelope>(envelope_slice).map_err(|_| Error::EnvelopeEncoding)
}

pub fn load_client() -> Result<AxonClient> {
    let metadata =
        hl::load_cell_data(0, Source::CellDep).map_err(|_| Error::FailedToLoadClientCellData)?;
    let metadata_type_script = hl::load_cell_type(0, Source::CellDep)
        .map_err(|_| Error::FailedToLoadClientTypeScript)?
        .unwrap();
    let client_data = metadata_type_script.args().raw_data();
    let client_id: [u8; 32] = client_data
        .as_ref()
        .try_into()
        .map_err(|_| Error::FailedToLoadClientId)?;
    AxonClient::new(client_id, &metadata).map_err(|_| Error::FailedToCreateClient)
}
