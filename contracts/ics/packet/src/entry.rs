use ics_base::error::CkbResult;
use ics_base::handler::{navigate_packet, verify, Navigator};

pub fn main() -> CkbResult<()> {
    match navigate_packet()? {
        Navigator::CheckMessage(envelope) => verify(envelope),
        _ => Ok(()),
    }
}
