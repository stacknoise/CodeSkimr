//! What the second pass of a scan says about each language (ADR 0033, decision 12).

use codeskimr_codes::ErrorCode;
use codeskimr_domain::{LanguageId, Snapshot};

use crate::ports::ToolProblem;

/// Why level 2 did not run or did not change anything for a language. Level 1 stays in every case.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Skipped {
    /// The principal may not start indexers (`SEC-403`).
    NotPermitted,
    /// Level 2 is not enabled or not confirmed for the project (`SEC-301`).
    NotConfirmed,
    /// The indexer or its toolchain is missing or too old.
    Unavailable(Vec<ToolProblem>),
    /// Every file changed since the snapshot was written, so no index fits.
    Stale,
    /// The scan was cancelled; the references stay as level 1 made them.
    Cancelled,
    /// The run, the import or the storage failed with this code.
    Failed(ErrorCode),
}

/// What happened for one language.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Level2Outcome {
    /// The references of `files` files now come from the index; `stale` files changed after the
    /// scan and keep level 1.
    Checked {
        /// Files whose references were replaced.
        files: u64,
        /// Files of the language that kept level 1 because their content changed meanwhile.
        stale: u64,
        /// Uses that became references.
        mapped: u64,
        /// Uses that were dropped.
        dropped: u64,
    },
    /// Level 2 did not run or was not used.
    Skipped(Skipped),
}

/// The result for one language of the project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageReport {
    /// The language.
    pub language: LanguageId,
    /// What happened.
    pub outcome: Level2Outcome,
}

/// What the second pass leaves behind.
#[derive(Debug, Clone, PartialEq)]
pub struct Level2Result {
    /// One entry per language that has an indexer and files in the snapshot.
    pub reports: Vec<LanguageReport>,
    /// The snapshot with the totals after the replacement; the one the scan wrote if nothing
    /// was replaced.
    pub snapshot: Snapshot,
}
