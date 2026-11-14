//! Errors of the indexer adapter, each mapped to one registered code.

use codeskimr_application::ports::ToolProblem;
use codeskimr_codes::{Coded, ErrorCode};

/// Why an indexer did not deliver an index. The index of level 1 stays in every case.
#[derive(Debug, thiserror::Error)]
pub enum IndexerError {
    /// The indexer or a tool of its toolchain is missing or too old.
    #[error("{}", ErrorCode::IndexerUnavailable.description())]
    Unavailable(Vec<ToolProblem>),
    /// The run took longer than `level2.timeout_seconds`.
    #[error("{}", ErrorCode::IndexerTimeout.description())]
    Timeout,
    /// The run wrote more than `level2.max_output_bytes`.
    #[error("{}", ErrorCode::IndexerOutputTooLarge.description())]
    OutputTooLarge,
    /// The run used more than `level2.max_memory_mb`.
    #[error("{}", ErrorCode::IndexerMemoryLimit.description())]
    MemoryLimit,
    /// The run could not be started, ended with an error or wrote no index.
    #[error("{}", ErrorCode::IndexerFailed.description())]
    Failed {
        /// What went wrong, without the output of the indexer.
        reason: FailureReason,
        /// The end of the console output, after redaction.
        excerpt: String,
    },
    /// The caller cancelled the run.
    #[error("{}", ErrorCode::ScanCancelled.description())]
    Cancelled,
}

/// What failed in a run that did not time out or hit a limit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FailureReason {
    /// The process could not be started or watched; the text is the operating system's.
    Io(String),
    /// The output folder could not be created; the text is the operating system's.
    OutputDir(String),
    /// The process ended with an error; the code is absent if a signal ended it.
    Exit(Option<i32>),
    /// The process ended well but wrote no index file.
    NoIndex,
}

impl Coded for IndexerError {
    fn code(&self) -> ErrorCode {
        match self {
            Self::Unavailable(_) => ErrorCode::IndexerUnavailable,
            Self::Timeout => ErrorCode::IndexerTimeout,
            Self::OutputTooLarge => ErrorCode::IndexerOutputTooLarge,
            Self::MemoryLimit => ErrorCode::IndexerMemoryLimit,
            Self::Failed { .. } => ErrorCode::IndexerFailed,
            Self::Cancelled => ErrorCode::ScanCancelled,
        }
    }
}
