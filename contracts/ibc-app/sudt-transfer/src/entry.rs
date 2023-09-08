use alloc::{format, string::String, vec::Vec};

use ckb_std::{
    ckb_constants::Source,
    ckb_types::prelude::Reader,
    high_level::{
        load_cell, load_cell_data, load_cell_lock_hash, load_cell_type_hash, load_input,
        load_script, load_script_hash, QueryIter,
    },
};
use ics_base::{
    ckb_ics::{handler::IbcPacket, message::MsgType},
    utils::{load_envelope, load_packet_cell},
};
use prost::Message;

use crate::error::Error;

pub fn main() -> Result<(), Error> {
    let script = load_script()?;
    let args = script.as_reader().args().raw_data();
    let args = Args::decode(args)?;

    if let Ok(envelope) = load_envelope() {
        match envelope.msg_type {
            MsgType::MsgSendPacket => {
                verify_channel_input(args)?;
                let packet = load_packet_cell(1, Source::Output)?.0;
                verify_send_packet(packet)
            }
            MsgType::MsgWriteAckPacket => {
                verify_channel_input(args)?;
                let packet = load_packet_cell(1, Source::Output)?.0;
                verify_write_ack_packet(packet)
            }
            MsgType::MsgConsumeAckPacket => {
                verify_packet_input(args)?;
                let packet = load_packet_cell(0, Source::Input)?.0;
                verify_consume_ack_packet(packet)
            }
            _ => Err(Error::InvalidMsgType),
        }
    } else {
        todo!()
    }
}

/// Check that condition is true, otherwise return the error.
fn ensure(cond: bool, error: Error) -> Result<(), Error> {
    if !cond {
        Err(error)
    } else {
        Ok(())
    }
}

/// (Receiving SUDT from counterparty ICS20 module.)
///
/// Verify the amount/type/receiver of SUDT transferred from this lock is the
/// same as the packet data.
///
/// The ack message must be vec![1].
///
/// What about failure ack? We must verify that it's impossible to withdrawal
/// this amount of this type of SUDT or there's a possibility of DoS. Maybe we
/// can rely on timeout and do not explicitly acknowledge failure.
fn verify_write_ack_packet(packet: IbcPacket) -> Result<(), Error> {
    ensure(packet.ack.as_deref() == Some(&[1]), Error::InvalidAck)?;
    let packet_data =
        FungibleTokenPacketData::decode(&packet.packet.data[..]).map_err(|_| Error::PacketData)?;
    let sudt_io = load_sudt_io()?;
    ensure(
        sudt_io.input_amount.checked_sub(packet_data.amount.into()) == Some(sudt_io.output_amount),
        Error::SudtAmount,
    )?;
    // CKB must be source chain, so the denom must be prefixed.
    let denom_prefix = format!(
        "{}/{}/",
        packet.packet.source_port_id, packet.packet.source_channel_id
    );
    let base_denom = packet_data
        .denom
        .strip_prefix(&denom_prefix)
        .ok_or(Error::Denom)?;
    verify_sudt_type_and_denom(&sudt_io.type_script_hash, base_denom)?;
    verify_ckb_sender_or_receiver(&packet_data.receiver)?;

    Ok(())
}

/// (Sending SUDT to counterparty ICS20 module.)
///
/// Verify the sender/amount/type of SUDT transferred to this lock is the same
/// as the packet data.
fn verify_send_packet(packet: IbcPacket) -> Result<(), Error> {
    let packet_data =
        FungibleTokenPacketData::decode(&packet.packet.data[..]).map_err(|_| Error::PacketData)?;
    let sudt_io = load_sudt_io()?;
    ensure(
        sudt_io.input_amount.checked_add(packet_data.amount.into()) == Some(sudt_io.output_amount),
        Error::SudtAmount,
    )?;
    // CKB must be source chain and denom must be base denom. A prefixed denom
    // will not pass this check.
    verify_sudt_type_and_denom(&sudt_io.type_script_hash, &packet_data.denom)?;
    verify_ckb_sender_or_receiver(&packet_data.sender)?;

    Ok(())
}

/// (Handling ack for sending SUDT.)
///
/// For a success ACK, verify that SUDT in this lock isn't changed.
///
/// For a failure ACK, verify that the amount/type of SUDT transferred from this
/// lock is the same as the original packet data.
fn verify_consume_ack_packet(packet: IbcPacket) -> Result<(), Error> {
    let packet_data =
        FungibleTokenPacketData::decode(&packet.packet.data[..]).map_err(|_| Error::PacketData)?;
    let sudt_io = load_sudt_io()?;

    if packet.ack.as_deref() == Some(&[0]) {
        // Failure ack: refund sender.
        verify_ckb_sender_or_receiver(&packet_data.sender)?;
        ensure(
            sudt_io.input_amount.checked_sub(packet_data.amount.into())
                == Some(sudt_io.output_amount),
            Error::SudtAmount,
        )?;
    } else {
        // Success ack.
        ensure(
            sudt_io.input_amount == sudt_io.output_amount,
            Error::SudtAmount,
        )?;
    }
    verify_sudt_type_and_denom(&sudt_io.type_script_hash, &packet_data.denom)?;

    Ok(())
}

