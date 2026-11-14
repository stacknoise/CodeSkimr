//! The overview of all registered projects (adapter: `codeskimr_storage`), for the project list.

use codeskimr_domain::{LanguageId, Project, ProjectId, Snapshot};

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
