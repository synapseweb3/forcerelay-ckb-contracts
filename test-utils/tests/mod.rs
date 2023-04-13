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

pub(crate) mod prelude {
    use ckb_error::Error;
    use ckb_types::core::Cycle;

    pub(crate) trait IsVerifyResult {
        fn should_be_ok(&self);
        fn should_be_err(&self);
    }

    impl IsVerifyResult for Result<Cycle, Error> {
        fn should_be_ok(&self) {
            match self {
                Ok(cycles) => {
                    println!("Cost: {} cycles", cycles);
                }
                Err(reason) => {
                    panic!("Failed since: {}", reason);
                }
            }
        }

        fn should_be_err(&self) {
            match self {
                Ok(cycles) => {
                    panic!("Cost: {} cycles", cycles);
                }
                Err(reason) => {
                    println!("Failed since: {}", reason);
                }
            }
        }
    }
}
