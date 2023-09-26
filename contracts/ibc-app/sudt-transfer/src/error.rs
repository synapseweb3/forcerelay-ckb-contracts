use ckb_std::{debug, error::SysError};

/// Error
#[repr(i8)]
#[derive(Debug)]
pub enum Error {
    IndexOutOfBound = 1,
    ItemMissing = 2,
    LengthNotEnough = 3,
    Encoding = 4,

    // Add customized errors here...
    InvalidArgs = 5,
    InvalidMsgType = 6,
    Input = 7,
    Output = 8,
    PacketData = 9,
    SudtAmount = 10,
    Denom = 11,
    InvalidAck = 12,
    SenderReceiver = 13,
    Ics = 20,
}

impl From<SysError> for Error {
    fn from(err: SysError) -> Self {
        use SysError::*;
        match err {
            IndexOutOfBound => Self::IndexOutOfBound,
            ItemMissing => Self::ItemMissing,
            LengthNotEnough(_) => Self::LengthNotEnough,
            Encoding => Self::Encoding,
            Unknown(err_code) => panic!("unexpected sys error {}", err_code),
        }
    }
}

impl From<ics_base::error::Error> for Error {
    fn from(error: ics_base::error::Error) -> Self {
        debug!("ics error: {:?}", error);
        Self::Ics
    }
}
