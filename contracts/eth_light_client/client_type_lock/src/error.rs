use core::result;

use ckb_std::error::SysError;
use eth_light_client_in_ckb_verification::error::{
    ClientBootstrapError, ClientUpdateError, SyncCommitteeUpdateError,
};

pub type Result<T> = result::Result<T, Error>;

#[repr(i8)]
pub enum InternalError {
    // 0x01 ~ 0x0f: Errors from SDK, or other system errors.
    IndexOutOfBound = 0x01,
    ItemMissing,
    LengthNotEnough,
    Encoding,
    UnknownSysError,

    // 0x10 ~ 0x1f: Errors before doing operations.
    UnknownOperation = 0x10,
    // 0x20 ~ 0x37: Errors when do create.
    CreateNotEnoughCells = 0x20,
    CreateShouldBeOrdered,
    CreateCellsCountNotMatched,
    CreateIncorrectUniqueId,
    CreateBadClientInfoCellData,
    CreateClientInfoIndexShouldBeZero,
    CreateClientInfoMinimalHeadersCountShouldNotBeZero,
    CreateWitnessIsNotExisted,
    CreateBadClientCellData,
    CreateNewClientIsIncorrect,
    CreateBadClientSyncCommitteeCellData,
    CreateNewSyncCommitteeIsIncorrect,
    // 0x38 ~ 0x3f: Errors when do destroy.
    DestroyNotEnoughCells = 0x3f,
    // 0x40 ~ 0x4f: Errors when update client.
    UpdateClientInputInfoNotFound = 0x40,
    UpdateClientInputClientNotFound,
    UpdateClientInputClientIdIsMismatch,
    UpdateClientOutputInfoNotFound,
    UpdateClientOutputClientNotFound,
    UpdateClientInfoChanged,
    UpdateClientCellDepsTooMany,
    UpdateClientCellDepsNotEnough,
    UpdateClientCellDepClientNotFound,
    UpdateClientCellDepSyncCommitteeNotFound,
    UpdateClientCellDepClientIdIsMismatch,
    UpdateClientWitnessIsNotExisted,
    UpdateClientHeadersNotEnough,
    // 0x50 ~ 0x5f: Errors when update sync committee.
    UpdateSyncCommitteeInputSyncCommitteeNotFound = 0x50,
    UpdateSyncCommitteeOutputSyncCommitteeNotFound,
    UpdateSyncCommitteeCellDepsTooMany,
    UpdateSyncCommitteeCellDepsNotEnough,
    UpdateSyncCommitteeCellDepInfoNotFound,
    UpdateSyncCommitteeCellDepClientNotFound,
    UpdateSyncCommitteeCellDepSyncCommitteeNotFound,
    UpdateSyncCommitteeCellDepClientIsNotLatest,
    UpdateSyncCommitteeCellDepSyncCommitteeIsNotOldest,
    UpdateSyncCommitteeWitnessIsNotExisted,
}

pub enum Error {
    // 0x01 ~ 0x5f: Errors that not from external crates.
    Internal(InternalError),
    // 0x60 ~ 0x7f: Errors when bootstrap or apply the update.
    //
    // Different steps may have same error codes.
    ClientBootstrap(ClientBootstrapError),
    ClientUpdate(ClientUpdateError),
    SyncCommitteeUpdate(SyncCommitteeUpdateError),
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

impl From<ClientBootstrapError> for Error {
    fn from(err: ClientBootstrapError) -> Self {
        Self::ClientBootstrap(err)
    }
}

impl From<ClientUpdateError> for Error {
    fn from(err: ClientUpdateError) -> Self {
        Self::ClientUpdate(err)
    }
}

impl From<SyncCommitteeUpdateError> for Error {
    fn from(err: SyncCommitteeUpdateError) -> Self {
        Self::SyncCommitteeUpdate(err)
    }
}

impl From<Error> for i8 {
    fn from(err: Error) -> Self {
        match err {
            Error::Internal(e) => e as i8,
            Error::ClientBootstrap(e) => 0x60 + e as i8,
            Error::ClientUpdate(e) => 0x60 + e as i8,
            Error::SyncCommitteeUpdate(e) => 0x60 + e as i8,
        }
    }
}
