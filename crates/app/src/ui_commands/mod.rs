//! The commands the browser UI calls over REST, registered once in the registry and exposed in
//! the server (blueprint 10.1). The scan runs on the thread of the call; the server runs every
//! command on its blocking pool, and progress reaches the UI through the event bus.

mod alerts;
mod annotation_wire;
mod annotations;
#[cfg(test)]
mod annotations_tests;
mod consts;
mod credentials;
#[cfg(test)]
mod credentials_tests;
mod explorer;
#[cfg(test)]
mod explorer_tests;
mod export;
#[cfg(test)]
mod export_tests;
mod graph;
mod graph_focus;
mod graph_types;
mod graph_wrap;
mod indexers;
mod layers;
#[cfg(test)]
mod layers_tests;
mod level2;
mod level2_revoke;
mod preferences;
mod projects;
mod projects_remove;
#[cfg(test)]
mod projects_remove_tests;
#[cfg(test)]
mod projects_tests;
mod rating;
mod search;
mod source;
mod status;
mod symbol;
mod temp_file;
#[cfg(test)]
pub(crate) mod tests;
mod usages;
mod usages_code;

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock, PoisonError};

use codeskimr_application::annotations::AnnotationWrites;
use codeskimr_application::config::{Config, Graph, MetricThresholds};
use codeskimr_application::error::CodedError;
use codeskimr_application::events::EventBus;
use codeskimr_application::level2::ConfirmationDesk;
use codeskimr_application::ports::{IndexRunner, ProjectToolCheck};
use codeskimr_application::registry::{Command, Registry};
use codeskimr_cli::GlobalOptions;
use codeskimr_domain::Author;
use codeskimr_parsing::{Blake3Digest, TreeSitterHighlighter};
use codeskimr_server::CommandApi;
use codeskimr_source::FolderText;

pub(crate) use status::{mark_ready, mark_started};

use crate::clock::SystemClock;
use crate::locations::Locations;
use crate::running::RunningScans;
use crate::wiring::{Secrets, index_runner, indexer_check, tool_checker};
pub(crate) use alerts::{
    AlertListInput, AlertListOut, AlertsDismiss, AlertsDismissAll, AlertsList, DismissAllInput,
    DismissInput, Dismissed,
};
pub(crate) use annotation_wire::{AlertOut, AnnotationOut, Nullable, PageOut, TargetIn};
pub(crate) use annotations::{
    AnnotationPorts, AnnotationsDelete, AnnotationsList, AnnotationsUpdate, AnyKind, Create,
    CreateInput, DeleteInput, ListInput, UpdateInput,
};
use annotations::{AnnotationsSummary, CriticalMark, NoteKind, TaskKind};
use codeskimr_storage::SqliteStore;
use consts::LOCAL_AUTHOR;
#[cfg(test)]
pub(crate) use consts::PREFERENCES_FILE;
use credentials::{DeleteCredential, ListCredentials, SetCredential};
use explorer::{ExplorerOutline, ExplorerPorts};
pub(crate) use export::{ExportInput, HtmlExport};
use graph::{GraphLayers, GraphPorts};
use indexers::ListIndexers;
use layers::{LayersGet, LayersSet};
use level2::Level2StatusCommand;
pub(crate) use level2::{Window, confirmation_desk};
use level2_revoke::Level2Revoke;
use preferences::{GetPreferences, PreferencesFile, SetPreferences};
use projects::{AddProject, CancelScan, CheckTools, ListProjects, RescanProject};
use projects_remove::RemoveProject;
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
    tools: Arc<dyn ProjectToolCheck>,
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
#[cfg(test)]
pub fn command_api(
    global: &GlobalOptions,
    locations: &Locations,
    events: EventBus,
    secrets: Arc<Secrets>,
) -> Result<CommandApi, CodedError> {
    command_api_probing(
        global,
        locations,
        (events, secrets),
        (None, RunningScans::default()),
    )
}

