//! The command behind the button "Open .codeskimr.toml" of the screen Rules: it opens the project
//! file of a folder project with the program of the operating system (ADR 0111).
//!
//! The path is built here from the folder of the registered project and the fixed name of the
//! file, never from the request. A link or anything but a file is refused, as when the file is
//! read. A missing file is created empty, exclusively, so that nothing there is ever followed or
//! overwritten.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use codeskimr_application::access;
use codeskimr_application::config::PROJECT_CONFIG_FILE;
use codeskimr_application::error::CodedError;
use codeskimr_application::registry::{Command, CommandDescriptor, CommandId, Request};
use codeskimr_codes::ErrorCode;
use codeskimr_domain::{Permission, ProjectId, ProjectSource};
use serde::{Deserialize, Serialize};

use super::consts::{
    EXECUTE_BITS, PROJECT_FILE_OPEN, PROJECT_FILE_OPEN_DESCRIPTION, PROJECT_FILE_OPEN_TITLE,
};
use super::explorer::ExplorerPorts;
use crate::file_opener::FileOpener;

/// The input of `project_file.open`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct OpenInput {
    /// Id of the project.
    pub project: u64,
}

/// What `project_file.open` reports back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Opened {
    /// The name of the file that was opened, without a folder.
    pub file: String,
    /// Whether the file did not exist and was created empty.
    pub created: bool,
}

/// Whether a file carries an execute bit (never on Windows). A project file has no reason to.
fn is_executable(meta: &fs::Metadata) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        meta.permissions().mode() & EXECUTE_BITS != 0
    }
    #[cfg(not(unix))]
    {
        let _ = meta;
        false
    }
}

/// Makes sure that `path` is a plain file: a missing one is created empty (and the answer says
/// so). A link, a folder or anything else is refused.
fn ensure_file(path: &Path) -> Result<bool, CodedError> {
    let refused = || CodedError::new(ErrorCode::ConfigParseFailed);
    match fs::symlink_metadata(path) {
        Ok(meta) if meta.is_file() && !is_executable(&meta) => Ok(false),
        Err(error) if error.kind() == ErrorKind::NotFound => fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map(|_| true)
            .map_err(|_| CodedError::new(ErrorCode::ProjectFileNotWritable)),
        Ok(_) | Err(_) => Err(refused()),
    }
}

/// The project file of the folder of `source`.
fn file_of(source: &ProjectSource) -> Result<PathBuf, CodedError> {
    let ProjectSource::Folder(folder) = source else {
        return Err(CodedError::new(ErrorCode::ProjectSourceInvalid));
    };
    Ok(Path::new(folder.as_str()).join(PROJECT_CONFIG_FILE))
}

/// Opens the project file of `source` with `opener`.
///
/// # Errors
/// - `PRJ-101` for a Git project.
/// - `CFG-102` if the file is a link, a folder, an executable file or cannot be looked at, `CFG-104` if a missing file
///   cannot be created.
/// - `APP-605` if no person sits at this machine (nothing is created then) or the system starts
///   no program.
pub(crate) fn open_project_file(
    opener: &dyn FileOpener,
    source: &ProjectSource,
) -> Result<Opened, CodedError> {
    let path = file_of(source)?;
    if !opener.available() {
        return Err(CodedError::new(ErrorCode::ProjectFileOpenFailed));
    }
    let created = ensure_file(&path)?;
    opener.open(&path)?;
    Ok(Opened {
        file: PROJECT_CONFIG_FILE.to_owned(),
        created,
    })
}

/// Opens the project file of a folder project.
pub struct ProjectFileOpen {
    pub(super) ports: Arc<ExplorerPorts>,
    pub(super) opener: Arc<dyn FileOpener>,
}

impl Command for ProjectFileOpen {
    type Input = OpenInput;
    type Output = Opened;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(PROJECT_FILE_OPEN),
        title_key: PROJECT_FILE_OPEN_TITLE,
        description_key: PROJECT_FILE_OPEN_DESCRIPTION,
        shortcut: None,
        read_only: false,
        permission: Permission::Manage,
    };

    /// # Errors
    /// `AUTH-401` without the right to manage, `PRJ-201` for an unknown project, and those of
    /// [`open_project_file`].
    fn execute(&self, request: Request<'_, Self>) -> Result<Opened, CodedError> {
        access::require(request.principal, Permission::Manage)?;
        let project = self
            .ports
            .projects
            .project(ProjectId::new(request.input.project))?;
        open_project_file(self.opener.as_ref(), &project.source)
    }
}
