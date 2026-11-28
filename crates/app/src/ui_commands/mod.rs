//! The commands the browser UI calls over REST, registered once in the registry and exposed in
//! the server (blueprint 10.1). The scan runs on the thread of the call; the server runs every
//! command on its blocking pool, and progress reaches the UI through the event bus.

mod alerts;
mod annotation_wire;
mod annotations;
#[cfg(test)]
mod annotations_tests;
mod consts;
mod conventions;
#[cfg(test)]
mod conventions_tests;
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
mod graph_module_places;
mod graph_modules;
mod graph_types;
mod graph_wrap;
mod highlight;
mod indexers;
mod layers;
#[cfg(test)]
mod layers_tests;
mod level2;
mod level2_revoke;
mod log;
mod preferences;
mod project_file_open;
#[cfg(test)]
mod project_file_open_tests;
mod projects;
mod projects_pick;
#[cfg(test)]
mod projects_pick_tests;
mod projects_remove;
#[cfg(test)]
mod projects_remove_tests;
mod projects_rename;
#[cfg(test)]
mod projects_rename_tests;
#[cfg(test)]
mod projects_tests;
mod rating;
mod rename_preview;
mod rules;
mod rules_edit;
#[cfg(test)]
mod rules_edit_tests;
mod rules_symbols;
#[cfg(test)]
mod rules_tests;
mod scans;
#[cfg(test)]
mod scans_compare_tests;
#[cfg(test)]
mod scans_tests;
mod scope;
#[cfg(test)]
mod scope_tests;
mod search;
mod settings;
#[cfg(test)]
mod settings_tests;
mod source;
mod stats;
#[cfg(test)]
mod stats_tests;
mod status;
mod symbol;
mod temp_file;
#[cfg(test)]
pub(crate) mod tests;
mod ui_settings;
mod unused_keep;
#[cfg(test)]
mod unused_keep_tests;
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
use codeskimr_domain::{Author, LanguageId};
use codeskimr_i18n::Messages;
use codeskimr_parsing::{Blake3Digest, TreeSitterHighlighter};
use codeskimr_server::CommandApi;
use codeskimr_source::{FolderMentions, FolderText};

pub(crate) use status::{mark_ready, mark_started};

use crate::clock::SystemClock;
use crate::file_opener::SystemOpener;
use crate::folder_dialog::{FolderPicker, SystemPicker};
use crate::level2_runs::Level2Runs;
use crate::locations::Locations;
use crate::running::RunningScans;
use crate::scan::BrowserRun;
use crate::scan_compare;
use crate::ui_settings::UiSettingsFile;
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
use conventions::{ConventionsCheck, ConventionsPorts, Runs};
use credentials::{DeleteCredential, ListCredentials, SetCredential};
use explorer::{ExplorerOutline, ExplorerPorts};
pub(crate) use export::{ExportInput, HtmlExport};
use graph::{GraphLayers, GraphPorts};
use graph_module_places::GraphModulePlaces;
use graph_modules::GraphModules;
use highlight::CodeHighlight;
use indexers::ListIndexers;
use layers::{LayersGet, LayersSet};
use level2::Level2StatusCommand;
pub(crate) use level2::{Window, confirmation_desk};
use level2_revoke::Level2Revoke;
use log::{LogClear, LogOpenFolder, LogPorts, LogSetLevel, LogStatusCommand, LogTail};
use preferences::{GetPreferences, PreferencesFile, SetPreferences};
use project_file_open::ProjectFileOpen;
use projects::{AddProject, CancelScan, CheckTools, ListProjects, RescanProject};
use projects_pick::{FolderDialogInfo, PickFolder};
use projects_remove::RemoveProject;
use projects_rename::{RenameProject, SuggestName};
use rename_preview::{RenamePreviewCommand, RenamePreviewPorts};
pub(crate) use rules::hold;
use rules::{RulesHold, RulesShow};
use rules_edit::{RulesPreview, RulesSet};
use scans::{
    ComparePorts, MarkdownPorts, ScanListPorts, ScansCompare, ScansImpact, ScansImpactMarkdown,
    ScansList, ScansMetrics, ScansMetricsMarkdown,
};
pub(crate) use scope::{
    GetInput as ScopeGetInput, ScopeAnswer, ScopeGet, ScopeSet, SetInput as ScopeSetInput,
};
use search::{SearchPorts, SearchQuery};
use settings::EffectiveSettings;
use source::ExplorerSource;
use stats::{StatsOverview, StatsPorts, StatsRanking};
use status::{BudgetTargets, StatusInfo};
use symbol::ExplorerSymbol;
use ui_settings::{GetUiSettings, SetIndexerMemory, SetSettingValue};
use unused_keep::{UnusedKeep, UnusedLift};
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
    level2_runs: Level2Runs,
    registration: Mutex<()>,
    tiers: OnceLock<BTreeMap<String, &'static str>>,
}

