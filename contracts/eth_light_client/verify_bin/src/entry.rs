use ckb_std::{ckb_constants::Source, env, error::SysError, high_level as hl};
use eth_light_client_in_ckb_verification::types::{
    core::Client,
    packed::{ClientReader, TransactionPayloadReader, TransactionProofReader},
    prelude::*,
};

use crate::error::{Error, InternalError, Result};

const CLIENT_ARG_INDEX: usize = 0;
const WITNESS_ARG_INDEX: usize = 1;

pub fn main() -> Result<()> {
    debug!("{} Starting ...", module_path!());

    let argv = env::argv();

    if argv.len() != 2 {
        return Err(InternalError::IncorrectArgc.into());
    }

    let client_cell_index = load_usize_from_argv(argv, CLIENT_ARG_INDEX)?;
    debug!("client cell index = {client_cell_index}");

    let witness_index = load_usize_from_argv(argv, WITNESS_ARG_INDEX)?;
    debug!("witness index = {witness_index}");

    let client: Client = {
        let data = hl::load_cell_data(client_cell_index, Source::CellDep)?;
        let client = ClientReader::from_slice(&data).map_err(|_| SysError::Encoding)?;
        debug!("packed client = {client}");
        client.unpack()
    };

    let witness_args = hl::load_witness_args(witness_index, Source::Input)?;
    let witness_args_reader = witness_args.as_reader();

    let tx_proof = if let Some(args) = witness_args_reader.input_type().to_opt() {
        let data = args.raw_data();
        if let Ok(reader) = TransactionProofReader::from_slice(data) {
            client
                .verify_packed_transaction_proof(reader)
                .map_err(Error::FailedToVerifyTransactionProof)?;
            reader.unpack()
        } else {
            return Err(InternalError::IncorrectTransactionProof.into());
        }
    } else {
        return Err(InternalError::TransactionProofIsNotExisted.into());
    };

    if let Some(args) = witness_args_reader.output_type().to_opt() {
        let data = args.raw_data();
        if let Ok(reader) = TransactionPayloadReader::from_slice(data) {
            tx_proof
                .verify_packed_payload(reader)
                .map_err(Error::FailedToVerifyTransactionPayload)?;
        } else {
            return Err(InternalError::IncorrectTransactionPayload.into());
        }
    } else {
        return Err(InternalError::TransactionPayloadIsNotExisted.into());
    };

    debug!("{} DONE.", module_path!());

    Ok(())
}

fn load_usize_from_argv(argv: &[env::Arg], index: usize) -> Result<usize> {
    if let Ok(arg_str) = argv[index].to_str() {
        if let Ok(value) = arg_str.parse() {
            Ok(value)
        } else {
            Err(InternalError::IncorrectArgv.into())
        }
    } else {
        Err(InternalError::IncorrectArgv.into())
    }
}