/// A window that never opens anything and says the system has none: what the tests use in place
/// of the real adapter, so that no test can open a window.
#[cfg(test)]
pub(crate) struct NoWindow;

#[cfg(test)]
impl codeskimr_application::ports::ConfirmationWindow for NoWindow {
    fn open(
        &self,
        _project: codeskimr_domain::ProjectId,
    ) -> Result<codeskimr_application::ports::WindowOutcome, CodedError> {
        Ok(codeskimr_application::ports::WindowOutcome::Unavailable)
    }
}

/// Like [`command_api`], but the tools are started through `runner` instead of as real processes
/// (tests use this to say which tools exist).
///
/// # Errors
/// The same as [`command_api`].
#[cfg(test)]
pub fn command_api_probing(
    global: &GlobalOptions,
    locations: &Locations,
    (events, secrets): (EventBus, Arc<Secrets>),
    run: (Option<Arc<dyn IndexRunner>>, RunningScans),
) -> Result<CommandApi, CodedError> {
    let config = crate::config_files::load(locations, global, None)?;
    let catalog = crate::wiring::open_serving_store(locations, &config)?;
    let window = Window {
        port: Arc::new(NoWindow),
        launch: codeskimr_application::level2::Launch::Copy,
    };
    command_api_over(
        &catalog,
        &config,
        (global, locations),
        (events, secrets),
        (run, window),
    )
    .map(|(api, _desk)| api)
}

/// Like [`command_api_probing`], but over a `catalog` that the caller opened, so that the MCP
/// service can use the same store (#358). Besides the commands it returns the desk of the level 2
/// confirmation, which the server needs for the route that opens the window; `window` is the
/// adapter of that window and what its system can do.
///
/// # Errors
/// The same as [`command_api`], without the opening of the database.
pub fn command_api_over(
    catalog: &Arc<SqliteStore>,
    config: &Config,
    (global, locations): (&GlobalOptions, &Locations),
    (events, secrets): (EventBus, Arc<Secrets>),
    ((runner, running), window): ((Option<Arc<dyn IndexRunner>>, RunningScans), Window),
) -> Result<(CommandApi, Arc<ConfirmationDesk>), CodedError> {
    let runner = runner.unwrap_or_else(|| index_runner(locations));
    let indexers = indexer_check(locations, runner.clone())?;
    let tools = tool_checker(catalog, (locations, runner))?;
    let context = Arc::new(Context {
        global: global.clone(),
        locations: locations.clone(),
        events,
        secrets,
        tools: tools.clone(),
        running,
        registration: Mutex::new(()),
        tiers: OnceLock::new(),
    });
    let mut registry = Registry::new();
    register_projects(&mut registry, catalog, &context)?;
    add(&mut registry, CheckTools { tools })?;
    add(&mut registry, ListIndexers { indexers })?;
    register_credentials(&mut registry, catalog, &context)?;
    register_search(&mut registry, catalog)?;
    register_usages(&mut registry, catalog, config.mcp.max_depth)?;
    register_graph(&mut registry, catalog, config.graph.clone())?;
    register_annotations(
        &mut registry,
        &annotation_ports(catalog, config, &context.events)?,
    )?;
    add(&mut registry, status_info(config))?;
    register_explorer(&mut registry, catalog, config.metrics.clone(), locations)?;
    register_preferences(&mut registry, locations)?;
    add(&mut registry, export_command(catalog, config, locations))?;
    let desk = confirmation_desk(catalog, config, (global, locations), window);
    register_level2(&mut registry, catalog, &desk)?;
    let mut api = CommandApi::new(Arc::new(registry));
    expose_all(&mut api)?;
    Ok((api, desk))
}

/// Registers the two commands of the level 2 switch (ADR 0071). Neither is an MCP tool.
fn register_level2(
    registry: &mut Registry,
    catalog: &Arc<SqliteStore>,
    desk: &Arc<ConfirmationDesk>,
) -> Result<(), CodedError> {
    add(
        registry,
        Level2StatusCommand {
            desk: Arc::clone(desk),
        },
    )?;
    add(
        registry,
        Level2Revoke {
            projects: catalog.clone(),
            store: catalog.clone(),
            desk: Arc::clone(desk),
        },
    )
}

