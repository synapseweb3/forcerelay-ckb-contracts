use alloc::ffi::NulError;
use core::result;

use ckb_std::error::SysError;

pub type Result<T> = result::Result<T, Error>;

#[repr(i8)]
pub enum Error {
    // 0x01 ~ 0x0f: Errors from SDK, or other system errors.
    IndexOutOfBound = 0x01,
    ItemMissing,
    LengthNotEnough,
    Encoding,
    UnknownSysError,

    // 0x10 ~ 0x2f: Errors in current crate.
    IncorrectArgs = 0x10,
    ClientCellDepIsNotExisted,
    BinCellDepIsNotExisted,
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

impl From<NulError> for Error {
    fn from(_err: NulError) -> Self {
        Self::Encoding
    }
}