impl Context {
    /// What a scan from the browser needs: the mark that the project is being scanned, the
    /// registry of level 2 runs and whether the caller wants level 2 (it does unless it said
    /// `false`).
    ///
    /// # Errors
    /// `PRJ-302` or `PRJ-501` as [`RunningScans::start`].
    fn browser_run(
        &self,
        id: codeskimr_domain::ProjectId,
        (level2, language): (Option<bool>, Option<LanguageId>),
    ) -> Result<BrowserRun, CodedError> {
        let guard = self.running.start(id)?;
        let running = self.running.clone();
        Ok(BrowserRun {
            level2: level2.unwrap_or(true),
            language,
            on_token: Box::new(move |token| running.attach(id, token)),
            guard,
            runs: self.level2_runs.clone(),
        })
    }

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
    let catalog = crate::wiring::open_store(locations, &config)?;
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
        level2_runs: Level2Runs::default(),
        registration: Mutex::new(()),
        tiers: OnceLock::new(),
    });
    let mut registry = Registry::new();
    register_projects(&mut registry, catalog, (&context, config))?;
    add(&mut registry, CheckTools { tools })?;
    add(&mut registry, ListIndexers { indexers })?;
    register_credentials(&mut registry, catalog, &context)?;
    register_annotations(
        &mut registry,
        &annotation_ports(catalog, config, &context.events)?,
    )?;
    register_views(&mut registry, catalog, config, (global, locations))?;
    let desk = confirmation_desk(catalog, config, (global, locations), window);
    register_level2(&mut registry, catalog, &desk)?;
    let mut api = CommandApi::new(Arc::new(registry));
    expose_all(&mut api)?;
    Ok((api, desk))
}

/// Registers the commands that read the index and the settings: search, usages, graph, status,
/// explorer, preferences and export.
fn register_views(
    registry: &mut Registry,
    catalog: &Arc<SqliteStore>,
    config: &Config,
    (global, locations): (&GlobalOptions, &Locations),
) -> Result<(), CodedError> {
    register_search(registry, catalog)?;
    register_usages(registry, catalog, config.mcp.max_depth)?;
    add(
        registry,
        RenamePreviewCommand {
            ports: rename_preview_ports(catalog, config),
        },
    )?;
    register_graph(registry, catalog, config.graph.clone())?;
    register_stats(registry, catalog, config)?;
    add(
        registry,
        ScansList {
            ports: Arc::new(ScanListPorts {
                projects: catalog.clone(),
                index: catalog.clone(),
                keep: config.snapshots.retention_count,
            }),
        },
    )?;
    register_compare(registry, catalog, (config, locations))?;
    add(registry, status_info(config))?;
    register_explorer(registry, catalog, config.metrics.clone(), locations)?;
    register_preferences(registry, locations)?;
    register_ui_settings(registry, (locations, global))?;
    register_log(registry, locations)?;
    add(
        registry,
        ConventionsCheck {
            ports: Arc::new(ConventionsPorts {
                projects: catalog.clone(),
                index: catalog.clone(),
                global: global.clone(),
                locations: locations.clone(),
                running: Runs::default(),
            }),
        },
    )?;
    add(registry, effective_settings(locations, global)?)?;
    add(registry, export_command(catalog, config, locations))
}

