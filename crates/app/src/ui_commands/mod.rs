//! The commands the browser UI calls over REST, registered once in the registry and exposed in
//! the server (blueprint 10.1). The scan runs on the thread of the call; the server runs every
//! command on its blocking pool, and progress reaches the UI through the event bus.

mod consts;
mod projects;
mod running;
#[cfg(test)]
mod tests;

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use codeskimr_application::error::CodedError;
use codeskimr_application::events::EventBus;
use codeskimr_application::registry::Registry;
use codeskimr_cli::GlobalOptions;
use codeskimr_server::CommandApi;

use crate::config_files;
use crate::locations::Locations;
use crate::wiring::open_store;
use projects::{AddProject, ListProjects, RescanProject};
use running::RunningScans;

/// What every command of the UI needs to scan: the command line, the folders and the event bus.
pub struct Context {
    global: GlobalOptions,
    locations: Locations,
    events: EventBus,
    running: RunningScans,
    registration: Mutex<()>,
}

impl Context {
    /// Held while a source is registered, so two adds of the same new source cannot both pick
    /// the next project id.
    fn registering(&self) -> MutexGuard<'_, ()> {
        self.registration
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }
}

/// Registers the commands and exposes them over REST.
///
/// # Errors
/// A coded failure of the configuration or of opening the database; `APP-901` if the wiring is
/// wrong (a command registered twice).
pub fn command_api(
    global: &GlobalOptions,
    locations: &Locations,
    events: EventBus,
) -> Result<CommandApi, CodedError> {
    let config = config_files::load(locations, global, None)?;
    let catalog = open_store(locations, &config)?;
    let context = Arc::new(Context {
        global: global.clone(),
        locations: locations.clone(),
        events,
        running: RunningScans::default(),
        registration: Mutex::new(()),
    });
    let mut registry = Registry::new();
    registry
        .register(ListProjects {
            catalog: catalog.clone(),
            context: Arc::clone(&context),
        })
        .map_err(CodedError::caused_by)?;
    registry
        .register(AddProject {
            context: Arc::clone(&context),
        })
        .map_err(CodedError::caused_by)?;
    registry
        .register(RescanProject { catalog, context })
        .map_err(CodedError::caused_by)?;
    let mut api = CommandApi::new(Arc::new(registry));
    api.expose::<ListProjects>()?;
    api.expose::<AddProject>()?;
    api.expose::<RescanProject>()?;
    Ok(api)
}
