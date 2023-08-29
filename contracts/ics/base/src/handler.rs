use ckb_ics_axon::handler::*;
use ckb_ics_axon::message::*;
use ckb_std::ckb_constants::Source;
use rlp::decode;

use crate::axon_client::AxonClient;
use crate::error::{CkbResult, Error, Result};
use crate::utils::{
    check_valid_port_id, load_channel_cell, load_connection_cell, load_envelope, load_packet_cell,
};

pub enum Navigator {
    CheckMessage(Envelope),
    CheckClient,
    Skip,
}

pub fn navigate_connection() -> Result<Navigator> {
    let envelope = load_envelope()?;
    match envelope.msg_type {
        // TODO: move this check code into further IBC_TYPE_ID contract, because the CONNECTION contract
        //       is located in LOCK_SCRIPT which won't execute
        MsgType::MsgClientCreate => Ok(Navigator::CheckClient),
        MsgType::MsgConnectionOpenInit
        | MsgType::MsgConnectionOpenTry
        | MsgType::MsgConnectionOpenAck
        | MsgType::MsgConnectionOpenConfirm
        | MsgType::MsgChannelOpenInit
        | MsgType::MsgChannelOpenTry => Ok(Navigator::CheckMessage(envelope)),
        MsgType::MsgChannelOpenAck | MsgType::MsgChannelOpenConfirm => Ok(Navigator::Skip),
        _ => Err(Error::UnexpectedConnectionMsg),
    }
}

pub fn navigate_channel() -> Result<Navigator> {
    let envelope = load_envelope()?;
    match envelope.msg_type {
        MsgType::MsgChannelOpenInit
        | MsgType::MsgChannelOpenTry
        | MsgType::MsgChannelOpenAck
        | MsgType::MsgChannelOpenConfirm
        | MsgType::MsgSendPacket
        | MsgType::MsgRecvPacket => Ok(Navigator::CheckMessage(envelope)),
        MsgType::MsgWriteAckPacket | MsgType::MsgAckPacket | MsgType::MsgTimeoutPacket => {
            Ok(Navigator::Skip)
        }
        _ => Err(Error::UnexpectedChannelMsg),
    }
}

pub fn navigate_packet() -> Result<Navigator> {
    let envelope = load_envelope()?;
    match envelope.msg_type {
        MsgType::MsgWriteAckPacket | MsgType::MsgAckPacket => Ok(Navigator::CheckMessage(envelope)),
        _ => Err(Error::UnexpectedPacketMsg),
    }
}

macro_rules! handle_single_connection_msg {
    ($msgty:ty, $content:expr, $client:ident, $handler:ident) => {{
        let (old_connections, old_connection_args) = load_connection_cell(0, Source::Input)?;
        let (new_connections, new_connection_args) = load_connection_cell(0, Source::Output)?;

        let msg: $msgty = decode($content).map_err(|_| Error::MsgEncoding)?;
        $handler(
            $client,
            old_connections,
            old_connection_args,
            new_connections,
            new_connection_args,
            msg,
        )
        .map_err(Into::into)
    }};
}

