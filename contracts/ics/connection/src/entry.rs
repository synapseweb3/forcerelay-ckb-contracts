use ckb_std::ckb_constants::Source;

use ics_base::error::{CkbResult, Error};
use ics_base::handler::{navigate_connection, verify, Navigator};
use ics_base::utils::{load_client, load_connection_cell};

pub fn main() -> CkbResult<()> {
    match navigate_connection()? {
        Navigator::CheckClient => check_client(),
        Navigator::CheckMessage(envelope) => verify(envelope),
        _ => Ok(()),
    }
}

fn check_client() -> CkbResult<()> {
    let (connection_cell, connection_args) = load_connection_cell(0, Source::Output)?;
    let _client = load_client(
        connection_args.metadata_type_id,
        connection_args.ibc_handler_address,
    )?;

    if !connection_cell.connections.is_empty() || connection_cell.next_channel_number != 0 {
        return Err(Error::ClientCreateWrongConnectionCell.into());
    }

    Ok(())
}
