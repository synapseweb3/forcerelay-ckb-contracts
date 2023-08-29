use ckb_std::ckb_constants::Source;

use ics_base::axon_client::AxonClient;
use ics_base::ckb_ics::handler::Client;
use ics_base::error::{CkbResult, Error};
use ics_base::handler::{navigate_connection, verify, Navigator};
use ics_base::utils::load_connection_cell;

pub fn main() -> CkbResult<()> {
    match navigate_connection()? {
        Navigator::CheckClient(client) => check_client(client),
        Navigator::CheckMessage(envelope, client) => verify(envelope, client),
        _ => Ok(()),
    }
}

fn check_client(client: AxonClient) -> CkbResult<()> {
    let (connection_cell, connection_args) = load_connection_cell(0, Source::Output)?;
    if connection_args.client_id.as_slice() != client.client_id() {
        return Err(Error::ClientCreateWrongClientId.into());
    }

    if !connection_cell.connections.is_empty()
        || connection_cell.next_channel_number != 0
        || connection_cell.next_connection_number != 0
    {
        return Err(Error::ClientCreateWrongConnectionCell.into());
    }

    Ok(())
}
