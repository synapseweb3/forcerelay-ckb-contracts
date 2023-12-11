#![no_std]
extern crate alloc;

atomics_polyfill::use_atomics_polyfill!();

pub mod error;
pub mod handler;
pub mod utils;

pub use ckb_ics_axon as ckb_ics;