/// What the commands of annotations and alerts work through: the store, the checks of the texts
/// against `config`, and the bus on which `AlertsChanged` goes out.
pub(crate) fn annotation_ports(
    store: &Arc<SqliteStore>,
    config: &Config,
    events: &EventBus,
) -> Result<Arc<AnnotationPorts>, CodedError> {
    let author = Author::new(LOCAL_AUTHOR).map_err(CodedError::caused_by)?;
    Ok(Arc::new(AnnotationPorts {
        store: store.clone(),
        projects: store.clone(),
        writes: AnnotationWrites {
            store: store.clone(),
            index: store.clone(),
            states: store.clone(),
            digest: Arc::new(Blake3Digest),
            clock: Arc::new(SystemClock),
            events: events.clone(),
            config: config.annotations.clone(),
            author,
        },
        config: config.annotations.clone(),
    }))
}

/// Registers the commands of annotations and alerts.
pub(crate) fn register_annotations(
    registry: &mut Registry,
    ports: &Arc<AnnotationPorts>,
) -> Result<(), CodedError> {
    let shared = || Arc::clone(ports);
    add(registry, AnnotationsList { ports: shared() })?;
    add(registry, AnnotationsSummary { ports: shared() })?;
    add(registry, Create::<AnyKind>::new(shared()))?;
    add(registry, Create::<CriticalMark>::new(shared()))?;
    add(registry, Create::<NoteKind>::new(shared()))?;
    add(registry, Create::<TaskKind>::new(shared()))?;
    add(registry, AnnotationsUpdate { ports: shared() })?;
    add(registry, AnnotationsDelete { ports: shared() })?;
    add(registry, AlertsList { ports: shared() })?;
    add(registry, AlertsDismiss { ports: shared() })?;
    add(registry, AlertsDismissAll { ports: shared() })
}

/// The command behind the status bar, with the budgets and the worker limit of `config`.
fn status_info(config: &Config) -> StatusInfo {
    StatusInfo {
        budgets: BudgetTargets::from(&config.budgets),
        max_workers: config.scan.max_workers,
    }
}

/// The two commands that keep theme and language in the data folder.
fn register_preferences(registry: &mut Registry, locations: &Locations) -> Result<(), CodedError> {
    let file = PreferencesFile::in_dir(&locations.data_dir);
    add(registry, GetPreferences { file: file.clone() })?;
    add(registry, SetPreferences { file })
}

/// Registers one command, with the registry's refusal as a coded failure.
fn add<C: Command>(registry: &mut Registry, command: C) -> Result<(), CodedError> {
    registry.register(command).map_err(CodedError::caused_by)
}

/// Makes every registered command of the UI reachable over REST.
fn expose_all(api: &mut CommandApi) -> Result<(), CodedError> {
    api.expose::<ListProjects>()?;
    api.expose::<AddProject>()?;
    api.expose::<RescanProject>()?;
    api.expose::<RemoveProject>()?;
    api.expose::<CheckTools>()?;
    api.expose::<ListIndexers>()?;
    api.expose::<Level2StatusCommand>()?;
    api.expose::<Level2Revoke>()?;
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
    api.expose::<LayersGet>()?;
    api.expose::<LayersSet>()?;
    api.expose::<StatusInfo>()?;
    api.expose::<GetPreferences>()?;
    api.expose::<SetPreferences>()?;
    api.expose::<HtmlExport>()?;
    expose_annotations(api)
}

