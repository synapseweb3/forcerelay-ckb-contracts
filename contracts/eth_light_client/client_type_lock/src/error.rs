use core::result;

use ckb_std::error::SysError;
use eth_light_client_in_ckb_verification::error::ProofUpdateError;

pub type Result<T> = result::Result<T, Error>;

#[repr(i8)]
pub enum Error {
    IndexOutOfBound = 1,
    ItemMissing,
    LengthNotEnough,
    Encoding,
    UnknownSysError,

    ClientShouldBeUniqueInInputs,
    ClientShouldBeUniqueInOutputs,
    UnknownOperation,
    WitnessIsNotExisted,
    ProofUpdateError,
    NewClientIsIncorrect,
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
    fn from(_err: ProofUpdateError) -> Self {
        Self::ProofUpdateError
    }
}
