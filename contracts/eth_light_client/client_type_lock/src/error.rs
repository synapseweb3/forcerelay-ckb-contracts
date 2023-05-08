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

    // 0x10 ~ 0x4f: Errors in current crate.
    UnknownOperation,
    OnlyOneInputWithCurrentType,

    CreateNotEnoughCells,
    CreateShouldBeOrdered,
    CreateCellsCountNotMatched,
    CreateIncorrectUniqueId,
    CreateBadClientInfoCellData,
    CreateClientInfoIndexShouldBeZero,
    CreateClientInfoMinimalUpdatesCountShouldNotBeZero,
    CreateBadClientCellData,
    CreateUpdatesIsNotEnough,
    CreateWitnessIsNotExisted,
    CreateNewClientIsIncorrect,

    UpdateInputClientInfoCellNotFound,
    UpdateInputClientCellNotFound,
    UpdateOutputClientInfoCellNotFound,
    UpdateOutputClientCellNotFound,
    UpdateClientInfoMinimalUpdatesCountChanged,
    UpdateClientInfoNewLastIdIsIncorrect,
    UpdateClientInputLastIdIsIncorrect,
    UpdateCellDepLastIdIsIncorrect,
    UpdateMoreThanOneCellDepsWithCurrentType,
    UpdateCellDepClientCellNotFound,
    UpdateUpdatesIsNotEnough,
    UpdateWitnessIsNotExisted,
    UpdateNewClientIsIncorrect,

    DestroyNotEnoughCells,

    // 0x50 ~ 0x7f: Errors when apply proof update.
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

            Error::UnknownOperation => 0x10,
            Error::OnlyOneInputWithCurrentType => 0x11,

            Error::CreateNotEnoughCells => 0x20,
            Error::CreateShouldBeOrdered => 0x21,
            Error::CreateCellsCountNotMatched => 0x22,
            Error::CreateIncorrectUniqueId => 0x23,
            Error::CreateBadClientInfoCellData => 0x24,
            Error::CreateClientInfoIndexShouldBeZero => 0x25,
            Error::CreateClientInfoMinimalUpdatesCountShouldNotBeZero => 0x26,
            Error::CreateBadClientCellData => 0x27,
            Error::CreateUpdatesIsNotEnough => 0x2d,
            Error::CreateWitnessIsNotExisted => 0x2e,
            Error::CreateNewClientIsIncorrect => 0x2f,

            Error::UpdateInputClientInfoCellNotFound => 0x30,
            Error::UpdateInputClientCellNotFound => 0x31,
            Error::UpdateOutputClientInfoCellNotFound => 0x32,
            Error::UpdateOutputClientCellNotFound => 0x33,
            Error::UpdateClientInfoMinimalUpdatesCountChanged => 0x34,
            Error::UpdateClientInfoNewLastIdIsIncorrect => 0x35,
            Error::UpdateClientInputLastIdIsIncorrect => 0x36,
            Error::UpdateCellDepLastIdIsIncorrect => 0x37,
            Error::UpdateMoreThanOneCellDepsWithCurrentType => 0x38,
            Error::UpdateCellDepClientCellNotFound => 0x39,
            Error::UpdateUpdatesIsNotEnough => 0x3d,
            Error::UpdateWitnessIsNotExisted => 0x3e,
            Error::UpdateNewClientIsIncorrect => 0x3f,

            Error::DestroyNotEnoughCells => 0x40,

            Error::ProofUpdate(e) => 0x50 + e as i8,
        }
    }
}
