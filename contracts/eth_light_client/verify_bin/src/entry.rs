use ckb_std::{ckb_constants::Source, cstr_core::CStr, error::SysError, high_level as hl};
use eth_light_client_in_ckb_verification::types::{
    core::Client,
    packed::{ClientReader, TransactionPayloadReader, TransactionProofReader},
    prelude::*,
};

use crate::error::{Error, Result};

const CLIENT_ARG_INDEX: isize = 0;
const WITNESS_ARG_INDEX: isize = 1;

pub fn main(argc: u64, argv: *const *const u8) -> Result<()> {
    debug!("{} Starting ...", module_path!());

    if argc != 2 {
        return Err(Error::IncorrectArgc);
    }

    let client_cell_index = load_usize_from_argv(argv, CLIENT_ARG_INDEX)?;
    debug!("client cell index = {}", client_cell_index);

    let witness_index = load_usize_from_argv(argv, WITNESS_ARG_INDEX)?;
    debug!("witness index = {}", witness_index);

    let client: Client = {
        let data = hl::load_cell_data(client_cell_index, Source::CellDep)?;
        let client = ClientReader::from_slice(&data).map_err(|_| SysError::Encoding)?;
        debug!("packed client = {}", client);
        client.unpack()
    };

    let witness_args = hl::load_witness_args(witness_index, Source::Input)?;
    let witness_args_reader = witness_args.as_reader();

    let tx_proof = if let Some(args) = witness_args_reader.input_type().to_opt() {
        let data = args.raw_data();
        if let Ok(reader) = TransactionProofReader::from_slice(data) {
            if client.verify_packed_transaction_proof(reader).is_ok() {
                reader.unpack()
            } else {
                return Err(Error::FailedToVerifyTransactionProof);
            }
        } else {
            return Err(Error::IncorrectTransactionProof);
        }
    } else {
        return Err(Error::TransactionProofIsNotExisted);
    };

    if let Some(args) = witness_args_reader.output_type().to_opt() {
        let data = args.raw_data();
        if let Ok(reader) = TransactionPayloadReader::from_slice(data) {
            if tx_proof.verify_packed_payload(reader).is_err() {
                return Err(Error::FailedToVerifyTransactionPayload);
            }
        } else {
            return Err(Error::IncorrectTransactionPayload);
        }
    } else {
        return Err(Error::TransactionPayloadIsNotExisted);
    };

    debug!("{} DONE.", module_path!());

    Ok(())
}

fn load_usize_from_argv(argv: *const *const u8, index: isize) -> Result<usize> {
    let argv_ptr = unsafe { argv.offset(index) };
    let value: usize = if let Some(arg_ptr) = unsafe { argv_ptr.as_ref() } {
        if let Some(arg_ptr) = unsafe { arg_ptr.as_ref() } {
            let arg_cstr = unsafe { CStr::from_ptr(arg_ptr) };
            if let Ok(arg_str) = arg_cstr.to_str() {
                if let Ok(index) = arg_str.parse() {
                    index
                } else {
                    return Err(Error::IncorrectArgv);
                }
            } else {
                return Err(Error::IncorrectArgv);
            }
        } else {
            return Err(Error::IncorrectArgv);
        }
    } else {
        return Err(Error::IncorrectArgv);
    };
    Ok(value)
}
