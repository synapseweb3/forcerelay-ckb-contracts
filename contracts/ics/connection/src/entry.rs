use ckb_std::{ckb_constants::Source, high_level as hl};

use ckb_ics_axon::message::Envelope;
use ics_base::error::{Error, Result};
use ics_base::handler::{navigate_connection, verify, Navigator};
use ics_base::utils::{load_client, load_connection_cell};

pub fn main() -> Result<()> {
    match navigate_connection()? {
        Navigator::CHECK_CLIENT => check_client()?,
        Navigator::CHECK_MESSAGE(envelope) => check_message(envelope)?,
        _ => {}
    }
}

fn check_client() -> Result<()> {
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

fn check_message(envelope: Envelope) -> Result<()> {
    let client = load_client()?;
    verify(envelope, client)
}
