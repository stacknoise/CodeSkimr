//! Errors of the domain model.

use std::fmt;

use codeskimr_codes::{Coded, ErrorCode};

use crate::consts::{
    MESSAGE_EMPTY, MESSAGE_INVERTED_SPAN, MESSAGE_NOT_FINITE, MESSAGE_UNKNOWN_LABEL,
};

/// A value violates a rule of the domain model.
///
/// Values reach the model from parsers, storage and the application layer, so a violation is a bug
/// in the producer and maps to the internal-error code; callers that take user input validate it first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelError {
    /// A text value is empty or only whitespace.
    Empty,
    /// A span ends before it starts.
    InvertedSpan,
    /// A label does not name any variant of the enumeration.
    UnknownLabel,
    /// A number is NaN or infinite.
    NotFinite,
}

impl Coded for ModelError {
    fn code(&self) -> ErrorCode {
        ErrorCode::UnexpectedInternalError
    }
}

impl fmt::Display for ModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Empty => MESSAGE_EMPTY,
            Self::InvertedSpan => MESSAGE_INVERTED_SPAN,
            Self::UnknownLabel => MESSAGE_UNKNOWN_LABEL,
            Self::NotFinite => MESSAGE_NOT_FINITE,
        })
    }
}

impl std::error::Error for ModelError {}
