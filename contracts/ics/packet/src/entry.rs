use ics_base::error::{CkbResult, Error};
use ics_base::handler::{navigate_packet, verify, Navigator};
use ics_base::utils::load_client;

pub fn main() -> CkbResult<()> {
    let envelope = match navigate_packet()? {
        Navigator::CheckMessage(envelope) => envelope,
        _ => return Err(Error::UnexpectedPacketMsg.into()),
    };

    let client = load_client()?;
    verify(envelope, client)
}
