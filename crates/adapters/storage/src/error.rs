//! Errors of the storage adapter and their mapping onto registered codes.

use std::error::Error;
use std::fmt;
use std::io;

use codeskimr_application::error::CodedError;
use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_domain::ModelError;
use rusqlite::ErrorCode as SqliteCode;

/// Why a storage operation failed.
#[derive(Debug)]
pub(crate) enum StorageError {
    /// `SQLite` reported a failure.
    Sqlite(rusqlite::Error),
    /// A schema migration could not be applied.
    Migration(rusqlite::Error),
    /// The database was written by a newer version of the program.
    SchemaTooNew,
    /// The database file or its folder could not be prepared.
    Io(io::Error),
    /// Stored data violates the domain model.
    Model(ModelError),
    /// A number does not fit the range of its column or type.
    OutOfRange,
    /// The project does not exist.
    ProjectNotFound,
    /// Another project already has this source.
    ProjectExists,
}

impl StorageError {
    /// A failed migration; a full disk keeps its own code.
    pub(crate) fn migration(cause: rusqlite::Error) -> Self {
        match Self::from(cause) {
            Self::Sqlite(error) if sqlite_code(&error) == ErrorCode::DiskFull => {
                Self::Sqlite(error)
            }
            Self::Sqlite(error) => Self::Migration(error),
            other => other,
        }
    }
}

fn sqlite_code(error: &rusqlite::Error) -> ErrorCode {
    match error.sqlite_error_code() {
        Some(SqliteCode::DatabaseBusy | SqliteCode::DatabaseLocked) => {
            ErrorCode::TransactionConflict
        }
        Some(SqliteCode::DiskFull) => ErrorCode::DiskFull,
        _ => ErrorCode::DatabaseFailed,
    }
}

impl Coded for StorageError {
    fn code(&self) -> ErrorCode {
        match self {
            Self::Sqlite(error) => sqlite_code(error),
            Self::Migration(_) | Self::SchemaTooNew => ErrorCode::SchemaMigrationFailed,
            Self::Io(error) if error.kind() == io::ErrorKind::StorageFull => ErrorCode::DiskFull,
            Self::Io(_) => ErrorCode::DatabaseFailed,
            Self::Model(_) | Self::OutOfRange => ErrorCode::StoredDataInvalid,
            Self::ProjectNotFound => ErrorCode::ProjectNotFound,
            Self::ProjectExists => ErrorCode::ProjectAlreadyRegistered,
        }
    }
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code().description())
    }
}

impl Error for StorageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Sqlite(error) | Self::Migration(error) => Some(error),
            Self::Io(error) => Some(error),
            Self::Model(error) => Some(error),
            Self::SchemaTooNew | Self::OutOfRange | Self::ProjectNotFound | Self::ProjectExists => {
                None
            }
        }
    }
}

impl From<rusqlite::Error> for StorageError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Sqlite(error)
    }
}

impl From<io::Error> for StorageError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<ModelError> for StorageError {
    fn from(error: ModelError) -> Self {
        Self::Model(error)
    }
}

impl From<StorageError> for CodedError {
    fn from(error: StorageError) -> Self {
        Self::caused_by(error)
    }
}

/// Result of a storage operation.
pub(crate) type StorageResult<T> = Result<T, StorageError>;
