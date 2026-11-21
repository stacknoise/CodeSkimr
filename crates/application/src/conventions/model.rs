//! What a convention check is asked and what it answers.

use codeskimr_codes::ErrorCode;
use codeskimr_domain::{
    FolderPath, Layer, Principal, QualifiedName, RelativePath, SnapshotId, Span,
};

use crate::ports::{ReferenceSites, SimilarityIndex};

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

/// The newest snapshot of a project, the index of its fingerprints and its references.
#[derive(Clone, Copy)]
pub struct StoredProject<'a> {
    /// Where the stored fingerprints are looked up.
    pub index: &'a dyn SimilarityIndex,
    /// Where the stored references are looked up, for the layer rule.
    pub references: &'a dyn ReferenceSites,
    /// The layers of the project; the layer rule needs at least one with `may_use`.
    pub layers: &'a [Layer],
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
    /// `CHK-105` and `CHK-106`: a count above its maximum, the cognitive complexity or the number of
    /// parameters; the code of the finding says which.
    Over {
        /// The count of the function.
        value: u32,
        /// The configured maximum.
        limit: u32,
    },
    /// `CHK-101`: the function contains literals outside the exceptions of rule R-06.
    Literals {
        /// How many there are.
        count: u32,
    },
    /// `CHK-103`: the function uses a layer that its own layer may not use.
    Layering {
        /// The layer of the function.
        from_layer: String,
        /// The layer it reaches into.
        to_layer: String,
        /// How many references of the function point into that layer.
        count: u32,
    },
    /// `CHK-102`: the function is a near copy of an earlier one.
    Duplicate {
        /// The function that came first in path and line order.
        of: Location,
        /// Fingerprint similarity in percent, rounded down.
        similarity_percent: u32,
    },
}

/// The numbers of a [`Detail`] by name, for the callers that print or serialise a finding. A
/// field the detail does not have is `None`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Measure<'a> {
    /// `CHK-104`: code lines of the function.
    pub lines: Option<u32>,
    /// `CHK-105` and `CHK-106`: the cognitive complexity or the number of parameters.
    pub value: Option<u32>,
    /// The most that is allowed: the maximum of the lines or of the value.
    pub limit: Option<u32>,
    /// `CHK-101`: the number of literals in the function.
    pub literals: Option<u32>,
    /// `CHK-103`: the layer of the function and the layer it must not use, with the number of
    /// references into it.
    pub layering: Option<(&'a str, &'a str, u32)>,
    /// `CHK-102`: the earlier function this one copies.
    pub duplicate_of: Option<&'a Location>,
    /// `CHK-102`: the similarity in whole percent.
    pub similarity_percent: Option<u32>,
}

impl Detail {
    /// The numbers of this detail by name.
    #[must_use]
    pub fn measure(&self) -> Measure<'_> {
        match self {
            Self::TooLong { lines, limit } => Measure {
                lines: Some(*lines),
                limit: Some(*limit),
                ..Measure::default()
            },
            Self::Over { value, limit } => Measure {
                value: Some(*value),
                limit: Some(*limit),
                ..Measure::default()
            },
            Self::Literals { count } => Measure {
                literals: Some(*count),
                ..Measure::default()
            },
            Self::Layering {
                from_layer,
                to_layer,
                count,
            } => Measure {
                layering: Some((from_layer, to_layer, *count)),
                ..Measure::default()
            },
            Self::Duplicate {
                of,
                similarity_percent,
            } => Measure {
                duplicate_of: Some(of),
                similarity_percent: Some(*similarity_percent),
                ..Measure::default()
            },
        }
    }
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
