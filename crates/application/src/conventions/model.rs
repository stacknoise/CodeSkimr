//! What a convention check is asked and what it answers.

use codeskimr_codes::ErrorCode;
use codeskimr_domain::{FolderPath, Principal, QualifiedName, RelativePath, SnapshotId, Span};

use crate::ports::SimilarityIndex;

/// One call of the check.
#[derive(Clone, Copy)]
pub struct CheckRequest<'a> {
    /// The caller; reading source needs `read_source`.
    pub principal: &'a Principal,
    /// Root folder of the project.
    pub root: &'a FolderPath,
    /// Files or folders below the root, with forward slashes. Empty means the whole project.
    /// A path that matches no file is ignored: paths only select, they are never opened.
    pub paths: &'a [String],
    /// The stored state of the project, to look for copies of code outside the selected files
    /// too; `None` compares the selected files with each other only.
    pub stored: Option<StoredProject<'a>>,
}

/// The newest snapshot of a project and the index of its fingerprints.
#[derive(Clone, Copy)]
pub struct StoredProject<'a> {
    /// Where the stored fingerprints are looked up.
    pub index: &'a dyn SimilarityIndex,
    /// The snapshot to compare with.
    pub snapshot: SnapshotId,
}

/// Where a finding is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    /// File of the function.
    pub path: RelativePath,
    /// Declaration range, zero-based.
    pub span: Span,
    /// The function, method or constructor.
    pub symbol: QualifiedName,
}

/// The numbers behind a finding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Detail {
    /// `CHK-104`: the function has more code lines than allowed.
    TooLong {
        /// Code lines of the function.
        lines: u32,
        /// The configured maximum.
        limit: u32,
    },
    /// `CHK-102`: the function is a near copy of an earlier one.
    Duplicate {
        /// The function that came first in path and line order.
        of: Location,
        /// Fingerprint similarity in percent, rounded down.
        similarity_percent: u32,
    },
}

/// One violation of the code rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// The registered `CHK` code.
    pub code: ErrorCode,
    /// The offending function.
    pub location: Location,
    /// The numbers behind it.
    pub detail: Detail,
}

/// A file that was selected but could not be analysed, with the code of the failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unchecked {
    /// The file.
    pub path: RelativePath,
    /// Why it was skipped, for example `PRS-501` for a timeout.
    pub code: ErrorCode,
}

/// The answer of one check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckReport {
    /// Number of files that were analysed.
    pub files_checked: u32,
    /// Findings, ordered by file, line and code.
    pub findings: Vec<Finding>,
    /// Selected files that could not be analysed.
    pub unchecked: Vec<Unchecked>,
    /// Requested paths that selected no file, as the check read them. A caller that gets one
    /// back should treat the check as not done: the path may be a typo.
    pub unmatched: Vec<String>,
    /// Rules of blueprint section 3 this version does not check yet; a clean report says nothing
    /// about them.
    pub not_checked: Vec<ErrorCode>,
}
