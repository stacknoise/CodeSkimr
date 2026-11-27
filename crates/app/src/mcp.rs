//! `codeskimr mcp`: serves the MCP tools for one scanned project over standard input and output.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use codeskimr_application::config::Config;
use codeskimr_application::conventions::ConventionChecker;
use codeskimr_application::error::CodedError;
use codeskimr_application::events::EventBus;
use codeskimr_application::level2::LimitSource;
use codeskimr_application::ports::{CodeIndex, IndexRunner, ProjectScanner};
use codeskimr_cli::{GlobalOptions, McpArgs};
use codeskimr_codes::ErrorCode;
use codeskimr_domain::ProjectId;
use codeskimr_i18n::Messages;
use codeskimr_mcp::{HttpService, McpServer, Ports, Tools, serve_stdio};
use codeskimr_source::{FolderKeys, FolderMentions, FolderText};
use codeskimr_storage::SqliteStore;

use crate::clock::SystemClock;
use crate::config_files;
use crate::locations::Locations;
use crate::principal;
use crate::rules_check::StoredRules;
use crate::running::{ExclusiveScans, RunningScans};
use crate::scan_compare;
use crate::source_arg::folder_spec;
use crate::unused_symbols::JudgedUnused;
use crate::wiring::{
    Analysis, Secrets, build_analysis, index_runner, open_store, os_secrets, scanner_over,
    tool_checker,
};

/// Serves MCP until the client disconnects.
///
/// The project is the folder given with `--project`, or the current folder. It must have been
/// scanned with `codeskimr scan` before. Nothing but the protocol is written to standard output.
///
/// # Errors
/// - `PRJ-201` if the folder is not a registered project, `MCP-203` if it has no snapshot yet.
/// - `APP-901` if the runtime or the protocol fails.
/// - Any coded failure of the configuration, the storage or the message texts.
pub fn run(
    args: &McpArgs,
    global: &GlobalOptions,
    locations: &Locations,
    current_dir: &Path,
) -> Result<(), CodedError> {
    let folder: PathBuf = args
        .project
        .clone()
        .unwrap_or_else(|| current_dir.to_path_buf());
    let spec = folder_spec(&folder)?;
    let config = config_files::load(locations, global, Some(&folder))?;
    // Nothing was ever scanned if there is no database; opening the store would create it.
    if !locations.database().exists() {
        return Err(CodedError::new(ErrorCode::ProjectNotFound));
    }
    let store = open_store(locations, &config)?;
    let project = store
        .project_by_source(&spec.source)?
        .ok_or_else(|| CodedError::new(ErrorCode::ProjectNotFound))?;
    if store.latest_snapshot(project.id)?.is_none() {
        return Err(CodedError::new(ErrorCode::ProjectNotScanned));
    }
    let messages = Arc::new(Messages::load().map_err(CodedError::caused_by)?);
    let tools = project_tools(store, &config, (locations, &messages), project.id)?;
    let server = McpServer::new(tools, &messages, principal::remote_client())?;
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| CodedError::new(ErrorCode::UnexpectedInternalError))?
        .block_on(serve_stdio(server))
}

/// The MCP tools for one project over `store`, as `codeskimr mcp` serves them. The commands that
/// ask what agents ask (`symbols`, `usages`, `impact`, `context`) use them too, so the answers
/// are the same.
///
/// # Errors
/// Any coded failure of loading the language packs or wiring the ports.
pub(crate) fn project_tools(
    store: Arc<SqliteStore>,
    config: &Config,
    (locations, messages): (&Locations, &Arc<Messages>),
    project: ProjectId,
) -> Result<Tools, CodedError> {
    let wiring = Wiring {
        locations,
        messages: Arc::clone(messages),
        runner: index_runner(locations),
        running: RunningScans::default(),
        limits: None,
    };
    let ports = build_ports(store, config, wiring, (EventBus::new(), &os_secrets()))?;
    Ok(Tools::new(
        ports,
        (config.mcp.clone(), config.graph.risk_limits()),
        project,
    ))
}

/// The MCP tools over streamable HTTP for the running app (blueprint 10.4): every registered
/// project, no default one, the same store file as the UI and the same event bus, so a `rescan`
/// of an agent shows up in the UI like a scan from the UI. The caller is the local owner; the web
/// server lets only requests with the installation token reach the service.
///
/// # Errors
/// Any coded failure of opening the store, loading the language packs or the message texts.
#[cfg(test)]
pub fn http_service(
    config: &Config,
    locations: &Locations,
    (events, secrets): (EventBus, &Arc<Secrets>),
) -> Result<HttpService, CodedError> {
    http_service_probing(
        config,
        locations,
        (events, secrets),
        (None, RunningScans::default()),
    )
}

