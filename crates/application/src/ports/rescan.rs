//! Starting an incremental scan from a driving adapter (implemented by the `Scanner`).

use codeskimr_codes::ErrorCode;
use codeskimr_domain::{Principal, ProjectId, RelativePath, Snapshot};

use crate::error::CodedError;

/// What a scan did with the `paths` it was asked to look at.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PathsReport {
    /// The paths that name no file of the project, neither one found now nor one of the previous
    /// snapshot (a typo, a path outside the project), as the caller wrote them.
    pub unmatched: Vec<String>,
    /// Whether the scan ignored the paths and looked at the whole project, because it had to be a
    /// full scan.
    pub ignored: bool,
    /// The files the scan could not analyse and left out, in the order of the scan; at most
    /// `scan.max_left_out_listed` of them once the scan has ended.
    pub left_out: Vec<LeftOut>,
    /// How many more files were left out than `left_out` names.
    pub left_out_more: u64,
}

/// A file that a scan left out because it alone could not be analysed. The scan goes on without
/// it; any other failure of the analysis stops the scan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeftOut {
    /// The file, relative to the root of the project.
    pub path: RelativePath,
    /// Why: one of [`LeftOut::CODES`].
    pub code: ErrorCode,
    /// Whether the snapshot still shows the file as the previous scan saw it. An incremental scan
    /// keeps that for a failure that may pass (see [`LeftOut::MAY_PASS`]); otherwise the file is
    /// missing in the snapshot.
    pub kept: bool,
}

impl LeftOut {
    /// The codes of failures that concern one file only: its encoding, the time its parsing took,
    /// or that it could not be read.
    pub const CODES: &[ErrorCode] = &[
        ErrorCode::FileEncodingUnsupported,
        ErrorCode::ParseTimeout,
        ErrorCode::FileUnreadable,
    ];
    /// The codes of failures that may be gone at the next scan: a parse that took too long while
    /// the machine was busy, a file that another program had locked for a moment.
    pub const MAY_PASS: &[ErrorCode] = &[ErrorCode::ParseTimeout, ErrorCode::FileUnreadable];
}

/// The snapshot of a rescan and what became of its paths.
#[derive(Debug, Clone, PartialEq)]
pub struct Rescanned {
    /// The snapshot the scan wrote.
    pub snapshot: Snapshot,
    /// What the scan did with the paths; empty if there were none.
    pub paths: PathsReport,
}

/// Scans a registered project again, taking over what did not change.
pub trait ProjectScanner: Send + Sync {
    /// Runs an incremental scan of `project` for `principal`, looking for changes only in the
    /// files and folders `paths` name (all of the project if there are none), and returns the
    /// snapshot it wrote. A file the paths do not name stays as the previous snapshot has it, also
    /// if it changed or is gone since. A path is only compared with the files found below the
    /// project root, never opened (blueprint S-05), so a path that leaves the project names nothing.
    ///
    /// # Errors
    /// `AUTH-401` if the principal may not scan, otherwise the code of the failed stage;
    /// a cancelled scan is `PRJ-303`.
    fn rescan_paths(
        &self,
        principal: &Principal,
        project: ProjectId,
        paths: &[String],
    ) -> Result<Rescanned, CodedError>;

    /// Runs an incremental scan of the whole `project`.
    ///
    /// # Errors
    /// As [`ProjectScanner::rescan_paths`].
    fn rescan(&self, principal: &Principal, project: ProjectId) -> Result<Snapshot, CodedError> {
        self.rescan_paths(principal, project, &[])
            .map(|rescanned| rescanned.snapshot)
    }
}
