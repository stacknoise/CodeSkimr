//! Errors of the SCIP import, each mapped to one registered code.

use codeskimr_codes::{Coded, ErrorCode};

/// What is wrong with the bytes of an index. The detail is for the log; the user sees one code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Corruption {
    /// The input ends inside a value.
    Truncated,
    /// A varint is longer than ten bytes.
    VarintTooLong,
    /// A field uses a wire type that its definition does not allow.
    WireType,
    /// Skipped groups are nested deeper than allowed.
    TooDeep,
    /// A text field is not valid UTF-8.
    NotUtf8,
    /// A text field is longer than allowed.
    TextTooLong,
    /// A number is outside the range of its field.
    NumberOutOfRange,
    /// A range has neither three nor four numbers, or ends before it starts.
    BadRange,
    /// A path is empty, absolute or leaves the project folder.
    UnsafePath,
    /// The index holds more distinct symbols than can be numbered.
    TooManySymbols,
}

/// Why an index could not be imported.
#[derive(Debug, thiserror::Error)]
pub enum ScipError {
    /// The bytes are not a well-formed index.
    #[error("{}", ErrorCode::ScipIndexInvalid.description())]
    Corrupt(Corruption),
    /// The index file could not be read.
    #[error("{}", ErrorCode::ScipIndexInvalid.description())]
    Unreadable(#[source] std::io::Error),
    /// The index is larger than the configured limit.
    #[error("{}", ErrorCode::ScipIndexTooLarge.description())]
    TooLarge {
        /// Size of the index in bytes (at least this much, if it was not read to the end).
        size: u64,
        /// The limit from `level2.max_index_bytes`.
        limit: u64,
    },
    /// Too few occurrences map onto project symbols (or no document matches a project file).
    #[error("{}", ErrorCode::ScipMappingTooLow.description())]
    MappingTooLow {
        /// Occurrences that were mapped.
        mapped: u64,
        /// Occurrences that were counted, mapped or not.
        total: u64,
    },
}

impl Coded for ScipError {
    fn code(&self) -> ErrorCode {
        match self {
            Self::Corrupt(_) | Self::Unreadable(_) => ErrorCode::ScipIndexInvalid,
            Self::TooLarge { .. } => ErrorCode::ScipIndexTooLarge,
            Self::MappingTooLow { .. } => ErrorCode::ScipMappingTooLow,
        }
    }
}

impl From<Corruption> for ScipError {
    fn from(detail: Corruption) -> Self {
        Self::Corrupt(detail)
    }
}
