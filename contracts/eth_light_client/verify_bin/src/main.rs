#![no_std]
#![no_main]

macro_rules! debug {
    ($fmt:literal $(,$args:expr)* $(,)?) => {
        #[cfg(feature = "debugging")]
        ckb_std::syscalls::debug(alloc::format!($fmt $(,$args)*));
    };
}

mod entry;
mod error;

use ckb_std::default_alloc;

ckb_std::entry!(program_entry);
default_alloc!();

atomics_polyfill::use_atomics_polyfill!();

fn program_entry() -> i8 {
    match entry::main() {
        Ok(_) => 0,
        Err(err) => err.into(),
    }
}