/// Registers the commands that compare two scans: the plain comparison, the impact and the impact
/// as Markdown.
fn register_compare(
    registry: &mut Registry,
    catalog: &Arc<SqliteStore>,
    (config, locations): (&Config, &Locations),
) -> Result<(), CodedError> {
    let ports = Arc::new(ComparePorts {
        projects: catalog.clone(),
        comparer: Arc::new(scan_compare::comparer(catalog, config)),
    });
    add(
        registry,
        ScansCompare {
            ports: Arc::clone(&ports),
        },
    )?;
    add(
        registry,
        ScansImpact {
            ports: Arc::clone(&ports),
        },
    )?;
    add(
        registry,
        ScansMetrics {
            ports: Arc::clone(&ports),
        },
    )?;
    let markdown = Arc::new(MarkdownPorts {
        compare: ports,
        messages: Messages::load().map_err(CodedError::caused_by)?,
        preferences: PreferencesFile::in_dir(&locations.data_dir),
    });
    add(
        registry,
        ScansImpactMarkdown {
            ports: Arc::clone(&markdown),
        },
    )?;
    add(registry, ScansMetricsMarkdown { ports: markdown })
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

/// The command that shows the settings in effect: read once from the same files and flags that the
/// configuration of this run came from.
fn effective_settings(
    locations: &Locations,
    global: &GlobalOptions,
) -> Result<EffectiveSettings, CodedError> {
    let layers = crate::config_files::layers(locations, global, None)?;
    EffectiveSettings::new(&layers, locations, global)
}

/// The two commands that keep theme and language in the data folder.
fn register_preferences(registry: &mut Registry, locations: &Locations) -> Result<(), CodedError> {
    let file = PreferencesFile::in_dir(&locations.data_dir);
    add(registry, GetPreferences { file: file.clone() })?;
    add(registry, SetPreferences { file })
}

/// The commands that keep the settings the UI saves in the data folder (ADR 0103, ADR 0117).
fn register_ui_settings(
    registry: &mut Registry,
    (locations, global): (&Locations, &GlobalOptions),
) -> Result<(), CodedError> {
    let file = UiSettingsFile::in_dir(&locations.data_dir);
    add(registry, GetUiSettings { file: file.clone() })?;
    add(registry, SetIndexerMemory { file: file.clone() })?;
    add(
        registry,
        SetSettingValue {
            file,
            locations: locations.clone(),
            global: global.clone(),
        },
    )
}

/// The commands of the tab Diagnosis: the log level and the log file in the data folder (ADR 0115).
fn register_log(registry: &mut Registry, locations: &Locations) -> Result<(), CodedError> {
    crate::log_control::attach_file(&locations.data_dir);
    let ports = Arc::new(LogPorts {
        opener: Arc::new(SystemOpener),
    });
    add(
        registry,
        LogStatusCommand {
            ports: ports.clone(),
        },
    )?;
    add(
        registry,
        LogSetLevel {
            ports: ports.clone(),
        },
    )?;
    add(registry, LogTail)?;
    add(
        registry,
        LogClear {
            ports: ports.clone(),
        },
    )?;
    add(registry, LogOpenFolder { ports })
}

/// Registers one command, with the registry's refusal as a coded failure.
fn add<C: Command>(registry: &mut Registry, command: C) -> Result<(), CodedError> {
    registry.register(command).map_err(CodedError::caused_by)
}

/// The commands that read or write the project file: layers, rules and the exception list.
fn expose_project_file(api: &mut CommandApi) -> Result<(), CodedError> {
    api.expose::<LayersGet>()?;
    api.expose::<LayersSet>()?;
    api.expose::<RulesShow>()?;
    api.expose::<RulesHold>()?;
    api.expose::<RulesSet>()?;
    api.expose::<ConventionsCheck>()?;
    api.expose::<RulesPreview>()?;
    api.expose::<ProjectFileOpen>()?;
    api.expose::<UnusedKeep>()?;
    api.expose::<UnusedLift>()?;
    Ok(())
}

/// The commands of the log.
fn expose_log(api: &mut CommandApi) -> Result<(), CodedError> {
    api.expose::<LogStatusCommand>()?;
    api.expose::<LogSetLevel>()?;
    api.expose::<LogTail>()?;
    api.expose::<LogClear>()?;
    api.expose::<LogOpenFolder>()?;
    Ok(())
}

/// Makes the commands of the saved preferences and settings reachable over REST.
fn expose_settings(api: &mut CommandApi) -> Result<(), CodedError> {
    api.expose::<GetPreferences>()?;
    api.expose::<SetPreferences>()?;
    api.expose::<GetUiSettings>()?;
    api.expose::<SetIndexerMemory>()?;
    api.expose::<SetSettingValue>()?;
    api.expose::<EffectiveSettings>()?;
    Ok(())
}

/// Makes every registered command of the UI reachable over REST.
fn expose_all(api: &mut CommandApi) -> Result<(), CodedError> {
    api.expose::<ListProjects>()?;
    api.expose::<AddProject>()?;
    api.expose::<RescanProject>()?;
    api.expose::<RemoveProject>()?;
    api.expose::<RenameProject>()?;
    api.expose::<SuggestName>()?;
    api.expose::<ScopeGet>()?;
    api.expose::<ScopeSet>()?;
    api.expose::<FolderDialogInfo>()?;
    api.expose::<PickFolder>()?;
    api.expose::<CheckTools>()?;
    api.expose::<ListIndexers>()?;
    api.expose::<Level2StatusCommand>()?;
    api.expose::<Level2Revoke>()?;
    api.expose::<CancelScan>()?;
    api.expose::<ListCredentials>()?;
    api.expose::<SetCredential>()?;
    api.expose::<DeleteCredential>()?;
    expose_index_views(api)?;
    api.expose::<GraphLayers>()?;
    api.expose::<GraphModules>()?;
    api.expose::<GraphModulePlaces>()?;
    expose_project_file(api)?;
    api.expose::<StatusInfo>()?;
    api.expose::<StatsOverview>()?;
    api.expose::<StatsRanking>()?;
    api.expose::<ScansList>()?;
    api.expose::<ScansCompare>()?;
    api.expose::<ScansMetrics>()?;
    api.expose::<ScansImpact>()?;
    api.expose::<ScansImpactMarkdown>()?;
    api.expose::<ScansMetricsMarkdown>()?;
    expose_settings(api)?;
    expose_log(api)?;
    api.expose::<HtmlExport>()?;
    expose_annotations(api)
}

/// Makes the commands that read the index reachable over REST: explorer, search, usages and the
/// check of a rename.
fn expose_index_views(api: &mut CommandApi) -> Result<(), CodedError> {
    api.expose::<ExplorerOutline>()?;
    api.expose::<ExplorerSymbol>()?;
    api.expose::<ExplorerSource>()?;
    api.expose::<CodeHighlight>()?;
    api.expose::<SearchQuery>()?;
    api.expose::<UsagesQuery>()?;
    api.expose::<UsagesCode>()?;
    api.expose::<RenamePreviewCommand>()?;
    Ok(())
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

// codeskimr: allow CHK-102 lists the commands of one area, one entry per command
fn register_projects(
    registry: &mut Registry,
    catalog: &Arc<SqliteStore>,
    (context, config): (&Arc<Context>, &Config),
) -> Result<(), CodedError> {
    add(
        registry,
        ListProjects {
            catalog: catalog.clone(),
            tools: Arc::clone(&context.tools),
            reports: catalog.clone(),
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
        RenameProject {
            store: catalog.clone(),
        },
    )?;
    add(registry, SuggestName)?;
    register_scope(registry, catalog, (config, &context.locations))?;
    register_folder_dialog(registry)?;
    register_scans(registry, catalog, context)
}

/// The two commands of the scope of a project (ADR 0102).
pub(crate) fn register_scope(
    registry: &mut Registry,
    catalog: &Arc<SqliteStore>,
    (config, locations): (&Config, &Locations),
) -> Result<(), CodedError> {
    add(
        registry,
        ScopeGet {
            store: catalog.clone(),
            config: config.clone(),
            locations: locations.clone(),
        },
    )?;
    add(
        registry,
        ScopeSet {
            store: catalog.clone(),
            max_entries: config.scan.max_scope_entries,
        },
    )
}

// codeskimr: allow CHK-102 lists the commands of one area, one entry per command
fn register_scans(
    registry: &mut Registry,
    catalog: &Arc<SqliteStore>,
    context: &Arc<Context>,
) -> Result<(), CodedError> {
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
        CancelScan {
            context: Arc::clone(context),
        },
    )
}

/// The two commands of the "Browse" button, on the dialog of the operating system.
fn register_folder_dialog(registry: &mut Registry) -> Result<(), CodedError> {
    let picker: Arc<dyn FolderPicker> = Arc::new(SystemPicker::default());
    add(
        registry,
        FolderDialogInfo {
            picker: Arc::clone(&picker),
        },
    )?;
    add(registry, PickFolder { picker })
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

/// The command that opens the project file with the program of the system (ADR 0111).
fn register_project_file_open(
    registry: &mut Registry,
    ports: &Arc<ExplorerPorts>,
) -> Result<(), CodedError> {
    add(
        registry,
        ProjectFileOpen {
            ports: Arc::clone(ports),
            opener: Arc::new(SystemOpener),
        },
    )
}

/// The commands that change the rules of the project file (ADR 0112).
fn register_rules_edit(
    registry: &mut Registry,
    ports: &Arc<ExplorerPorts>,
) -> Result<(), CodedError> {
    add(
        registry,
        RulesSet {
            ports: Arc::clone(ports),
        },
    )?;
    add(
        registry,
        RulesPreview {
            ports: Arc::clone(ports),
        },
    )
}

/// The commands of the project file, see [`expose_project_file`].
fn register_project_file(
    registry: &mut Registry,
    ports: &Arc<ExplorerPorts>,
) -> Result<(), CodedError> {
    add(
        registry,
        LayersGet {
            ports: Arc::clone(ports),
        },
    )?;
    add(
        registry,
        LayersSet {
            ports: Arc::clone(ports),
        },
    )?;
    add(
        registry,
        RulesShow {
            ports: Arc::clone(ports),
        },
    )?;
    add(
        registry,
        RulesHold {
            ports: Arc::clone(ports),
        },
    )?;
    register_project_file_open(registry, ports)?;
    add(
        registry,
        UnusedKeep {
            ports: Arc::clone(ports),
        },
    )?;
    add(
        registry,
        UnusedLift {
            ports: Arc::clone(ports),
        },
    )?;
    Ok(())
}

fn register_graph(
    registry: &mut Registry,
    store: &Arc<SqliteStore>,
    look: Graph,
) -> Result<(), CodedError> {
    let ports = graph_ports(store, look);
    add(
        registry,
        GraphLayers {
            ports: Arc::clone(&ports),
        },
    )?;
    add(
        registry,
        GraphModules {
            ports: Arc::clone(&ports),
        },
    )?;
    add(registry, GraphModulePlaces { ports })
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
    register_project_file(registry, &ports)?;
    register_rules_edit(registry, &ports)?;
    add(
        registry,
        ExplorerSource {
            ports: Arc::clone(&ports),
        },
    )?;
    add(
        registry,
        CodeHighlight {
            ports: Arc::clone(&ports),
        },
    )?;
    add(registry, ExplorerSymbol { ports })
}

fn register_stats(
    registry: &mut Registry,
    store: &Arc<SqliteStore>,
    config: &Config,
) -> Result<(), CodedError> {
    let ports = Arc::new(StatsPorts {
        projects: store.clone(),
        index: store.clone(),
        statistics: store.clone(),
        thresholds: config.metrics.clone(),
        test_names: config.conventions.test_folders.clone(),
        approximate: config.ui.approximate_languages.clone(),
    });
    add(
        registry,
        StatsOverview {
            ports: Arc::clone(&ports),
        },
    )?;
    add(registry, StatsRanking { ports })
}

fn usages_ports(store: &Arc<SqliteStore>, max_depth: u32) -> Arc<UsagesPorts> {
    Arc::new(UsagesPorts {
        projects: store.clone(),
        index: store.clone(),
        usages: store.clone(),
        text: Arc::new(FolderText),
        reports: store.clone(),
        max_depth,
    })
}

fn rename_preview_ports(store: &Arc<SqliteStore>, config: &Config) -> Arc<RenamePreviewPorts> {
    Arc::new(RenamePreviewPorts {
        projects: store.clone(),
        index: store.clone(),
        usages: store.clone(),
        mentions: Arc::new(FolderMentions::new(config.scan.excluded_folders.clone())),
        source: Arc::new(FolderText),
    })
}

fn graph_ports(store: &Arc<SqliteStore>, look: Graph) -> Arc<GraphPorts> {
    Arc::new(GraphPorts {
        projects: store.clone(),
        index: store.clone(),
        references: store.clone(),
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
        references: store.clone(),
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
            max_references: config.export.max_references,
        },
        locations.existing_dir(&config.ui.dir),
        crate::embedded_ui::FILES,
    )
}
