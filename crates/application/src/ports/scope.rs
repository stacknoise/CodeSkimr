//! The scope of a project: what its scans leave out, and a look at what lies in the folder
//! (ADR 0101).

use codeskimr_domain::{FolderPath, LanguageId, ProjectId, RelativePath, Scope};

use crate::cancel::CancelToken;
use crate::error::CodedError;

/// Writes the scope of a project (adapter: `codeskimr_storage`). The scope is read through
/// [`crate::ports::ProjectRepository::scope`].
pub trait ScopeStore: Send + Sync {
    /// Replaces the scope of `project`.
    ///
    /// # Errors
    /// `PRJ-201` if the project does not exist, otherwise a coded failure of the `STO` group.
    fn store_scope(&self, project: ProjectId, scope: &Scope) -> Result<(), CodedError>;
}

/// A file below the project folder, as the walk of a scan sees it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListedFile {
    /// Path inside the project.
    pub path: RelativePath,
    /// The language of the file, `None` for a file that no pack claims (`Cargo.toml`, `README`).
    pub language: Option<LanguageId>,
}

/// Lists the files of a folder the way a scan walks it (ignore rules, excluded folders), without
/// the scope and without reading a file.
pub trait FolderLister: Send + Sync {
    /// The files below `root` in path order. Cancellation works as for `FileDiscovery::discover`.
    ///
    /// # Errors
    /// `SRC-201` if the folder or a part of it cannot be read.
    fn list(&self, root: &FolderPath, cancel: &CancelToken) -> Result<Vec<ListedFile>, CodedError>;
}
