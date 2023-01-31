use ckb_std::{ckb_constants::Source, error::SysError, high_level as hl};
use eth_light_client_in_ckb_verification::types::{
    core::Client,
    packed::{ClientReader, ProofUpdateReader},
    prelude::*,
};

use crate::error::{Error, Result};

pub fn main() -> Result<()> {
    debug!("{} Starting ...", module_path!());

    let script_hash = hl::load_script_hash()?;
    debug!("script hash = {:#x}", script_hash.pack());

    let mut client_cell_index_opt = None;
    let mut new_client_cell_index_opt = None;

    for (index, type_hash_opt) in
        hl::QueryIter::new(hl::load_cell_type_hash, Source::Input).enumerate()
    {
        if let Some(type_hash) = type_hash_opt {
            debug!("{}-th type hash of inputs: {:#x}", index, type_hash.pack());
            if type_hash == script_hash.as_slice() {
                debug!("found client cell from inputs: {}", index);
                if client_cell_index_opt.is_none() {
                    client_cell_index_opt = Some(index);
                } else {
                    return Err(Error::ClientShouldBeUniqueInInputs);
                }
            }
        }
    }

    for (index, type_hash_opt) in
        hl::QueryIter::new(hl::load_cell_type_hash, Source::Output).enumerate()
    {
        if let Some(type_hash) = type_hash_opt {
            debug!("{}-th type hash of outputs: {:#x}", index, type_hash.pack());
            if type_hash == script_hash.as_slice() {
                debug!("found client cell from outputs: {}", index);
                if new_client_cell_index_opt.is_none() {
                    new_client_cell_index_opt = Some(index);
                } else {
                    return Err(Error::ClientShouldBeUniqueInOutputs);
                }
            }
        }
    }

    if let Some(new_client_cell_index) = new_client_cell_index_opt {
        debug!("create or update client cell");
        let witness_args = hl::load_witness_args(new_client_cell_index, Source::Output)?;
        let actual_new_client = if let Some(args) = witness_args.input_type().to_opt() {
            let data = args.raw_data();
            let proof_update =
                ProofUpdateReader::from_slice(&data).map_err(|_| SysError::Encoding)?;
            debug!(
                "packed proof update size = {}",
                proof_update.as_slice().len()
            );

            if let Some(client_cell_index) = client_cell_index_opt {
                debug!("update client cell");
                let input_data = hl::load_cell_data(client_cell_index, Source::Input)?;
                let client: Client = {
                    let client =
                        ClientReader::from_slice(&input_data).map_err(|_| SysError::Encoding)?;
                    debug!("packed client = {}", client);
                    client.unpack()
                };
                client.try_apply_packed_proof_update(proof_update)
            } else {
                debug!("create client cell");
                Client::new_from_packed_proof_update(proof_update)
            }?
        } else {
            return Err(Error::WitnessIsNotExisted);
        };

        let actual = actual_new_client.pack();
        debug!("actual packed new client = {}", actual);

        let output_data = hl::load_cell_data(new_client_cell_index, Source::Output)?;
        let expected = ClientReader::from_slice(&output_data).map_err(|_| SysError::Encoding)?;
        debug!("expected packed new client = {}", expected);

        if actual.as_slice() != expected.as_slice() {
            return Err(Error::NewClientIsIncorrect);
        }
    } else if client_cell_index_opt.is_some() {
        debug!("destroy the client cell: no checks");
    } else {
        debug!("unknown operation: throw an error");
        return Err(Error::UnknownOperation);
    }

    debug!("{} DONE.", module_path!());

    Ok(())
}
