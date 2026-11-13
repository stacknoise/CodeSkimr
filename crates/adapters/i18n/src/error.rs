//! Errors of the i18n adapter, each mapped to one registered code.

use codeskimr_codes::{Coded, ErrorCode};

/// Why a message could not be produced.
#[derive(Debug, thiserror::Error)]
pub enum I18nError {
    /// The key exists neither in the requested language nor in the default language.
    #[error("{}", ErrorCode::MessageKeyMissing.description())]
    MessageKeyMissing {
        /// The message key that was requested.
        key: String,
        /// The language that was requested.
        language: String,
    },
    /// An embedded Fluent file is invalid. The tests of this crate prevent this at build time.
    #[error("{}", ErrorCode::UnexpectedInternalError.description())]
    InvalidResource {
        /// Language folder of the file.
        language: String,
        /// File name such as `app.ftl`.
        file: String,
    },
}

impl Coded for I18nError {
    fn code(&self) -> ErrorCode {
        match self {
            Self::MessageKeyMissing { .. } => ErrorCode::MessageKeyMissing,
            Self::InvalidResource { .. } => ErrorCode::UnexpectedInternalError,
        }
    }
}
