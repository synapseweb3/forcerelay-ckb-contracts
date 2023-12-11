#![no_std]

#[macro_export]
macro_rules! use_atomics_polyfill {
    () => {
        #[link(name = "atomics-polyfill")]
        extern "C" {}
    };
}
