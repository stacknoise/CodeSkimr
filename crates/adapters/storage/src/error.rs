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
    /// A reference points at a symbol or file that the snapshot does not hold.
    DanglingReference,
    /// The project does not exist.
    ProjectNotFound,
    /// Another project already has this source.
    ProjectExists,
    /// The symbol of an annotation is not in the newest snapshot of its project.
    TargetNotFound,
    /// The fields of an annotation do not fit its kind, or the annotation is not of the kind that
    /// the operation needs.
    AnnotationInvalid,
    /// The annotation does not exist.
    AnnotationNotFound,
    /// The alert does not exist.
    AlertNotFound,
    /// The symbol already has a critical annotation.
    CriticalExists,
}

impl StorageError {
    /// A failed migration; busy, locked and full-disk failures keep their own codes.
    pub(crate) fn migration(cause: rusqlite::Error) -> Self {
        if sqlite_code(&cause) == ErrorCode::DatabaseFailed {
            Self::Migration(cause)
        } else {
            Self::Sqlite(cause)
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
            Self::Model(_) | Self::OutOfRange | Self::DanglingReference => {
                ErrorCode::StoredDataInvalid
            }
            Self::ProjectNotFound => ErrorCode::ProjectNotFound,
            Self::ProjectExists => ErrorCode::ProjectAlreadyRegistered,
            Self::TargetNotFound => ErrorCode::AnnotationTargetNotFound,
            Self::AnnotationInvalid => ErrorCode::AnnotationInvalid,
            Self::AnnotationNotFound => ErrorCode::AnnotationNotFound,
            Self::AlertNotFound => ErrorCode::AlertNotFound,
            Self::CriticalExists => ErrorCode::CriticalAlreadyMarked,
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
            Self::SchemaTooNew
            | Self::OutOfRange
            | Self::DanglingReference
            | Self::ProjectNotFound
            | Self::ProjectExists
            | Self::TargetNotFound
            | Self::AnnotationInvalid
            | Self::AnnotationNotFound
            | Self::AlertNotFound
            | Self::CriticalExists => None,
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
