use axon_client::AxonClient as Client;
use ckb_ics_axon::handler::{
    handle_msg_ack_inbox_packet, handle_msg_ack_outbox_packet, handle_msg_ack_packet,
    handle_msg_recv_packet, handle_msg_send_packet, IbcChannel, IbcPacket,
};
use ckb_ics_axon::message::{
    Envelope, MsgAckInboxPacket, MsgAckOutboxPacket, MsgAckPacket, MsgType,
};
use ckb_ics_axon::message::{MsgRecvPacket, MsgSendPacket};
use ckb_ics_axon::{ChannelArgs, PacketArgs};
use ckb_standalone_types::prelude::Entity;
use rlp::decode;
use tiny_keccak::{Hasher as _, Keccak};

use ckb_std::error::SysError;
use ckb_std::{ckb_constants::Source, high_level as hl};

use crate::error::{Error, Result};

// We have these conventions in this contract:
// - The last witnesses args's output type is the serialization of Packet Message
// - The 1st cell dep's data is Client
pub fn main() -> Result<()> {
    let envelope = load_envelope()?;
    match &envelope.msg_type {
        MsgType::MsgSendPacket => {
            let client = load_client()?;

            let (old_channel, old_channel_args) =
                load_and_validate_channel_from_idx(0, Source::Input)?;
            let (new_channel, new_channel_args) =
                load_and_validate_channel_from_idx(0, Source::Output)?;
            let (ibc_packet, packet_args) =
                load_and_validate_ibc_packet_from_idx(1, Source::Output)?;

            let msg = decode::<MsgSendPacket>(&envelope.content).map_err(|_| Error::MsgEncoding)?;
            handle_msg_send_packet(
                client,
                old_channel,
                old_channel_args,
                new_channel,
                new_channel_args,
                ibc_packet,
                packet_args,
                msg,
            )
            .map_err(|_| Error::PacketProofInvalid)
        }
        MsgType::MsgRecvPacket => {
            let client = load_client()?;

            let (old_channel, old_channel_args) =
                load_and_validate_channel_from_idx(0, Source::Input)?;
            let (new_channel, new_channel_args) =
                load_and_validate_channel_from_idx(0, Source::Output)?;
            let (ibc_packet, packet_args) =
                load_and_validate_ibc_packet_from_idx(1, Source::Output)?;
            let msg = decode::<MsgRecvPacket>(&envelope.content).map_err(|_| Error::Encoding)?;
            handle_msg_recv_packet(
                client,
                old_channel,
                old_channel_args,
                new_channel,
                new_channel_args,
                ibc_packet,
                packet_args,
                msg,
            )
            .map_err(|_| Error::PacketProofInvalid)
        }
        MsgType::MsgAckOutboxPacket => {
            let (old_ibc_packet, old_packet_args) =
                load_and_validate_ibc_packet_from_idx(0, Source::Input)?;
            let (new_ibc_packet, new_packet_args) =
                load_and_validate_ibc_packet_from_idx(0, Source::Output)?;
            let msg =
                decode::<MsgAckOutboxPacket>(&envelope.content).map_err(|_| Error::MsgEncoding)?;
            handle_msg_ack_outbox_packet(
                old_ibc_packet,
                old_packet_args,
                new_ibc_packet,
                new_packet_args,
                msg,
            )
            .map_err(|_| Error::PacketProofInvalid)
        }
        MsgType::MsgAckInboxPacket => {
            let (old_ibc_packet, _packet_args) =
                load_and_validate_ibc_packet_from_idx(0, Source::Input)?;
            let msg =
                decode::<MsgAckInboxPacket>(&envelope.content).map_err(|_| Error::MsgEncoding)?;
            handle_msg_ack_inbox_packet(old_ibc_packet, msg).map_err(|_| Error::PacketProofInvalid)
        }
        MsgType::MsgAckPacket => {
            let client = load_client()?;

            let (old_channel, old_channel_args) =
                load_and_validate_channel_from_idx(0, Source::Input)?;
            let (new_channel, new_channel_args) =
                load_and_validate_channel_from_idx(0, Source::Output)?;
            let (old_ibc_packet, old_packet_args) =
                load_and_validate_ibc_packet_from_idx(1, Source::Input)?;
            let (new_ibc_packet, new_packet_args) =
                load_and_validate_ibc_packet_from_idx(1, Source::Output)?;

            let msg = decode::<MsgAckPacket>(&envelope.content).map_err(|_| Error::MsgEncoding)?;
            handle_msg_ack_packet(
                client,
                old_channel,
                old_channel_args,
                new_channel,
                new_channel_args,
                old_ibc_packet,
                old_packet_args,
                new_ibc_packet,
                new_packet_args,
                msg,
            )
            .map_err(|_| Error::PacketProofInvalid)
        }
        _ => Err(Error::UnexpectedMsg),
    }
}

