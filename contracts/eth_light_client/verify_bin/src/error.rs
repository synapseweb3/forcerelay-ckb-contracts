use core::result;

use ckb_std::error::SysError;
use eth_light_client_in_ckb_verification::error::TxVerificationError;

pub type Result<T> = result::Result<T, Error>;

#[repr(i8)]
pub enum InternalError {
    // 0x01 ~ 0x0f: Errors from SDK, or other system errors.
    IndexOutOfBound = 0x01,
    ItemMissing,
    LengthNotEnough,
    Encoding,
    UnknownSysError,

    // 0x10 ~ 0x2f: Errors in current crate.
    IncorrectArgc = 0x10,
    IncorrectArgv,
    TransactionProofIsNotExisted,
    TransactionPayloadIsNotExisted,
    IncorrectTransactionProof,
    IncorrectTransactionPayload,
}

pub enum Error {
    // 0x01 ~ 0x2f: Errors that not from external crates.
    Internal(InternalError),
    // 0x40 ~ 0x5f: Errors when verify transaction proof.
    FailedToVerifyTransactionProof(TxVerificationError),
    // 0x60 ~ 0x7f: Errors when verify transaction payload.
    FailedToVerifyTransactionPayload(TxVerificationError),
}

impl From<SysError> for InternalError {
    fn from(err: SysError) -> Self {
        match err {
            SysError::IndexOutOfBound => Self::IndexOutOfBound,
            SysError::ItemMissing => Self::ItemMissing,
            SysError::LengthNotEnough(_) => Self::LengthNotEnough,
            SysError::Encoding => Self::Encoding,
            SysError::Unknown(_) => Self::UnknownSysError,
        }
    }
}

impl From<SysError> for Error {
    fn from(err: SysError) -> Self {
        Into::<InternalError>::into(err).into()
    }
}

impl From<InternalError> for Error {
    fn from(err: InternalError) -> Self {
        Self::Internal(err)
    }
}

impl From<Error> for i8 {
    fn from(err: Error) -> Self {
        match err {
            Error::Internal(e) => e as i8,
            Error::FailedToVerifyTransactionProof(e) => 0x40 + e as i8,
            Error::FailedToVerifyTransactionPayload(e) => 0x60 + e as i8,
        }
    }
}
