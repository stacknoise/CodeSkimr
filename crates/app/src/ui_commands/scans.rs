//! The command of the list of the last scans of a project (ADR 0097): when each ran, how long it
//! took, how many files it saw and how many it changed.

use std::sync::Arc;

use codeskimr_application::access;
use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{CodeIndex, ProjectRepository, ScanEntry};
use codeskimr_application::registry::{Command, CommandDescriptor, CommandId, Request};
use codeskimr_domain::{CommitHash, Permission, ProjectId};
use serde::{Deserialize, Serialize};

use super::consts::{SCANS_LIST, SCANS_LIST_DESCRIPTION, SCANS_LIST_TITLE};

/// The input of `scans.list`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct ListInput {
    /// Id of the project.
    pub project: u64,
}

/// One scan of the list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScanOut {
    /// Id of the snapshot the scan wrote; two of them can be compared.
    pub snapshot: u64,
    /// When the last scan that wrote or confirmed the snapshot ran, in seconds since the epoch.
    pub created: i64,
    /// Commit, if the source is a Git repository.
    pub commit: Option<String>,
    /// `full` or `incremental`.
    pub mode: &'static str,
    /// Number of files of the snapshot.
    pub files: u32,
    /// Number of symbols of the snapshot.
    pub symbols: u32,
    /// Wall time of the scan that wrote the snapshot in milliseconds; `None` for a snapshot
    /// written before the time was stored.
    pub duration_ms: Option<u64>,
    /// Files that scan changed against the previous snapshot; `None` for the first scan and for
    /// a snapshot written before this was stored.
    pub files_changed: Option<u32>,
}

/// The output of `scans.list`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ListOut {
    /// The retained scans, newest first.
    pub scans: Vec<ScanOut>,
    /// How many scans the project keeps (`snapshots.retention_count`).
    pub keep: u32,
    /// Average duration in milliseconds of the scans that have one; `None` if none has.
    pub average_ms: Option<u64>,
}

/// What the command reads.
pub struct ScanListPorts {
    pub(super) projects: Arc<dyn ProjectRepository>,
    pub(super) index: Arc<dyn CodeIndex>,
    /// `snapshots.retention_count`.
    pub(super) keep: u32,
}

/// Lists the retained scans of a project, newest first.
pub struct ScansList {
    pub(super) ports: Arc<ScanListPorts>,
}

fn scan_out(entry: &ScanEntry) -> ScanOut {
    ScanOut {
        snapshot: entry.snapshot.get(),
        created: entry.created.0,
        commit: entry.commit.as_ref().map(CommitHash::to_string),
        mode: entry.mode.label(),
        files: entry.files,
        symbols: entry.symbols,
        duration_ms: entry.duration_ms,
        files_changed: entry.files_changed,
    }
}

fn average(scans: &[ScanOut]) -> Option<u64> {
    let times: Vec<u64> = scans.iter().filter_map(|scan| scan.duration_ms).collect();
    let count = u64::try_from(times.len()).ok().filter(|count| *count > 0)?;
    Some(times.iter().sum::<u64>() / count)
}

impl Command for ScansList {
    type Input = ListInput;
    type Output = ListOut;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(SCANS_LIST),
        title_key: SCANS_LIST_TITLE,
        description_key: SCANS_LIST_DESCRIPTION,
        shortcut: None,
        read_only: true,
        permission: Permission::Read,
    };

    fn execute(&self, request: Request<'_, Self>) -> Result<ListOut, CodedError> {
        access::require(request.principal, Permission::Read)?;
        let project = ProjectId::new(request.input.project);
        self.ports.projects.project(project)?;
        let history = self.ports.index.scan_history(project)?;
        let scans: Vec<ScanOut> = history.iter().map(scan_out).collect();
        Ok(ListOut {
            average_ms: average(&scans),
            keep: self.ports.keep,
            scans,
        })
    }
}
