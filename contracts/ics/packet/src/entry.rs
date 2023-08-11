use ics_base::error::{Error, Result};
use ics_base::handler::{navigate_packet, verify, Navigator};
use ics_base::utils::load_client;

pub fn main() -> Result<()> {
    let envelope = match navigate_packet()? {
        Navigator::CheckMessage(envelope) => envelope,
        _ => return Err(Error::UnexpectedPacketMsg),
    };

    let client = load_client().unwrap_or_default();
    verify(envelope, client)
}
