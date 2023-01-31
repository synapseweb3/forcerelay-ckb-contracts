use core::result;

use ckb_std::error::SysError;
use eth_light_client_in_ckb_verification::error::ProofUpdateError;

pub type Result<T> = result::Result<T, Error>;

pub enum Error {
    // 0x01 ~ 0x0f: Errors from SDK, or other system errors.
    IndexOutOfBound,
    ItemMissing,
    LengthNotEnough,
    Encoding,
    UnknownSysError,

    // 0x10 ~ 0x2f: Errors in current crate.
    NewClientIsIncorrect,
    ClientShouldBeUniqueInInputs,
    ClientShouldBeUniqueInOutputs,
    UnknownOperation,
    WitnessIsNotExisted,

    // 0x30 ~ 0x3f: Errors when apply proof update.
    ProofUpdate(ProofUpdateError),
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

impl From<ProofUpdateError> for Error {
    fn from(err: ProofUpdateError) -> Self {
        Self::ProofUpdate(err)
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

            Error::NewClientIsIncorrect => 0x10,
            Error::ClientShouldBeUniqueInInputs => 0x11,
            Error::ClientShouldBeUniqueInOutputs => 0x12,
            Error::UnknownOperation => 0x13,
            Error::WitnessIsNotExisted => 0x14,

            Error::ProofUpdate(e) => 0x30 + e as i8,
        }
    }
}
