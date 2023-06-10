use crate::error::{Error, Result};
use axon_client::AxonClient as Client;
use ckb_ics_axon::handler::{
    handle_channel_open_ack_and_confirm, handle_channel_open_init_and_try, IbcChannel,
    IbcConnections,
};
use ckb_ics_axon::message::{Envelope, MsgType};
use ckb_ics_axon::{ChannelArgs, ConnectionArgs};
use ckb_standalone_types::prelude::Entity;
use ckb_std::{ckb_constants::Source, high_level as hl};
use rlp::decode;
use tiny_keccak::{Hasher as _, Keccak};

pub fn main() -> Result<()> {
    let envelope = load_envelope()?;
    match envelope.msg_type {
        MsgType::MsgChannelOpenInit
        | MsgType::MsgChannelOpenTry
        | MsgType::MsgChannelOpenAck
        | MsgType::MsgChannelOpenConfirm => {}
        _ => return Ok(()),
    }

    let client = load_client()?;

    let input_data = hl::load_cell_data(0, Source::Input)?;
    let expected_input_hash: [u8; 32] = input_data
        .try_into()
        .map_err(|_| Error::ChannelHashUnmatch)?;

    let output_data = hl::load_cell_data(0, Source::Output)?;
    let expected_output_hash: [u8; 32] = output_data
        .try_into()
        .map_err(|_| Error::ChannelHashUnmatch)?;

    let witness_args0 = hl::load_witness_args(0, Source::Input)?;

    let old_cell_data = witness_args0.input_type();
    let new_cell_data = witness_args0.output_type();

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

            let new_lock =
                hl::load_cell_lock(0, Source::Output).map_err(|_| Error::ConnectionLock)?;
            let new_lock_args = new_lock.args();
            let new_connection_args = ConnectionArgs::from_slice(new_lock_args.as_slice())
                .map_err(|_| Error::ConnectionLock)?;

            let old_lock =
                hl::load_cell_lock(0, Source::Input).map_err(|_| Error::ConnectionLock)?;
            let old_lock_args = old_lock.args();
            let old_connection_args = ConnectionArgs::from_slice(old_lock_args.as_slice())
                .map_err(|_| Error::ConnectionLock)?;

            let channel_lock = hl::load_cell_lock(1, Source::Output)?;
            let channel_lock_args = channel_lock.args();
            let channel_args = ChannelArgs::from_slice(channel_lock_args.as_slice())
                .map_err(|_| Error::ChannelLock)?;

            handle_channel_open_init_and_try(
                client,
                channel,
                channel_args,
                envelope,
                old_connections,
                old_connection_args,
                new_connections,
                new_connection_args,
            )
            .map_err(|_| Error::ChannelProofInvalid)
        }
        MsgType::MsgChannelOpenAck | MsgType::MsgChannelOpenConfirm => {
            let old_channel: IbcChannel = decode(old_data).map_err(|_| Error::ChannelEncoding)?;
            let new_channel: IbcChannel = decode(new_data).map_err(|_| Error::ChannelEncoding)?;
            let new_lock = hl::load_cell_lock(0, Source::Output).map_err(|_| Error::ChannelLock)?;
            let new_lock_args = new_lock.args();
            let new_channel_args = ChannelArgs::from_slice(new_lock_args.as_slice())
                .map_err(|_| Error::ChannelLock)?;

            let old_lock = hl::load_cell_lock(0, Source::Input).map_err(|_| Error::ChannelLock)?;
            let old_lock_args = old_lock.args();
            let old_channel_args = ChannelArgs::from_slice(old_lock_args.as_slice())
                .map_err(|_| Error::ChannelLock)?;
            handle_channel_open_ack_and_confirm(
                client,
                envelope,
                old_channel,
                old_channel_args,
                new_channel,
                new_channel_args,
            )
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

fn load_client() -> Result<Client> {
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
    Client::new(client_id, &metadata).map_err(|_| Error::FailedToCreateClient)
}
