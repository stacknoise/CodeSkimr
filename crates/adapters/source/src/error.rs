//! Errors of the source adapter (blueprint section 11.5).

use codeskimr_application::error::CodedError;
use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_domain::{BranchName, FolderPath};

use crate::redact::redact_credentials;

/// Why a source could not be opened, scanned or watched.
///
/// Texts of causes are stored with credentials removed from any URL (E-05).
#[derive(Debug, thiserror::Error)]
pub enum SourceError {
    /// The source folder does not exist or is not a readable folder.
    #[error("{}", ErrorCode::SourcePathNotFound.description())]
    PathNotFound {
        /// The folder that was configured.
        path: FolderPath,
    },
    /// The branch does not exist on the remote.
    #[error("{}", ErrorCode::GitBranchNotFound.description())]
    BranchNotFound {
        /// The requested branch.
        branch: BranchName,
    },
    /// The remote wants credentials that are not available.
    #[error("{}", ErrorCode::GitAuthRequired.description())]
    AuthRequired,
    /// The files of the source add up to more than the allowed size.
    #[error("{}", ErrorCode::SourceTooLarge.description())]
    TooLarge {
        /// The limit that was exceeded, in bytes.
        limit_bytes: u64,
    },
    /// Cloning or fetching failed.
    #[error("{}: {detail}", ErrorCode::GitFetchFailed.description())]
    GitFetchFailed {
        /// What the Git library reported, without credentials.
        detail: String,
    },
    /// The file watcher could not be started or stopped unexpectedly.
    #[error("{}: {detail}", ErrorCode::WatcherFailed.description())]
    WatcherFailed {
        /// What the notification library reported.
        detail: String,
    },
}

impl SourceError {
    /// A failed Git transfer; `cause` is shortened to its text with credentials removed.
    pub fn git_failed(cause: &dyn std::error::Error) -> Self {
        Self::GitFetchFailed {
            detail: redact_credentials(&error_chain_text(cause)),
        }
    }

    /// A failed watcher.
    pub fn watcher_failed(cause: &dyn std::error::Error) -> Self {
        Self::WatcherFailed {
            detail: error_chain_text(cause),
        }
    }
}

/// The messages of an error and all its sources, from outer to inner.
pub(crate) fn error_chain_text(error: &dyn std::error::Error) -> String {
    let mut parts = vec![error.to_string()];
    let mut next = error.source();
    while let Some(inner) = next {
        parts.push(inner.to_string());
        next = inner.source();
    }
    parts.join(crate::consts::CHAIN_SEPARATOR)
}

impl Coded for SourceError {
    fn code(&self) -> ErrorCode {
        match self {
            Self::PathNotFound { .. } => ErrorCode::SourcePathNotFound,
            Self::BranchNotFound { .. } => ErrorCode::GitBranchNotFound,
            Self::AuthRequired => ErrorCode::GitAuthRequired,
            Self::TooLarge { .. } => ErrorCode::SourceTooLarge,
            Self::GitFetchFailed { .. } => ErrorCode::GitFetchFailed,
            Self::WatcherFailed { .. } => ErrorCode::WatcherFailed,
        }
    }
}

impl From<SourceError> for CodedError {
    fn from(error: SourceError) -> Self {
        Self::caused_by(error)
    }
}
