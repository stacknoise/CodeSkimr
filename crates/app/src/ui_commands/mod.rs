//! The commands the browser UI calls over REST, registered once in the registry and exposed in
//! the server (blueprint 10.1). The scan runs on the thread of the call; the server runs every
//! command on its blocking pool, and progress reaches the UI through the event bus.

mod consts;
mod credentials;
#[cfg(test)]
mod credentials_tests;
mod explorer;
mod export;
#[cfg(test)]
mod export_tests;
mod graph;
mod graph_types;
mod projects;
mod rating;
mod running;
mod search;
mod source;
mod status;
mod symbol;
#[cfg(test)]
mod tests;
mod usages;
mod usages_code;

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock, PoisonError};

use codeskimr_application::config::{Config, Graph, MetricThresholds};
use codeskimr_application::error::CodedError;
use codeskimr_application::events::EventBus;
use codeskimr_application::registry::Registry;
use codeskimr_cli::GlobalOptions;
use codeskimr_server::CommandApi;
use codeskimr_source::FolderText;

pub(crate) use status::{mark_ready, mark_started};

use crate::config_files;
use crate::locations::Locations;
use crate::wiring::{Secrets, open_store};
use codeskimr_storage::SqliteStore;
use credentials::{DeleteCredential, ListCredentials, SetCredential};
use explorer::{ExplorerOutline, ExplorerPorts};
pub(crate) use export::{ExportInput, HtmlExport};
use graph::{GraphLayers, GraphPorts};
use projects::{AddProject, CancelScan, ListProjects, RescanProject};
use running::RunningScans;
use search::{SearchPorts, SearchQuery};
use source::ExplorerSource;
use status::{BudgetTargets, StatusInfo};
use symbol::ExplorerSymbol;
use usages::{UsagesPorts, UsagesQuery};
use usages_code::UsagesCode;

/// What every command of the UI needs to scan: the command line, the folders and the event bus.
pub struct Context {
    global: GlobalOptions,
    locations: Locations,
    events: EventBus,
    secrets: Arc<Secrets>,
    running: RunningScans,
    registration: Mutex<()>,
    tiers: OnceLock<BTreeMap<String, &'static str>>,
}

impl Context {
    /// The tier of a language pack, `full` or `basic`; `None` if no installed pack has that id (or
    /// the packs cannot be read, which the scan reports on its own).
    fn tier_of(&self, language: &str) -> Option<&'static str> {
        if self.tiers.get().is_none() {
            // Only a table that was read is kept: a folder that is not there yet is tried again
            // at the next call, not for the lifetime of the server.
            let found = codeskimr_parsing::tiers(&self.locations.languages_dir).ok()?;
            let table = found
                .into_iter()
                .map(|(id, tier)| (id, tier.label()))
                .collect();
            return self.tiers.get_or_init(|| table).get(language).copied();
        }
        self.tiers.get()?.get(language).copied()
    }

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
    secrets: Arc<Secrets>,
) -> Result<CommandApi, CodedError> {
    let config = config_files::load(locations, global, None)?;
    let catalog = open_store(locations, &config)?;
    let context = Arc::new(Context {
        global: global.clone(),
        locations: locations.clone(),
        events,
        secrets,
        running: RunningScans::default(),
        registration: Mutex::new(()),
        tiers: OnceLock::new(),
    });
    let mut registry = Registry::new();
    register_projects(&mut registry, &catalog, &context)?;
    register_credentials(&mut registry, &catalog, &context)?;
    register_search(&mut registry, &catalog)?;
    register_usages(&mut registry, &catalog, config.mcp.max_depth)?;
    register_graph(&mut registry, &catalog, config.graph.clone())?;
    registry
        .register(StatusInfo {
            budgets: BudgetTargets::from(&config.budgets),
        })
        .map_err(CodedError::caused_by)?;
    register_explorer(&mut registry, &catalog, config.metrics.clone())?;
    registry
        .register(export_command(&catalog, &config, locations))
        .map_err(CodedError::caused_by)?;
    let mut api = CommandApi::new(Arc::new(registry));
    expose_all(&mut api)?;
    Ok(api)
}