#[inline]
fn load_and_validate_channel_from_idx(
    idx: usize,
    source: Source,
) -> Result<(IbcChannel, ChannelArgs)> {
    let witness = hl::load_witness_args(idx, source)?;
    let cell_data = witness.input_type();
    if cell_data.is_none() {
        return Err(Error::ChannelEncoding);
    }

    let lock = hl::load_cell_lock(idx, source)?;
    let lock_args = lock.args();
    let channel_args =
        ChannelArgs::from_slice(lock_args.as_slice()).map_err(|_| Error::ChannelLock)?;

    let bytes = cell_data.to_opt().unwrap();
    let slice = bytes.as_slice();

    let data = hl::load_cell_data(0, source)?;
    let expected_hash: [u8; 32] = data.try_into().map_err(|_| Error::ChannelHashUnmatch)?;

    if keccak256(slice) != expected_hash {
        return Err(Error::ChannelHashUnmatch);
    }
    let channel = decode_channel_cell(slice)?;
    Ok((channel, channel_args))
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

#[inline]
fn load_and_validate_ibc_packet_from_idx(
    idx: usize,
    source: Source,
) -> Result<(IbcPacket, PacketArgs)> {
    let witness = hl::load_witness_args(idx, source)?;

    let cell_data = if source == Source::Input {
        witness.input_type()
    } else {
        witness.output_type()
    };

    if cell_data.is_none() {
        return Err(Error::WitnessIsNotExisted);
    }

    let bytes = cell_data.to_opt().unwrap();
    let slice = bytes.as_slice();

    let data = hl::load_cell_data(1, source)?;
    let expected_hash: [u8; 32] = data.try_into().map_err(|_| Error::ChannelHashUnmatch)?;

    if keccak256(slice) != expected_hash {
        return Err(Error::ChannelHashUnmatch);
    }

    let lock = hl::load_cell_lock(idx, source)?;
    let lock_args = lock.args();
    let packet_args =
        PacketArgs::from_slice(lock_args.as_slice()).map_err(|_| Error::PacketLock)?;
    let packet = decode_ibc_packet(slice)?;
    Ok((packet, packet_args))
}

#[inline]
fn decode_ibc_packet(bytes: &[u8]) -> Result<IbcPacket> {
    decode(bytes).map_err(|_| Error::PacketEncoding)
}

#[inline]
fn decode_channel_cell(bytes: &[u8]) -> Result<IbcChannel> {
    decode(bytes).map_err(|_| Error::ChannelEncoding)
}

fn keccak256(slice: &[u8]) -> [u8; 32] {
    let mut hasher = Keccak::v256();
    hasher.update(slice);
    let mut output = [0u8; 32];
    hasher.finalize(&mut output);
    output
}

fn load_client() -> Result<Client> {
    let metadata = hl::load_cell_data(0, Source::CellDep).map_err(|_| Error::LoadCellDataErr)?;
    let metadata_type_script = hl::load_cell_type(0, Source::CellDep)
        .map_err(|_| Error::LoadCellDataErr)?
        .unwrap();
    let client_id: [u8; 32] = metadata_type_script
        .args()
        .as_slice()
        .try_into()
        .map_err(|_| Error::LoadCellDataErr)?;
    Client::new(client_id, &metadata).map_err(|_| Error::LoadCellDataErr)
}
