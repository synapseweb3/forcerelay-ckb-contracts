use ics_base::error::CkbResult;
use ics_base::handler::{navigate_channel, verify, Navigator};

pub fn main() -> CkbResult<()> {
    match navigate_channel()? {
        Navigator::CheckMessage(envelope) => verify(envelope),
        _ => Ok(()),
    }
}
