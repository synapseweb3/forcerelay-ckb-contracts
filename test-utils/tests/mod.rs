use std::io::Write as _;

use env_logger::{Builder, Target};
use log::LevelFilter;

pub(crate) mod eth_light_client;
pub(crate) mod mock_contracts;

pub(crate) fn setup() {
    let _ = Builder::new()
        .filter_module("ibc_ckb_contracts", LevelFilter::Trace)
        .format(|buf, record| writeln!(buf, "[{}] {}", record.level(), record.args()))
        .target(Target::Stdout)
        .is_test(true)
        .try_init();
    println!();
}
