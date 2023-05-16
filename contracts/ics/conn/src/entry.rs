use ckb_ics_axon::handler::{
    handle_msg_connection_open_ack, handle_msg_connection_open_confirm,
    handle_msg_connection_open_init, handle_msg_connection_open_try, IbcConnections,
};
use ckb_ics_axon::message::{
    Envelope, MsgConnectionOpenAck, MsgConnectionOpenConfirm, MsgConnectionOpenInit,
    MsgConnectionOpenTry, MsgType,
};
use ckb_standalone_types::prelude::Entity;
use rlp::decode;
use tiny_keccak::{Hasher as _, Keccak};
use axon_client::AxonClient as Client;

use ckb_std::error::SysError;
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
        _ => return Err(Error::UnexpectedMsg),
    }

    let _current_script = hl::load_script().map_err(|_| Error::LoadScriptErr)?;
    // let client_data = hl::load_cell_data(0, Source::CellDep).map_err(|_| Error::LoadCellDataErr)?;
    // let client = decode::<Client>(&client_data).map_err(|_| Error::ClientEncoding)?;
    let client = load_client()?;

    let input_data = hl::load_cell_data(0, Source::GroupInput)?;
    let expected_input_hash: [u8; 32] = input_data
        .try_into()
        .map_err(|_| Error::ConnectionHashUnmatch)?;

    let output_data = hl::load_cell_data(0, Source::GroupOutput)?;
    let expected_output_hash: [u8; 32] = output_data
        .try_into()
        .map_err(|_| Error::ConnectionHashUnmatch)?;

    let witness_args0 = hl::load_witness_args(0, Source::Input)?;
    let witness_args1 = hl::load_witness_args(0, Source::Output)?;

    let old_connection_cell_data = witness_args0.input_type();
    let new_connection_cell_data = witness_args1.output_type();

    if old_connection_cell_data.is_none() || new_connection_cell_data.is_none() {
        return Err(Error::ConnectionEncoding);
    }

    let old_connection_bytes = old_connection_cell_data.to_opt().unwrap();
    let old_connection_slice = old_connection_bytes.as_slice();
    if keccak256(old_connection_slice) != expected_input_hash {
        return Err(Error::ConnectionHashUnmatch);
    }

    let old_connection_cell = decode_connection_cell(old_connection_slice)?;

    let new_connection_bytes = new_connection_cell_data.to_opt().unwrap();
    let new_connection_slice = new_connection_bytes.as_slice();
    if keccak256(new_connection_slice) != expected_output_hash {
        return Err(Error::ConnectionHashUnmatch);
    }

    let new_connection_cell = decode_connection_cell(new_connection_slice)?;

    verify(old_connection_cell, new_connection_cell, envelope, client)?;

    Ok(())
}

fn verify(
    old: IbcConnections,
    new: IbcConnections,
    envelope: Envelope,
    client: Client,
) -> Result<()> {
    match envelope.msg_type {
        MsgType::MsgConnectionOpenInit => {
            let msg = decode::<MsgConnectionOpenInit>(&envelope.content)
                .map_err(|_| Error::MsgEncoding)?;
            handle_msg_connection_open_init(client, old, new, msg)
                .map_err(|_| Error::ConnectionProofInvalid)
        }
        MsgType::MsgConnectionOpenTry => {
            let msg = decode::<MsgConnectionOpenTry>(&envelope.content)
                .map_err(|_| Error::MsgEncoding)?;
            handle_msg_connection_open_try(client, old, new, msg)
                .map_err(|_| Error::ConnectionProofInvalid)
        }
        MsgType::MsgConnectionOpenAck => {
            let msg = decode::<MsgConnectionOpenAck>(&envelope.content)
                .map_err(|_| Error::MsgEncoding)?;
            handle_msg_connection_open_ack(client, old, new, msg)
                .map_err(|_| Error::ConnectionProofInvalid)
        }
        MsgType::MsgConnectionOpenConfirm => {
            let msg = decode::<MsgConnectionOpenConfirm>(&envelope.content)
                .map_err(|_| Error::MsgEncoding)?;
            handle_msg_connection_open_confirm(client, old, new, msg)
                .map_err(|_| Error::ConnectionProofInvalid)
        }
        _ => Err(Error::UnexpectedMsg),
    }
}

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