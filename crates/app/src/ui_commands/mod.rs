//! The commands the browser UI calls over REST, registered once in the registry and exposed in
//! the server (blueprint 10.1). The scan runs on the thread of the call; the server runs every
//! command on its blocking pool, and progress reaches the UI through the event bus.

mod consts;
mod explorer;
mod projects;
mod rating;
mod running;
mod search;
mod source;
mod status;
mod symbol;
#[cfg(test)]
mod tests;

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use codeskimr_application::config::MetricThresholds;
use codeskimr_application::error::CodedError;
use codeskimr_application::events::EventBus;
use codeskimr_application::registry::Registry;
use codeskimr_cli::GlobalOptions;
use codeskimr_server::CommandApi;
use codeskimr_source::FolderText;

pub(crate) use status::{mark_ready, mark_started};

use crate::config_files;
use crate::locations::Locations;
use crate::wiring::open_store;
use codeskimr_storage::SqliteStore;
use explorer::{ExplorerOutline, ExplorerPorts};
use projects::{AddProject, ListProjects, RescanProject};
use running::RunningScans;
use search::{SearchPorts, SearchQuery};
use source::ExplorerSource;
use status::{BudgetTargets, StatusInfo};
use symbol::ExplorerSymbol;

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
    register_projects(&mut registry, &catalog, &context)?;
    register_search(&mut registry, &catalog)?;
    registry
        .register(StatusInfo {
            budgets: BudgetTargets::from(&config.budgets),
        })
        .map_err(CodedError::caused_by)?;
    register_explorer(&mut registry, catalog, config.metrics.clone())?;
    let mut api = CommandApi::new(Arc::new(registry));
    api.expose::<ListProjects>()?;
    api.expose::<AddProject>()?;
    api.expose::<RescanProject>()?;
    api.expose::<ExplorerOutline>()?;
    api.expose::<ExplorerSymbol>()?;
    api.expose::<ExplorerSource>()?;
    api.expose::<SearchQuery>()?;
    api.expose::<StatusInfo>()?;
    Ok(api)
}

fn register_projects(
    registry: &mut Registry,
    catalog: &Arc<SqliteStore>,
    context: &Arc<Context>,
) -> Result<(), CodedError> {
    registry
        .register(ListProjects {
            catalog: catalog.clone(),
            context: Arc::clone(context),
        })
        .map_err(CodedError::caused_by)?;
    registry
        .register(AddProject {
            context: Arc::clone(context),
        })
        .map_err(CodedError::caused_by)?;
    registry
        .register(RescanProject {
            catalog: catalog.clone(),
            context: Arc::clone(context),
        })
        .map_err(CodedError::caused_by)
}

fn register_search(registry: &mut Registry, store: &Arc<SqliteStore>) -> Result<(), CodedError> {
    let ports = Arc::new(SearchPorts {
        projects: store.clone(),
        index: store.clone(),
    });
    registry
        .register(SearchQuery { ports })
        .map_err(CodedError::caused_by)
}

fn register_explorer(
    registry: &mut Registry,
    store: Arc<SqliteStore>,
    thresholds: MetricThresholds,
) -> Result<(), CodedError> {
    let ports = Arc::new(ExplorerPorts {
        projects: store.clone(),
        index: store.clone(),
        outline: store,
        text: Arc::new(FolderText),
        thresholds,
    });
    registry
        .register(ExplorerOutline {
            ports: Arc::clone(&ports),
        })
        .map_err(CodedError::caused_by)?;
    registry
        .register(ExplorerSource {
            ports: Arc::clone(&ports),
        })
        .map_err(CodedError::caused_by)?;
    registry
        .register(ExplorerSymbol { ports })
        .map_err(CodedError::caused_by)
}
