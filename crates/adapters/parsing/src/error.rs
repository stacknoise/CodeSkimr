//! Errors of the parsing adapter, each mapped to one registered code.

use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_domain::ModelError;

/// Why a file could not be analysed or a language pack could not be loaded.
#[derive(Debug, thiserror::Error)]
pub enum ParsingError {
    /// The file is neither valid UTF-8 nor UTF-16.
    #[error("{}", ErrorCode::FileEncodingUnsupported.description())]
    EncodingUnsupported,
    /// The file could not be read.
    #[error("{}", ErrorCode::SourcePathNotFound.description())]
    Unreadable(#[source] std::io::Error),
    /// No language pack is installed for the language of the file.
    #[error("{}", ErrorCode::LanguagePackMissing.description())]
    PackMissing(String),
    /// A pack's metadata or queries are invalid.
    #[error("{}", ErrorCode::LanguagePackInvalid.description())]
    PackInvalid {
        /// Folder name of the pack.
        pack: String,
        /// What is wrong, for the log.
        reason: String,
    },
    /// Parsing took longer than the configured limit.
    #[error("{}", ErrorCode::ParseTimeout.description())]
    Timeout,
    /// A value violated the domain model, which is a bug of this adapter.
    #[error(transparent)]
    Model(#[from] ModelError),
}

impl ParsingError {
    /// Shorthand for an invalid pack.
    pub fn invalid(pack: impl Into<String>, reason: String) -> Self {
        Self::PackInvalid {
            pack: pack.into(),
            reason,
        }
    }
}

impl Coded for ParsingError {
    fn code(&self) -> ErrorCode {
        match self {
            Self::EncodingUnsupported => ErrorCode::FileEncodingUnsupported,
            Self::Unreadable(_) => ErrorCode::SourcePathNotFound,
            Self::PackMissing(_) => ErrorCode::LanguagePackMissing,
            Self::PackInvalid { .. } => ErrorCode::LanguagePackInvalid,
            Self::Timeout => ErrorCode::ParseTimeout,
            Self::Model(error) => error.code(),
        }
    }
}