/// Makes every registered command of the UI reachable over REST.
fn expose_all(api: &mut CommandApi) -> Result<(), CodedError> {
    api.expose::<ListProjects>()?;
    api.expose::<AddProject>()?;
    api.expose::<RescanProject>()?;
    api.expose::<CancelScan>()?;
    api.expose::<ListCredentials>()?;
    api.expose::<SetCredential>()?;
    api.expose::<DeleteCredential>()?;
    api.expose::<ExplorerOutline>()?;
    api.expose::<ExplorerSymbol>()?;
    api.expose::<ExplorerSource>()?;
    api.expose::<SearchQuery>()?;
    api.expose::<UsagesQuery>()?;
    api.expose::<UsagesCode>()?;
    api.expose::<GraphLayers>()?;
    api.expose::<StatusInfo>()?;
    api.expose::<HtmlExport>()?;
    Ok(())
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
        .map_err(CodedError::caused_by)?;
    registry
        .register(CancelScan {
            context: Arc::clone(context),
        })
        .map_err(CodedError::caused_by)
}

fn register_credentials(
    registry: &mut Registry,
    catalog: &Arc<SqliteStore>,
    context: &Arc<Context>,
) -> Result<(), CodedError> {
    registry
        .register(ListCredentials {
            catalog: catalog.clone(),
            secrets: Arc::clone(&context.secrets),
        })
        .map_err(CodedError::caused_by)?;
    registry
        .register(SetCredential {
            secrets: Arc::clone(&context.secrets),
        })
        .map_err(CodedError::caused_by)?;
    registry
        .register(DeleteCredential {
            secrets: Arc::clone(&context.secrets),
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

fn register_usages(
    registry: &mut Registry,
    store: &Arc<SqliteStore>,
    max_depth: u32,
) -> Result<(), CodedError> {
    let ports = usages_ports(store, max_depth);
    registry
        .register(UsagesQuery {
            ports: Arc::clone(&ports),
        })
        .map_err(CodedError::caused_by)?;
    registry
        .register(UsagesCode { ports })
        .map_err(CodedError::caused_by)
}

fn register_graph(
    registry: &mut Registry,
    store: &Arc<SqliteStore>,
    look: Graph,
) -> Result<(), CodedError> {
    registry
        .register(GraphLayers {
            ports: graph_ports(store, look),
        })
        .map_err(CodedError::caused_by)
}

fn register_explorer(
    registry: &mut Registry,
    store: &Arc<SqliteStore>,
    thresholds: MetricThresholds,
) -> Result<(), CodedError> {
    let ports = explorer_ports(store, thresholds);
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

fn usages_ports(store: &Arc<SqliteStore>, max_depth: u32) -> Arc<UsagesPorts> {
    Arc::new(UsagesPorts {
        projects: store.clone(),
        index: store.clone(),
        usages: store.clone(),
        text: Arc::new(FolderText),
        max_depth,
    })
}

fn graph_ports(store: &Arc<SqliteStore>, look: Graph) -> Arc<GraphPorts> {
    Arc::new(GraphPorts {
        projects: store.clone(),
        index: store.clone(),
        look,
    })
}

fn explorer_ports(store: &Arc<SqliteStore>, thresholds: MetricThresholds) -> Arc<ExplorerPorts> {
    Arc::new(ExplorerPorts {
        projects: store.clone(),
        index: store.clone(),
        outline: store.clone(),
        text: Arc::new(FolderText),
        thresholds,
    })
}

/// The HTML export: it reads through the commands of the explorer, the usages and the map, so its
/// pages show exactly what the browser UI shows.
pub(crate) fn export_command(
    store: &Arc<SqliteStore>,
    config: &Config,
    locations: &Locations,
) -> HtmlExport {
    let explorer = explorer_ports(store, config.metrics.clone());
    HtmlExport::new(
        export::ExportParts {
            projects: store.clone(),
            index: store.clone(),
            outline: ExplorerOutline {
                ports: Arc::clone(&explorer),
            },
            symbol: ExplorerSymbol { ports: explorer },
            usages: UsagesQuery {
                ports: usages_ports(store, config.mcp.max_depth),
            },
            graph: GraphLayers {
                ports: graph_ports(store, config.graph.clone()),
            },
        },
        locations.existing_dir(&config.ui.dir),
    )
}
