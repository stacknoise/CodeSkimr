//! What the last scan of a project says about itself (ADR 0072, decision 9; adapter: `codeskimr_storage`).
//!
//! Only language ids, counts, reason keys and error codes are kept: no path, no tool name, no
//! output of an indexer.

use codeskimr_codes::ErrorCode;
use codeskimr_domain::{LanguageId, ProjectId, ScanLevel, ScanOrigin, SkipReason, Timestamp};

use crate::error::CodedError;

/// What level 2 did for one language.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LanguageOutcome {
    /// An indexer checked the language.
    Checked {
        /// Files whose references now come from the index.
        files: u64,
        /// Files that changed meanwhile and kept level 1.
        stale: u64,
    },
    /// Level 2 did not check the language, for this reason.
    Skipped(SkipReason),
    /// The run, the import or the storage failed with this code.
    Failed(ErrorCode),
}

/// The outcome for one language.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageLine {
    /// The language.
    pub language: LanguageId,
    /// What happened.
    pub outcome: LanguageOutcome,
}

/// The header of the newest scan of a project.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LastScan {
    /// When the scan wrote its snapshot.
    pub at: Timestamp,
    /// Level 2 if a pass checked at least one language in this scan.
    pub level: ScanLevel,
    /// The channel that started it.
    pub origin: ScanOrigin,
}

/// What the newest scan that ran the second pass said about each language. A scan without the
/// pass (watch mode, MCP) leaves it as it is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Level2Report {
    /// When the pass ended.
    pub at: Timestamp,
    /// The channel that started the scan.
    pub origin: ScanOrigin,
    /// One line per language.
    pub languages: Vec<LanguageLine>,
}

/// Both parts of the report of a project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanReport {
    /// The newest scan.
    pub last: LastScan,
    /// The newest second pass, if any scan has run one.
    pub level2: Option<Level2Report>,
}

/// The order to store the report of a finished scan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportWrite {
    /// The project that was scanned.
    pub project: ProjectId,
    /// The header of the scan.
    pub last: LastScan,
    /// The report of the second pass if the scan ran it, `None` otherwise.
    pub level2: Option<Level2Report>,
}

/// Keeps the report of the last scan per project.
pub trait ScanReportStore: Send + Sync {
    /// Replaces the header of the project; replaces the level 2 report only if `write` carries
    /// one. Nothing is stored for a project that does not exist.
    ///
    /// # Errors
    /// A coded failure of the `STO` group.
    fn store_report(&self, write: &ReportWrite) -> Result<(), CodedError>;

    /// The report of `project`; `None` before the first scan that stored one.
    ///
    /// # Errors
    /// A coded failure of the `STO` group.
    fn report(&self, project: ProjectId) -> Result<Option<ScanReport>, CodedError>;
}
