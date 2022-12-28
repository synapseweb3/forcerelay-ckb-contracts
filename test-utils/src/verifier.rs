use ckb_error::Error;
use ckb_script::TransactionScriptsVerifier;
use ckb_types::{
    core::{cell::ResolvedTransaction, Cycle},
    packed,
};

use crate::Context;

#[derive(Default)]
pub struct Verifier {}

impl Verifier {
    pub fn verify_without_limit(&self, rtx: &ResolvedTransaction) -> Result<Cycle, Error> {
        self.verify(rtx, u64::MAX)
    }

    pub fn verify(&self, rtx: &ResolvedTransaction, max_cycles: Cycle) -> Result<Cycle, Error> {
        self.verify_map(rtx, |verifier| verifier.verify(max_cycles))
    }

    pub fn verify_map<R, F>(&self, rtx: &ResolvedTransaction, mut verify_func: F) -> R
    where
        F: FnMut(TransactionScriptsVerifier<'_, Context>) -> R,
    {
        let context = Context::new();
        let mut verifier = TransactionScriptsVerifier::new(rtx, &context);
        verifier.set_debug_printer(|hash: &packed::Byte32, message: &str| {
            log::info!("{:#x} {}", hash, message);
        });
        verify_func(verifier)
    }
}
