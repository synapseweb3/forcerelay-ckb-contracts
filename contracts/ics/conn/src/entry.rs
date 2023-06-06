use ckb_ics_axon::handler::Client;
use ckb_ics_axon::handler::{
    handle_msg_connection_open_ack, handle_msg_connection_open_confirm,
    handle_msg_connection_open_init, handle_msg_connection_open_try, IbcConnections,
};
use ckb_ics_axon::message::{
    Envelope, MsgConnectionOpenAck, MsgConnectionOpenConfirm, MsgConnectionOpenInit,
    MsgConnectionOpenTry, MsgType,
};
use ckb_ics_axon::ConnectionArgs;

use axon_client::AxonClient;
use rlp::decode;
use tiny_keccak::{Hasher as _, Keccak};

use ckb_std::{ckb_constants::Source, high_level as hl};

use crate::error::{Error, Result};

pub fn main() -> Result<()> {
    let envelope = load_envelope()?;
    match &envelope.msg_type {
        MsgType::MsgConnectionOpenInit
        | MsgType::MsgConnectionOpenTry
        | MsgType::MsgConnectionOpenAck
        | MsgType::MsgConnectionOpenConfirm => {}
        MsgType::MsgChannelOpenInit
        | MsgType::MsgChannelOpenTry
        | MsgType::MsgChannelOpenAck
        | MsgType::MsgChannelOpenConfirm => return Ok(()),
        MsgType::MsgClientCreate => return check_create(),
        _ => return Err(Error::UnexpectedMsg),
    }

    let client = load_client()?;

    let (old_connection_cell, old_connection_args) = load_connection_cell(0, Source::Input)?;
    let (new_connection_cell, new_connection_args) = load_connection_cell(0, Source::Output)?;
    verify(
        old_connection_cell,
        old_connection_args,
        new_connection_cell,
        new_connection_args,
        envelope,
        client,
    )?;

    Ok(())
}

fn check_create() -> Result<()> {
    let client = load_client()?;

    let (connection_cell, connection_args) = load_connection_cell(0, Source::Output)?;

    if connection_args.client_id.as_slice() != client.client_id() {
        return Err(Error::ClientCreateWrongClientId);
    }

    if !connection_cell.connections.is_empty()
        || connection_cell.next_channel_number != 0
        || connection_cell.next_connection_number != 0
    {
        return Err(Error::ClientCreateWrongConnectionCell);
    }

    Ok(())
}

fn verify(
    old: IbcConnections,
    old_args: ConnectionArgs,
    new: IbcConnections,
    new_args: ConnectionArgs,
    envelope: Envelope,
    client: AxonClient,
) -> Result<()> {
    match envelope.msg_type {
        MsgType::MsgConnectionOpenInit => {
            let msg = decode::<MsgConnectionOpenInit>(&envelope.content)
                .map_err(|_| Error::MsgEncoding)?;
            handle_msg_connection_open_init(client, old, old_args, new, new_args, msg)
                .map_err(|_| Error::ConnectionProofInvalid)
        }
        MsgType::MsgConnectionOpenTry => {
            let msg = decode::<MsgConnectionOpenTry>(&envelope.content)
                .map_err(|_| Error::MsgEncoding)?;
            handle_msg_connection_open_try(client, old, old_args, new, new_args, msg)
                .map_err(|_| Error::ConnectionProofInvalid)
        }
        MsgType::MsgConnectionOpenAck => {
            let msg = decode::<MsgConnectionOpenAck>(&envelope.content)
                .map_err(|_| Error::MsgEncoding)?;
            handle_msg_connection_open_ack(client, old, old_args, new, new_args, msg)
                .map_err(|_| Error::ConnectionProofInvalid)
        }
        MsgType::MsgConnectionOpenConfirm => {
            let msg = decode::<MsgConnectionOpenConfirm>(&envelope.content)
                .map_err(|_| Error::MsgEncoding)?;
            handle_msg_connection_open_confirm(client, old, old_args, new, new_args, msg)
                .map_err(|_| Error::ConnectionProofInvalid)
        }
        _ => Err(Error::UnexpectedMsg),
    }
}

fn load_connection_cell(idx: usize, source: Source) -> Result<(IbcConnections, ConnectionArgs)> {
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

    let connection = decode_connection_cell(&witness_slice)?;
    Ok((connection, connection_args))
}

#[inline]
fn decode_connection_cell(bytes: &[u8]) -> Result<IbcConnections> {
    decode(bytes).map_err(|_| Error::ConnectionEncoding)
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

fn load_client() -> Result<AxonClient> {
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
