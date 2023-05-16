use crate::error::{Error, Result};
use ckb_ics_axon::handler::{
    handle_channel_open_ack_and_confirm, handle_channel_open_init_and_try, IbcChannel,
    IbcConnections,
};
use ckb_ics_axon::message::{Envelope, MsgType};
use ckb_standalone_types::prelude::Entity;
use ckb_std::error::SysError;
use ckb_std::{ckb_constants::Source, high_level as hl};
use rlp::decode;
use tiny_keccak::{Hasher as _, Keccak};
use axon_client::AxonClient as Client;

pub fn main() -> Result<()> {
    let envelope = load_envelope()?;
    match envelope.msg_type {
        MsgType::MsgChannelOpenInit
        | MsgType::MsgChannelOpenTry
        | MsgType::MsgChannelOpenAck
        | MsgType::MsgChannelOpenConfirm => {}
        _ => return Ok(()),
    }

    // let client_data = hl::load_cell_data(0, Source::CellDep).map_err(|_| Error::LoadCellDataErr)?;
    // let client = decode::<Client>(&client_data).map_err(|_| Error::ClientEncoding)?;
    let client = load_client()?;

    let input_data = hl::load_cell_data(0, Source::GroupInput)?;
    let expected_input_hash: [u8; 32] = input_data
        .try_into()
        .map_err(|_| Error::ChannelHashUnmatch)?;

    let output_data = hl::load_cell_data(0, Source::GroupOutput)?;
    let expected_output_hash: [u8; 32] = output_data
        .try_into()
        .map_err(|_| Error::ChannelHashUnmatch)?;

    let witness_args0 = hl::load_witness_args(0, Source::Input)?;
    let witness_args1 = hl::load_witness_args(0, Source::Output)?;

    let old_cell_data = witness_args0.input_type();
    let new_cell_data = witness_args1.output_type();

    if old_cell_data.is_none() || new_cell_data.is_none() {
        return Err(Error::ChannelEncoding);
    }

    let old_bytes = old_cell_data.to_opt().unwrap();
    let old_slice = old_bytes.as_slice();
    if keccak256(old_slice) != expected_input_hash {
        return Err(Error::ChannelHashUnmatch);
    }

    let new_bytes = new_cell_data.to_opt().unwrap();
    let new_slice = new_bytes.as_slice();
    if keccak256(new_slice) != expected_output_hash {
        return Err(Error::ChannelHashUnmatch);
    }

    verify(client, envelope, old_slice, new_slice)
}

fn verify(client: Client, envelope: Envelope, old_data: &[u8], new_data: &[u8]) -> Result<()> {
    match &envelope.msg_type {
        MsgType::MsgChannelOpenInit | MsgType::MsgChannelOpenTry => {
            let old_connections: IbcConnections =
                decode(old_data).map_err(|_| Error::ConnectionEncoding)?;
            let new_connections: IbcConnections =
                decode(new_data).map_err(|_| Error::ConnectionEncoding)?;

            let witness_args3 = hl::load_witness_args(1, Source::Input)?;
            let channel_cell_data = witness_args3.output_type();
            if channel_cell_data.is_none() {
                return Err(Error::Encoding);
            }
            let channel_bytes = channel_cell_data.to_opt().unwrap();
            let channel_slice = channel_bytes.as_slice();
            let channel =
                decode::<IbcChannel>(channel_slice).map_err(|_| Error::ChannelEncoding)?;

            handle_channel_open_init_and_try(
                client,
                channel,
                envelope,
                old_connections,
                new_connections,
            )
            .map_err(|_| Error::ChannelProofInvalid)
        }
        MsgType::MsgChannelOpenAck | MsgType::MsgChannelOpenConfirm => {
            let old_channel: IbcChannel = decode(old_data).map_err(|_| Error::ChannelEncoding)?;
            let new_channel: IbcChannel = decode(new_data).map_err(|_| Error::ChannelEncoding)?;
            handle_channel_open_ack_and_confirm(client, envelope, old_channel, new_channel)
                .map_err(|_| Error::ChannelEncoding)
        }
        _ => Err(Error::UnexpectedMsg),
    }
}

fn keccak256(slice: &[u8]) -> [u8; 32] {
    let mut hasher = Keccak::v256();
    hasher.update(slice);
    let mut output = [0u8; 32];
    hasher.finalize(&mut output);
    output
}

#[inline]
fn load_envelope() -> Result<Envelope> {
    let witness_len = {
        if let Err(SysError::LengthNotEnough(len)) = hl::load_witness_args(99, Source::Input) {
            len
        } else {
            return Err(Error::WitnessTooMany);
        }
    };
    let last_witness = hl::load_witness_args(witness_len - 1, Source::Input)?;
    let envelope_data = last_witness.output_type();
    if envelope_data.is_none() {
        return Err(Error::WitnessIsIncorrect);
    }
    let envelope_bytes = envelope_data.to_opt().unwrap();
    let envelope_slice = envelope_bytes.as_slice();
    decode::<Envelope>(envelope_slice).map_err(|_| Error::EnvelopeEncoding)
}

fn load_client() -> Result<Client> {
    use alloc::string::ToString;
    let metadata = hl::load_cell_data(0, Source::CellDep).map_err(|_| Error::LoadCellDataErr)?;
    let metadata_type_script = hl::load_cell_type(0, Source::CellDep).map_err(|_| Error::LoadCellDataErr)?.unwrap();
    let client_id = metadata_type_script.args().to_string();
    Client::new(client_id, &metadata).map_err(|_| Error::LoadCellDataErr)
}