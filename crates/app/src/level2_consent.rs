//! `codeskimr level2 confirm|revoke`: the one place that allows or forbids level 2 for a project
//! (ADR 0033, decision 10). It is a local, interactive command of the owner of this computer: the
//! caller supplies the question, and nothing else (MCP, REST, flags, watch mode, CI) reaches
//! `level2::confirm`. A test (`tests/consent_origin.rs`) keeps it that way.
//!
//! The user confirms by typing the name of the project, after seeing which project, where and at
//! which branch that is. The project is read again after the answer, and nothing is stored if it
//! no longer is what was shown.

use codeskimr_application::error::CodedError;
use codeskimr_application::level2;
use codeskimr_application::ports::ProjectRepository;
use codeskimr_cli::{GlobalOptions, Level2Action};
use codeskimr_codes::ErrorCode;
use codeskimr_domain::{Project, ProjectId, ProjectSource};

use crate::locations::Locations;
use crate::wiring::open_store;
use crate::{config_files, export, principal};

/// What the command did; the text is the name of the project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Done {
    /// Level 2 is allowed now.
    Confirmed(String),
    /// The user did not confirm; nothing changed.
    Declined(String),
    /// Level 2 is forbidden again.
    Revoked(String),
}

/// The project the user is asked about, as the project record says it: the name they have to type
/// and where the code comes from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    /// Name of the project; the user types it to confirm.
    pub name: String,
    /// The folder, or the Git address.
    pub place: String,
    /// The Git branch; `None` for a folder.
    pub branch: Option<String>,
}

impl Target {
    fn of(project: &Project) -> Self {
        let (place, branch) = match &project.source {
            ProjectSource::Folder(folder) => (folder.to_string(), None),
            ProjectSource::Git { url, branch, .. } => {
                (url.as_str().to_owned(), Some(branch.to_string()))
            }
        };
        Self {
            name: project.name.to_string(),
            place,
            branch,
        }
    }
}

/// Runs `action`. `ask` shows the warning and the [`Target`] and returns what the user typed, or
/// fails if there is nobody to ask. It is called only for `Confirm`, after the project is found.
/// The user confirms only by typing the name of the project.
///
/// # Errors
/// `PRJ-201` if the project is not registered, `SEC-403` if `ask` finds no interactive terminal
/// or the project changed while the user was asked, otherwise any coded failure of the
/// configuration or the storage.
pub fn run(
    action: &Level2Action,
    global: &GlobalOptions,
    locations: &Locations,
    ask: &dyn Fn(&Target) -> Result<String, CodedError>,
) -> Result<Done, CodedError> {
    let config = config_files::load(locations, global, None)?;
    let store = open_store(locations, &config)?;
    let owner = principal::resolve();
    match action {
        Level2Action::Confirm { project } => {
            let id = ProjectId::new(export::resolve(project, &store)?);
            let shown = store.project(id)?;
            let target = Target::of(&shown);
            if ask(&target)?.trim() != target.name {
                return Ok(Done::Declined(target.name));
            }
            let now = store.project(id)?;
            if now.name != shown.name || now.source != shown.source {
                return Err(CodedError::new(ErrorCode::IndexerNotPermitted));
            }
            level2::confirm(&owner, store.as_ref(), id)?;
            Ok(Done::Confirmed(target.name))
        }
        Level2Action::Revoke { project } => {
            let id = ProjectId::new(export::resolve(project, &store)?);
            let name = store.project(id)?.name.to_string();
            level2::revoke(&owner, store.as_ref(), id)?;
            Ok(Done::Revoked(name))
        }
    }
}
