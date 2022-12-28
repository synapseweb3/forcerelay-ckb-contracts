use alloc::vec::Vec;

#[cfg(feature = "debugging")]
use ckb_std::ckb_types::prelude::*;
use ckb_std::{ckb_constants::Source, high_level as hl};

use crate::error::{Error, Result};

pub fn main() -> Result<()> {
    debug!("{} Starting ...", module_path!());

    let script_hash = hl::load_script_hash()?;
    debug!("script hash = {:#x}", script_hash.pack());
    let args = hl::load_script()?.args();
    let expected_witness = args.raw_data().iter().cloned().rev().collect::<Vec<u8>>();

    let mut handled_lock_hashes: Vec<[u8; 32]> = Vec::new();

    for (index, lock_hash) in
        hl::QueryIter::new(hl::load_cell_lock_hash, Source::Output).enumerate()
    {
        let mut checked = false;
        if lock_hash == script_hash {
            checked = true;
        } else {
            for handled_lock_hash in &handled_lock_hashes {
                if lock_hash == *handled_lock_hash {
                    checked = true;
                }
            }
        }
        let witness_args = match hl::load_witness_args(index, Source::Input) {
            Ok(witness_args) => witness_args,
            Err(_) => {
                if checked {
                    continue;
                } else {
                    debug!("failed to load {}-th witness", index);
                    return Err(Error::WitnessIsNotExisted);
                }
            }
        };
        if let Some(lock_witness) = witness_args.lock().to_opt() {
            if lock_witness.raw_data().as_ref() != expected_witness {
                debug!(
                    "failed to check witness: args: {:#x}, witness: {:#x}",
                    args, lock_witness
                );
                return Err(Error::WitnessIsIncorrect);
            }
        } else if !checked {
            debug!("failed to check witness: witness is empty");
            return Err(Error::WitnessIsEmpty);
        }

        if !checked {
            handled_lock_hashes.push(lock_hash);
        }
    }

    debug!("{} DONE.", module_path!());

    Ok(())
}
