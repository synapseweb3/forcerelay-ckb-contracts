use crate::error::Result;
use ckb_std::{ckb_constants::Source, high_level as hl};
use eth_light_client_in_ckb_verification::types::prelude::*;

const CKB_HASH_PERSONALIZATION: &[u8] = b"ckb-default-hash";
const BLAKE2B_LEN: usize = 32;

pub(crate) fn calculate_type_id(outputs_count: usize) -> Result<[u8; BLAKE2B_LEN]> {
    let input = hl::load_input(0, Source::Input)?;

    let mut blake2b = blake2b_rs::Blake2bBuilder::new(32)
        .personal(CKB_HASH_PERSONALIZATION)
        .build();
    blake2b.update(input.as_slice());

    blake2b.update(&(outputs_count as u64).to_le_bytes());

    let mut ret = [0; BLAKE2B_LEN];
    blake2b.finalize(&mut ret);

    Ok(ret)
}
