use ics_base::error::Result;
use ics_base::handler::{navigate_channel, verify, Navigator};
use ics_base::utils::load_client;

pub fn main() -> Result<()> {
    let envelope = match navigate_channel()? {
        Navigator::CHECK_MESSAGE(envelope) => envelope,
        _ => return Ok(()),
    };

    let client = load_client()?;
    verify(envelope, client)
}
