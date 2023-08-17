#![no_std]
#![no_main]

mod entry;

use ckb_std::default_alloc;

ckb_std::entry!(program_entry);
default_alloc!();

fn program_entry() -> i8 {
    match entry::main() {
        Ok(_) => 0,
        Err(err) => err,
    }
}