pub fn verify(envelope: Envelope, client: AxonClient) -> CkbResult<()> {
    match envelope.msg_type {
        MsgType::MsgConnectionOpenInit => {
            handle_single_connection_msg!(
                MsgConnectionOpenInit,
                &envelope.content,
                client,
                handle_msg_connection_open_init
            )
        }
        MsgType::MsgConnectionOpenTry => {
            handle_single_connection_msg!(
                MsgConnectionOpenTry,
                &envelope.content,
                client,
                handle_msg_connection_open_try
            )
        }
        MsgType::MsgConnectionOpenAck => {
            handle_single_connection_msg!(
                MsgConnectionOpenAck,
                &envelope.content,
                client,
                handle_msg_connection_open_ack
            )
        }
        MsgType::MsgConnectionOpenConfirm => {
            handle_single_connection_msg!(
                MsgConnectionOpenConfirm,
                &envelope.content,
                client,
                handle_msg_connection_open_confirm
            )
        }
        MsgType::MsgChannelOpenInit | MsgType::MsgChannelOpenTry => {
            let (old_connections, old_connection_args) = load_connection_cell(0, Source::Input)?;
            let (new_connections, new_connection_args) = load_connection_cell(0, Source::Output)?;
            let (new_channel, new_channel_args) = load_channel_cell(1, Source::Output)?;

            handle_channel_open_init_and_try(
                client,
                new_channel,
                new_channel_args,
                envelope,
                old_connections,
                old_connection_args,
                new_connections,
                new_connection_args,
            )
            .map_err(Into::into)
        }
        MsgType::MsgChannelOpenAck | MsgType::MsgChannelOpenConfirm => {
            let (old_channel, old_channel_args) = load_channel_cell(0, Source::Input)?;
            let (new_channel, new_channel_args) = load_channel_cell(0, Source::Output)?;

            handle_channel_open_ack_and_confirm(
                client,
                envelope,
                old_channel,
                old_channel_args,
                new_channel,
                new_channel_args,
            )
            .map_err(Into::into)
        }
        MsgType::MsgSendPacket => {
            let (old_channel, old_channel_args) = load_channel_cell(0, Source::Input)?;
            let (new_channel, new_channel_args) = load_channel_cell(0, Source::Output)?;
            let (ibc_packet, packet_args) = load_packet_cell(1, Source::Output)?;
            check_valid_port_id(&packet_args.port_id)?;

            let msg: MsgSendPacket = decode(&envelope.content).map_err(|_| Error::MsgEncoding)?;
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
            .map_err(Into::into)
        }
        MsgType::MsgRecvPacket | MsgType::MsgTimeoutPacket => {
            let (old_channel, old_channel_args) = load_channel_cell(0, Source::Input)?;
            let (new_channel, new_channel_args) = load_channel_cell(0, Source::Output)?;
            let useless_ibc_packet =
                if let Ok((useless_packet, _)) = load_packet_cell(1, Source::Input) {
                    Some(useless_packet)
                } else {
                    None
                };
            let (ibc_packet, packet_args) = load_packet_cell(1, Source::Output)?;

            let msg: MsgRecvPacket = decode(&envelope.content).map_err(|_| Error::Encoding)?;
            handle_msg_recv_packet(
                client,
                old_channel,
                old_channel_args,
                new_channel,
                new_channel_args,
                useless_ibc_packet,
                ibc_packet,
                packet_args,
                msg,
            )
            .map_err(Into::into)
        }
        MsgType::MsgWriteAckPacket => {
            let (old_ibc_packet, old_packet_args) = load_packet_cell(1, Source::Input)?;
            let (new_ibc_packet, new_packet_args) = load_packet_cell(1, Source::Output)?;
            check_valid_port_id(&old_packet_args.port_id)?;

            let msg: MsgWriteAckPacket =
                decode(&envelope.content).map_err(|_| Error::MsgEncoding)?;
            handle_msg_write_ack_packet(
                old_ibc_packet,
                old_packet_args,
                new_ibc_packet,
                new_packet_args,
                msg,
            )
            .map_err(Into::into)
        }
        MsgType::MsgAckPacket => {
            let (old_channel, old_channel_args) = load_channel_cell(0, Source::Input)?;
            let (new_channel, new_channel_args) = load_channel_cell(0, Source::Output)?;
            let (old_ibc_packet, old_packet_args) = load_packet_cell(1, Source::Input)?;
            let (new_ibc_packet, new_packet_args) = load_packet_cell(1, Source::Output)?;

            let msg: MsgAckPacket = decode(&envelope.content).map_err(|_| Error::MsgEncoding)?;
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
            .map_err(Into::into)
        }
        MsgType::MsgConsumeAckPacket => {
            let (old_channel, old_channel_args) = load_channel_cell(0, Source::Input)?;
            check_valid_port_id(&old_packet_args.port_id)?;

            let msg: MsgConsumeAckPacket =
                decode(&envelope.content).map_err(|_| Error::MsgEncoding)?;
            handle_msg_consume_ack_packet(old_channel, old_channel_args, msg).map_err(Into::into)
        }
    }
}
