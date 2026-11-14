//! `codeskimr level2 confirm|revoke`: the one place that allows or forbids level 2 for a project
//! (ADR 0033, decision 10). It is a local, interactive command of the owner of this computer: the
//! caller supplies the question, and nothing else (MCP, REST, flags, watch mode, CI) reaches
//! `level2::confirm`. A test (`tests/consent_origin.rs`) keeps it that way.

use codeskimr_application::error::CodedError;
use codeskimr_application::level2;
use codeskimr_cli::{GlobalOptions, Level2Action};
use codeskimr_domain::ProjectId;

use crate::locations::Locations;
use crate::wiring::open_store;
use crate::{config_files, export, principal};

/// What the command did; the text is the project as the user named it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Done {
    /// Level 2 is allowed now.
    Confirmed(String),
    /// The user did not confirm; nothing changed.
    Declined(String),
    /// Level 2 is forbidden again.
    Revoked(String),
}

/// Runs `action`. `ask` shows the warning and asks the user; it returns whether they confirmed,
/// or fails if there is nobody to ask. It is called only for `Confirm`, after the project is found.
///
/// # Errors
/// `PRJ-201` if the project is not registered, `SEC-403` if `ask` finds no interactive terminal,
/// otherwise any coded failure of the configuration or the storage.
pub fn run(
    action: &Level2Action,
    global: &GlobalOptions,
    locations: &Locations,
    ask: &dyn Fn() -> Result<bool, CodedError>,
) -> Result<Done, CodedError> {
    let config = config_files::load(locations, global, None)?;
    let store = open_store(locations, &config)?;
    let owner = principal::resolve();
    match action {
        Level2Action::Confirm { project } => {
            let id = ProjectId::new(export::resolve(project, &store)?);
            if !ask()? {
                return Ok(Done::Declined(project.clone()));
            }
            level2::confirm(&owner, store.as_ref(), id)?;
            Ok(Done::Confirmed(project.clone()))
        }
        Level2Action::Revoke { project } => {
            let id = ProjectId::new(export::resolve(project, &store)?);
            level2::revoke(&owner, store.as_ref(), id)?;
            Ok(Done::Revoked(project.clone()))
        }
    }
}