/// Makes the commands of annotations and alerts reachable over REST.
fn expose_annotations(api: &mut CommandApi) -> Result<(), CodedError> {
    api.expose::<AnnotationsList>()?;
    api.expose::<AnnotationsSummary>()?;
    api.expose::<Create<AnyKind>>()?;
    api.expose::<Create<CriticalMark>>()?;
    api.expose::<Create<NoteKind>>()?;
    api.expose::<Create<TaskKind>>()?;
    api.expose::<AnnotationsUpdate>()?;
    api.expose::<AnnotationsDelete>()?;
    api.expose::<AlertsList>()?;
    api.expose::<AlertsDismiss>()?;
    api.expose::<AlertsDismissAll>()?;
    Ok(())
}

fn register_projects(
    registry: &mut Registry,
    catalog: &Arc<SqliteStore>,
    context: &Arc<Context>,
) -> Result<(), CodedError> {
    add(
        registry,
        ListProjects {
            catalog: catalog.clone(),
            tools: Arc::clone(&context.tools),
            context: Arc::clone(context),
        },
    )?;
    add(
        registry,
        AddProject {
            context: Arc::clone(context),
        },
    )?;
    add(
        registry,
        RescanProject {
            catalog: catalog.clone(),
            context: Arc::clone(context),
        },
    )?;
    add(
        registry,
        RemoveProject {
            projects: catalog.clone(),
            removal: catalog.clone(),
            context: Arc::clone(context),
        },
    )?;
    add(
        registry,
        CancelScan {
            context: Arc::clone(context),
        },
    )
}

fn register_credentials(
    registry: &mut Registry,
    catalog: &Arc<SqliteStore>,
    context: &Context,
) -> Result<(), CodedError> {
    let secrets = &context.secrets;
    let list = ListCredentials {
        catalog: catalog.clone(),
        secrets: Arc::clone(secrets),
    };
    add(registry, list)?;
    add(
        registry,
        SetCredential {
            secrets: Arc::clone(secrets),
        },
    )?;
    let secrets = Arc::clone(secrets);
    add(registry, DeleteCredential { secrets })
}

fn register_search(registry: &mut Registry, store: &Arc<SqliteStore>) -> Result<(), CodedError> {
    let ports = Arc::new(SearchPorts {
        projects: store.clone(),
        index: store.clone(),
        catalog: store.clone(),
    });
    add(registry, SearchQuery { ports })
}

fn register_usages(
    registry: &mut Registry,
    store: &Arc<SqliteStore>,
    max_depth: u32,
) -> Result<(), CodedError> {
    let ports = usages_ports(store, max_depth);
    add(
        registry,
        UsagesQuery {
            ports: Arc::clone(&ports),
        },
    )?;
    add(registry, UsagesCode { ports })
}

fn register_graph(
    registry: &mut Registry,
    store: &Arc<SqliteStore>,
    look: Graph,
) -> Result<(), CodedError> {
    add(
        registry,
        GraphLayers {
            ports: graph_ports(store, look),
        },
    )
}

fn register_explorer(
    registry: &mut Registry,
    store: &Arc<SqliteStore>,
    thresholds: MetricThresholds,
    locations: &Locations,
) -> Result<(), CodedError> {
    let ports = explorer_ports(store, thresholds, locations);
    add(
        registry,
        ExplorerOutline {
            ports: Arc::clone(&ports),
        },
    )?;
    add(
        registry,
        LayersGet {
            ports: Arc::clone(&ports),
        },
    )?;
    add(
        registry,
        LayersSet {
            ports: Arc::clone(&ports),
        },
    )?;
    add(
        registry,
        ExplorerSource {
            ports: Arc::clone(&ports),
        },
    )?;
    add(registry, ExplorerSymbol { ports })
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

fn explorer_ports(
    store: &Arc<SqliteStore>,
    thresholds: MetricThresholds,
    locations: &Locations,
) -> Arc<ExplorerPorts> {
    Arc::new(ExplorerPorts {
        projects: store.clone(),
        index: store.clone(),
        outline: store.clone(),
        text: Arc::new(FolderText),
        highlighter: Arc::new(TreeSitterHighlighter::new(&locations.languages_dir)),
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
    let explorer = explorer_ports(store, config.metrics.clone(), locations);
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
        crate::embedded_ui::FILES,
    )
}
