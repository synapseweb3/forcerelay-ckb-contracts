use ckb_std::{ckb_constants::Source, error::SysError, high_level as hl};
use eth_light_client_in_ckb_verification::types::{
    core::Client,
    packed::{ClientReader, ProofUpdateReader},
    prelude::*,
};

use crate::error::{Error, Result};

const CLIENT_INDEX: usize = 0;
const NEW_CLIENT_INDEX: usize = 0;

pub fn main() -> Result<()> {
    debug!("{} Starting ...", module_path!());

    let client: Client = {
        let data = hl::load_cell_data(CLIENT_INDEX, Source::Input)?;
        let client = ClientReader::from_slice(&data).map_err(|_| SysError::Encoding)?;
        debug!("packed client = {}", client);
        client.unpack()
    };

    let witness_args = hl::load_witness_args(CLIENT_INDEX, Source::Input)?;

    if witness_args.lock().is_none() {
        let client_type_hash =
            if let Some(client_type_hash) = hl::load_cell_type_hash(CLIENT_INDEX, Source::Input)? {
                client_type_hash
            } else {
                return Err(Error::ClientShouldHasTypeScript);
            };
        if let Some(new_client_type_hash) =
            hl::load_cell_type_hash(NEW_CLIENT_INDEX, Source::Output)?
        {
            if client_type_hash != new_client_type_hash {
                return Err(Error::TypeShouldNotChangeWhenNoLockWitness);
            }
        } else {
            return Err(Error::TypeShouldNotChangeWhenNoLockWitness);
        }
    }

    let actual_new_client = if let Some(args) = witness_args.input_type().to_opt() {
        let data = args.raw_data();
        let proof_update = ProofUpdateReader::from_slice(&data).map_err(|_| SysError::Encoding)?;
        debug!(
            "packed proof update size = {}",
            proof_update.as_slice().len()
        );
        client.try_apply_packed_proof_update(proof_update)?
    } else {
        return Err(Error::WitnessIsNotExisted);
    };

    let actual = actual_new_client.pack();
    debug!("actual packed new client = {}", actual);

    let data = hl::load_cell_data(NEW_CLIENT_INDEX, Source::Output)?;
    let expected = ClientReader::from_slice(&data).map_err(|_| SysError::Encoding)?;
    debug!("expected packed new client = {}", expected);

    if actual.as_slice() != expected.as_slice() {
        return Err(Error::NewClientIsIncorrect);
    }

    debug!("{} DONE.", module_path!());

    Ok(())
}
