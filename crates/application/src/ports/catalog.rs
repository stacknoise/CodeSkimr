//! The overview of all registered projects (adapter: `codeskimr_storage`), for the project list.

use codeskimr_domain::{LanguageId, Project, ProjectId, Snapshot};

use super::{LeftOut, ScanWarning};
use crate::error::CodedError;

/// The share of one language in the newest snapshot of a project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageShare {
    /// The language.
    pub language: LanguageId,
    /// Number of its files.
    pub files: u32,
    /// Lines of code of its files.
    pub loc: u32,
    /// Number of symbols declared in its files.
    pub symbols: u32,
    /// Microseconds the last full scan spent parsing its files; `None` for a snapshot from before
    /// the figure was recorded.
    pub parse_micros: Option<u64>,
}

/// A project with what its newest snapshot says about it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectOverview {
    /// The registered project.
    pub project: Project,
    /// Its newest snapshot, `None` before the first scan.
    pub latest: Option<Snapshot>,
    /// Number of types (classes, interfaces, records and so on) in that snapshot.
    pub types: u32,
    /// Languages of that snapshot, the one with the most lines first.
    pub languages: Vec<LanguageShare>,
    /// The warnings of the scan that wrote it; empty if it had none or the snapshot is older than
    /// the record.
    pub warnings: Vec<ScanWarning>,
    /// The files that scan left out by name (ADR 0085); the warnings count all of them. Empty if
    /// it left none out or the snapshot is older than the record.
    pub left_out: Vec<LeftOut>,
    /// The removal of the project has begun and may be interrupted; it has no data to show.
    pub removing: bool,
    /// The file that holds the project; `None` for a store that keeps no files.
    pub database: Option<DatabaseInfo>,
}

/// What the file of a project is: it exists, it is missing, it cannot be opened, or it is being removed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseState {
    /// The file is there.
    Ok,
    /// The project is registered but its file is not there.
    Missing,
    /// The removal of the project has begun.
    Removing,
    /// The file is there but cannot be opened; the project is listed so that it can be removed.
    Unreadable,
}

/// Where the data of a project lies and how large it is (ADR 0095).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatabaseInfo {
    /// The path of the database file of the project.
    pub path: String,
    /// The size of the file with its write-ahead log, in bytes; 0 if it is missing.
    pub size_bytes: u64,
    /// Whether the file is there.
    pub state: DatabaseState,
}

/// Reads the registered projects.
pub trait ProjectCatalog: Send + Sync {
    /// All projects in the order they were registered.
    ///
    /// # Errors
    /// A coded failure of the `STO` group.
    fn overviews(&self) -> Result<Vec<ProjectOverview>, CodedError>;

    /// The languages of the newest snapshot of one project, the one with the most lines first;
    /// empty before the first scan.
    ///
    /// # Errors
    /// `PRJ-201` if the project does not exist, otherwise a coded failure of the `STO` group.
    fn languages(&self, project: ProjectId) -> Result<Vec<LanguageShare>, CodedError>;
}
