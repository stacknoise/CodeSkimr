//! Errors of the domain model.

use std::fmt;

use codeskimr_codes::{Coded, ErrorCode};

use crate::consts::{
    MESSAGE_EMPTY, MESSAGE_EMPTY_TEXT, MESSAGE_INCONSISTENT, MESSAGE_INVERTED_SPAN,
    MESSAGE_NOT_FINITE, MESSAGE_NOT_POSITIVE, MESSAGE_TOO_LONG, MESSAGE_UNKNOWN_LABEL,
    MESSAGE_UNKNOWN_NODE,
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
    /// A count that must be at least one is zero.
    NotPositive,
    /// An edge names a node that does not exist.
    UnknownNode,
    /// A text of user input (an annotation text or title) is empty or only whitespace (ADR 0068).
    EmptyText,
    /// A text of user input is longer than the limit (ADR 0068).
    TooLong,
    /// The fields of an annotation do not fit its kind, for example a due date on a note.
    Inconsistent,
}

impl Coded for ModelError {
    fn code(&self) -> ErrorCode {
        match self {
            Self::TooLong => ErrorCode::AnnotationTextTooLong,
            Self::Inconsistent | Self::EmptyText => ErrorCode::AnnotationInvalid,
            _ => ErrorCode::UnexpectedInternalError,
        }
    }
}

impl fmt::Display for ModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Empty => MESSAGE_EMPTY,
            Self::InvertedSpan => MESSAGE_INVERTED_SPAN,
            Self::UnknownLabel => MESSAGE_UNKNOWN_LABEL,
            Self::NotFinite => MESSAGE_NOT_FINITE,
            Self::NotPositive => MESSAGE_NOT_POSITIVE,
            Self::UnknownNode => MESSAGE_UNKNOWN_NODE,
            Self::TooLong => MESSAGE_TOO_LONG,
            Self::EmptyText => MESSAGE_EMPTY_TEXT,
            Self::Inconsistent => MESSAGE_INCONSISTENT,
        })
    }
}

impl std::error::Error for ModelError {}
