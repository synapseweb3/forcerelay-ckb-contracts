use ckb_ics_axon::handler::{
    handle_channel_open_init_and_try, handle_msg_connection_open_ack,
    handle_msg_connection_open_confirm, handle_msg_connection_open_init,
    handle_msg_connection_open_try, IbcChannel, IbcConnections,
};
use ckb_ics_axon::message::{
    Envelope, MsgChannelOpenAck, MsgChannelOpenConfirm, MsgChannelOpenInit, MsgChannelOpenTry,
    MsgConnectionOpenAck, MsgConnectionOpenConfirm, MsgConnectionOpenInit, MsgConnectionOpenTry,
    MsgType,
};
use ckb_ics_axon::{ChannelArgs, ConnectionArgs};
use rlp::decode;

use crate::axon_client::AxonClient;
use crate::error::{Error, Result};
use crate::utils::{load_channel_cell, load_connection_cell, load_envelope};

pub enum Navigator {
    CHECK_MESSAGE(Envelope),
    CHECK_CLIENT,
    SKIP,
}

macro_rules! handle_single_connection_msg {
    ($msgty:ty, $content:expr, $client:expr, $handler:ident) => {
        let (old_connections, old_connection_args) = load_connection_cell(0, Source::Input)?;
        let (new_connections, new_connection_args) = load_connection_cell(0, Source::Output)?;

        let msg = decode::<$msgty>($content).map_err(|_| Error::MsgEncoding)?;
        $handler(
            $client,
            old_connections,
            old_connection_args,
            new_connections,
            new_connection_args,
            msg,
        )
        .map_err(|_| Error::ConnectionProofInvalid)
    };
}

pub fn navigate_connection() -> Result<Navigator> {
    let envelope = load_envelope()?;
    match envelope.msg_type {
        // TODO: move this check code into further IBC_TYPE_ID contract, because the CONNECTION contract
        //       is located in LOCK_SCRIPT which won't execute
        MsgType::MsgClientCreate => Ok(Navigator::CHECK_CLIENT),
        MsgType::MsgConnectionOpenInit
        | MsgType::MsgConnectionOpenTry
        | MsgType::MsgConnectionOpenAck
        | MsgType::MsgConnectionOpenConfirm
        | MsgType::MsgChannelOpenInit
        | MsgType::MsgChannelOpenTry => Ok(Navigator::CHECK_MESSAGE(envelope)),
        MsgType::MsgChannelOpenAck | MsgType::MsgChannelOpenConfirm => Ok(Navigator::SKIP),
        _ => Err(Error::UnexpectedMsg),
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
        | MsgType::MsgRecvPacket => Ok(Navigator::CHECK_MESSAGE(envelope)),
        MsgType::MsgWriteAckPackt | MsgType::MsgAckPacket | MsgType::MsgTimeoutPacket => {
            Ok(Navigator::SKIP)
        }
        _ => Err(Error::UnexpectedMsg),
    }
}

pub fn verify(envelope: Envelope, client: AxonClient) -> Result<()> {
    match envelope.msg_type {
        MsgType::MsgConnectionOpenInit => {
            handle_single_connection_msg!(
                MsgConnectionOpenInit,
                envelope.content,
                client,
                handle_msg_connection_open_init
            )
        }
        MsgType::MsgConnectionOpenTry => {
            handle_single_connection_msg!(
                MsgConnectionOpenTry,
                envelope.content,
                client,
                handle_msg_connection_open_try
            )
        }
        MsgType::MsgConnectionOpenAck => {
            handle_single_connection_msg!(
                MsgConnectionOpenAck,
                envelope.content,
                client,
                handle_msg_connection_open_ack
            )
        }
        MsgType::MsgConnectionOpenConfirm => {
            handle_single_connection_msg!(
                MsgConnectionOpenConfirm,
                envelope.content,
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
            .map_err(|_| Error::ChannelProofInvalid)
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
            .map_err(|_| Error::ChannelProofInvalid)
        }
        _ => Err(Error::UnexpectedMsg),
    }
}