/// Like [`http_service`], but over the `store` the UI uses too, and a `rescan` and the scans of
/// the UI share `running`, so a second scan of the same project is refused with `PRJ-302`
/// whichever side started the first. One store means one usages index: a level 2 import through
/// either side is seen by the other (#358).
///
/// # Errors
/// The same as [`http_service`], without the opening of the store.
pub fn http_service_sharing(
    store: Arc<SqliteStore>,
    config: &Config,
    locations: &Locations,
    (events, secrets): (EventBus, &Arc<Secrets>),
    (running, limits): (RunningScans, Arc<dyn LimitSource>),
) -> Result<HttpService, CodedError> {
    http_service_over(
        store,
        config,
        locations,
        (events, secrets),
        (None, running, Some(limits)),
    )
}

/// Like [`http_service`], but the tools are started through `runner` instead of as real processes
/// (tests use this to say which tools exist).
///
/// # Errors
/// The same as [`http_service`].
#[cfg(test)]
pub fn http_service_probing(
    config: &Config,
    locations: &Locations,
    (events, secrets): (EventBus, &Arc<Secrets>),
    (runner, running): (Option<Arc<dyn IndexRunner>>, RunningScans),
) -> Result<HttpService, CodedError> {
    let store = crate::wiring::open_store(locations, config)?;
    http_service_over(
        store,
        config,
        locations,
        (events, secrets),
        (runner, running, None),
    )
}

fn http_service_over(
    store: Arc<SqliteStore>,
    config: &Config,
    locations: &Locations,
    (events, secrets): (EventBus, &Arc<Secrets>),
    (runner, running, limits): RunParts,
) -> Result<HttpService, CodedError> {
    let messages = Arc::new(Messages::load().map_err(CodedError::caused_by)?);
    let wiring = Wiring {
        locations,
        messages: Arc::clone(&messages),
        runner: runner.unwrap_or_else(|| index_runner(locations)),
        running,
        limits,
    };
    let ports = build_ports(store, config, wiring, (events, secrets))?;
    let tools = Tools::unscoped(ports, (config.mcp.clone(), config.graph.risk_limits()));
    let server = McpServer::new(tools, &messages, principal::remote_client())?;
    Ok(codeskimr_mcp::http_service(&server))
}

/// How the scans of the tools run: the runner of the indexers (`None`: real processes), the mark of
/// the running scans and where the limits of an indexer come from.
type RunParts = (
    Option<Arc<dyn IndexRunner>>,
    RunningScans,
    Option<Arc<dyn LimitSource>>,
);

/// What the ports of the tools are built with besides the store and the configuration.
struct Wiring<'a> {
    locations: &'a Locations,
    messages: Arc<Messages>,
    runner: Arc<dyn IndexRunner>,
    running: RunningScans,
    limits: Option<Arc<dyn LimitSource>>,
}

/// The convention check over the files and language packs of `analysis`.
fn convention_checker(analysis: &Analysis, config: &Config) -> Arc<ConventionChecker> {
    Arc::new(ConventionChecker::new(
        Arc::clone(&analysis.discovery),
        Arc::clone(&analysis.analyzer),
        Arc::new(FolderText),
        config.conventions.clone(),
    ))
}

/// The ports of the tools: the store for everything it implements, the analysis ports the
/// convention check and the snippet fingerprints read files and language packs through, and the
/// scanner for `rescan`.
fn build_ports(
    store: Arc<SqliteStore>,
    config: &Config,
    Wiring {
        locations,
        messages,
        runner,
        running,
        limits,
    }: Wiring<'_>,
    (events, secrets): (EventBus, &Arc<Secrets>),
) -> Result<Ports, CodedError> {
    let analysis = build_analysis(config, locations)?;
    let checker = convention_checker(&analysis, config);
    let scanner: Arc<dyn ProjectScanner> = Arc::new(ExclusiveScans::new(
        scanner_over(
            &analysis,
            (&store, config, locations),
            (events, secrets),
            limits,
        ),
        running,
    ));
    let snippets = analysis.snippets;
    let tool_check = tool_checker(&store, (locations, runner))?;
    Ok(Ports {
        projects: store.clone(),
        usages: store.clone(),
        similarity: store.clone(),
        references: store.clone(),
        places: store.clone(),
        snippets,
        checker,
        clock: Arc::new(SystemClock),
        scanner,
        catalog: store.clone(),
        tool_check,
        messages,
        keys: Arc::new(FolderKeys),
        key_sources: config.keys.clone(),
        annotations: store.clone(),
        reports: store.clone(),
        unused: Arc::new(JudgedUnused::new(store.clone(), config)),
        rules: Arc::new(StoredRules::new(store.clone())),
        compare: Arc::new(scan_compare::comparer(&store, config)),
        mentions: Arc::new(FolderMentions::new(config.scan.excluded_folders.clone())),
        text: Arc::new(FolderText),
        index: store,
    })
}
