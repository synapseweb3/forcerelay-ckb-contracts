use core::result;

use ckb_std::error::SysError;
use eth_light_client_in_ckb_verification::error::TxVerificationError;

pub type Result<T> = result::Result<T, Error>;

pub enum Error {
    // 0x01 ~ 0x0f: Errors from SDK, or other system errors.
    IndexOutOfBound,
    ItemMissing,
    LengthNotEnough,
    Encoding,
    UnknownSysError,

    // 0x10 ~ 0x2f: Errors in current crate.
    IncorrectArgc,
    IncorrectArgv,
    TransactionProofIsNotExisted,
    TransactionPayloadIsNotExisted,
    IncorrectTransactionProof,
    IncorrectTransactionPayload,

    // 0x30 ~ 0x3f: Errors when verify transaction proof.
    FailedToVerifyTransactionProof(TxVerificationError),
    // 0x40 ~ 0x4f: Errors when verify transaction payload.
    FailedToVerifyTransactionPayload(TxVerificationError),
}

impl From<SysError> for Error {
    fn from(err: SysError) -> Self {
        use SysError::*;
        match err {
            IndexOutOfBound => Self::IndexOutOfBound,
            ItemMissing => Self::ItemMissing,
            LengthNotEnough(_) => Self::LengthNotEnough,
            Encoding => Self::Encoding,
            Unknown(_) => Self::UnknownSysError,
        }
    }
}

impl From<Error> for i8 {
    fn from(err: Error) -> Self {
        match err {
            Error::IndexOutOfBound => 0x01,
            Error::ItemMissing => 0x02,
            Error::LengthNotEnough => 0x03,
            Error::Encoding => 0x04,
            Error::UnknownSysError => 0x0f,

            Error::IncorrectArgc => 0x11,
            Error::IncorrectArgv => 0x12,
            Error::TransactionProofIsNotExisted => 0x13,
            Error::TransactionPayloadIsNotExisted => 0x14,
            Error::IncorrectTransactionProof => 0x15,
            Error::IncorrectTransactionPayload => 0x16,

            Error::FailedToVerifyTransactionProof(e) => 0x30 + e as i8,
            Error::FailedToVerifyTransactionPayload(e) => 0x40 + e as i8,
        }
    }
}
