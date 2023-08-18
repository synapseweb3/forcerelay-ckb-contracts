use core::result;

use ckb_std::error::SysError;

pub type Result<T> = result::Result<T, Error>;
pub type CkbResult<T> = result::Result<T, i8>;

#[repr(i8)]
pub enum Error {
    IndexOutOfBound = 1,
    ItemMissing,
    LengthNotEnough,
    Encoding,
    UnknownSysError,

    UnexpectedMsg,
    UnexpectedConnectionMsg,
    UnexpectedChannelMsg,
    UnexpectedPacketMsg,
    MetadataSerde,

    PacketEncoding,
    ChannelEncoding,
    ConnectionEncoding,
    EnvelopeEncoding,
    MsgEncoding,

    WitnessIsIncorrect,
    ConnectionWitnessInputOrOutputIsNone,
    ChannelWitnessInputOrOutputIsNone,
    PacketWitnessInputOrOutputIsNone,

    FailedToLoadClientCellData,
    FailedToLoadClientTypeScript,
    FailedToLoadClientId,
    FailedToCreateClient,

    CellDataUnmatch,
    ConnectionHashUnmatch,
    ChannelHashUnmatch,
    PacketHashUnmatch,

    ConnectionLock,
    ChannelLock,
    PacketLock,

    ClientCreateWrongClientId,
    ClientCreateWrongConnectionCell,
}

impl From<Error> for i8 {
    fn from(value: Error) -> Self {
        value as i8
    }
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