fn verify_channel_input(_args: Args<'_>) -> Result<(), Error> {
    // TODO: check channel type id input.
    Ok(())
}

fn verify_packet_input(_args: Args<'_>) -> Result<(), Error> {
    // TODO: check packet type id input.
    Ok(())
}

struct SudtIo {
    type_script_hash: [u8; 32],
    input_amount: u128,
    output_amount: u128,
}

/// Verify and get SUDT input/output type and amount of this lock.
///
/// Also verifies that cell capacity doesn't change.
fn load_sudt_io() -> Result<SudtIo, Error> {
    let self_lock_hash = load_script_hash()?;

    // For now, allow only one input in this input group
    ensure(load_input(1, Source::GroupInput).is_err(), Error::Input)?;

    let mut outputs = QueryIter::new(load_cell_lock_hash, Source::Output)
        .enumerate()
        .filter(|(_, h)| *h == self_lock_hash);
    let first_output_idx = outputs.next().ok_or(Error::Output)?.0;
    // Only allow a single output for now.
    ensure(outputs.next().is_none(), Error::Output)?;

    let input_cell = load_cell(0, Source::GroupInput)?;
    let output_cell = load_cell(first_output_idx, Source::Output)?;

    let type_script_hash = load_cell_type_hash(0, Source::GroupInput)?.ok_or(Error::Input)?;

    // Output should have the same lock/type/capacity.
    ensure(
        input_cell.as_reader().as_slice() == output_cell.as_reader().as_slice(),
        Error::Output,
    )?;

    let input_data = load_cell_data(0, Source::GroupInput)?;
    let input_amount = u128::from_le_bytes(
        input_data
            .get(..16)
            .ok_or(Error::Input)?
            .try_into()
            .unwrap(),
    );

    let output_data = load_cell_data(first_output_idx, Source::Output)?;
    let output_amount = u128::from_le_bytes(
        output_data
            .get(..16)
            .ok_or(Error::Output)?
            .try_into()
            .unwrap(),
    );

    Ok(SudtIo {
        type_script_hash,
        input_amount,
        output_amount,
    })
}

pub struct Args<'a> {
    pub client_id: &'a [u8; 32],
    pub channel_id: u16,
    pub channel_contract_code_hash: &'a [u8; 32],
    pub packet_contract_code_hash: &'a [u8; 32],
}

macro_rules! try_read {
    ($buf:ident, $len:literal) => {{
        let x: &[u8; $len] = $buf
            .get(..$len)
            .ok_or(Error::InvalidArgs)?
            .try_into()
            .unwrap();
        $buf = &$buf[$len..];
        x
    }};
}

impl<'a> Args<'a> {
    pub fn decode(mut args: &'a [u8]) -> Result<Self, Error> {
        let client_id = try_read!(args, 32);
        let channel_id = u16::from_be_bytes(*try_read!(args, 2));
        let channel_contract_code_hash = try_read!(args, 32);
        let packet_contract_code_hash = try_read!(args, 32);
        ensure(args.is_empty(), Error::InvalidArgs)?;
        Ok(Self {
            client_id,
            channel_id,
            channel_contract_code_hash,
            packet_contract_code_hash,
        })
    }

    #[allow(unused)]
    pub fn encode(&self) -> Vec<u8> {
        [
            self.client_id,
            &u16::to_be_bytes(self.channel_id)[..],
            self.channel_contract_code_hash,
            self.packet_contract_code_hash,
        ]
        .concat()
    }
}

#[derive(Message)]
pub struct FungibleTokenPacketData {
    /// hex(sudt type script)
    #[prost(string, tag = "1")]
    pub denom: String,
    /// SUDT amount.
    #[prost(uint64, tag = "2")]
    pub amount: u64,
    /// For ckb address, this should be ckb_blake2b(packed lock script)[..20]
    #[prost(bytes, tag = "3")]
    pub sender: Vec<u8>,
    /// For ckb address, this should be ckb_blake2b(packed lock script)[..20]
    #[prost(bytes, tag = "4")]
    pub receiver: Vec<u8>,
}

fn verify_sudt_type_and_denom(type_script_hash: &[u8; 32], denom: &str) -> Result<(), Error> {
    let mut out = [0u8; 32];
    hex::decode_to_slice(denom, &mut out).map_err(|_| Error::SudtAmount)?;
    ensure(&out == type_script_hash, Error::Denom)?;
    Ok(())
}

/// Verify that there's an input with a matching lock script.
///
/// The address should be ckb_blake2b(packed lock script)[..20].
fn verify_ckb_sender_or_receiver(address: &[u8]) -> Result<(), Error> {
    ensure(address.len() == 20, Error::SenderReceiver)?;

    let found =
        QueryIter::new(load_cell_lock_hash, Source::Input).any(|lh| lh.starts_with(address));

    ensure(found, Error::SenderReceiver)?;

    Ok(())
}
