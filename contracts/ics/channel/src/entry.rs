use ics_base::error::CkbResult;
use ics_base::handler::{navigate_channel, verify, Navigator};
use ics_base::utils::load_client;

pub fn main() -> CkbResult<()> {
    let envelope = match navigate_channel()? {
        Navigator::CheckMessage(envelope) => envelope,
        _ => return Ok(()),
    };

    let client = load_client()?;
    verify(envelope, client)
}
