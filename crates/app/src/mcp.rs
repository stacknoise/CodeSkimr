//! `codeskimr mcp`: serves the MCP tools for one scanned project over standard input and output.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use codeskimr_application::config::Config;
use codeskimr_application::conventions::ConventionChecker;
use codeskimr_application::error::CodedError;
use codeskimr_application::events::EventBus;
use codeskimr_application::ports::{
    CodeIndex, IndexRunner, ProjectCatalog, ProjectRepository, ProjectScanner, SimilarityIndex,
    UsageIndex,
};
use codeskimr_cli::{GlobalOptions, McpArgs};
use codeskimr_codes::ErrorCode;
use codeskimr_i18n::Messages;
use codeskimr_mcp::{HttpService, McpServer, Ports, Tools, serve_stdio};
use codeskimr_storage::SqliteStore;

use crate::clock::SystemClock;
use crate::config_files;
use crate::locations::Locations;
use crate::principal;
use crate::source_arg::folder_spec;
use crate::wiring::{
    Analysis, Secrets, build_analysis, build_scanner, index_runner, open_store, os_secrets,
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
    let store = open_store(locations, &config)?;
    let project = store
        .project_by_source(&spec.source)?
        .ok_or_else(|| CodedError::new(ErrorCode::ProjectNotFound))?;
    if store.latest_snapshot(project.id)?.is_none() {
        return Err(CodedError::new(ErrorCode::ProjectNotScanned));
    }
    let messages = Arc::new(Messages::load().map_err(CodedError::caused_by)?);
    let runner = index_runner(locations);
    let wiring = Wiring {
        locations,
        messages: Arc::clone(&messages),
        runner,
    };
    let ports = build_ports(store, &config, wiring, (EventBus::new(), &os_secrets()))?;
    let tools = Tools::new(ports, config.mcp, project.id);
    let server = McpServer::new(tools, &messages, principal::remote_client())?;
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| CodedError::new(ErrorCode::UnexpectedInternalError))?
        .block_on(serve_stdio(server))
}

/// The MCP tools over streamable HTTP for the running app (blueprint 10.4): every registered
/// project, no default one, the same store file as the UI and the same event bus, so a `rescan`
/// of an agent shows up in the UI like a scan from the UI. The caller is the local owner; the web
/// server lets only requests with the installation token reach the service.
///
/// # Errors
/// Any coded failure of opening the store, loading the language packs or the message texts.
pub fn http_service(
    config: &Config,
    locations: &Locations,
    (events, secrets): (EventBus, &Arc<Secrets>),
) -> Result<HttpService, CodedError> {
    http_service_probing(config, locations, (events, secrets), None)
}

/// Like [`http_service`], but the tools are started through `runner` instead of as real processes
/// (tests use this to say which tools exist).
///
/// # Errors
/// The same as [`http_service`].
pub fn http_service_probing(
    config: &Config,
    locations: &Locations,
    (events, secrets): (EventBus, &Arc<Secrets>),
    runner: Option<Arc<dyn IndexRunner>>,
) -> Result<HttpService, CodedError> {
    let store = open_store(locations, config)?;
    let messages = Arc::new(Messages::load().map_err(CodedError::caused_by)?);
    let wiring = Wiring {
        locations,
        messages: Arc::clone(&messages),
        runner: runner.unwrap_or_else(|| index_runner(locations)),
    };
    let ports = build_ports(store, config, wiring, (events, secrets))?;
    let tools = Tools::unscoped(ports, config.mcp.clone());
    let server = McpServer::new(tools, &messages, principal::remote_client())?;
    Ok(codeskimr_mcp::http_service(&server))
}

/// What the ports of the tools are built with besides the store and the configuration.
struct Wiring<'a> {
    locations: &'a Locations,
    messages: Arc<Messages>,
    runner: Arc<dyn IndexRunner>,
}

/// The ports of the tools: the store for everything it implements, the analysis ports the
/// convention check and the snippet fingerprints read files and language packs through, and the
/// scanner for `rescan`.
fn build_ports(
    store: Arc<SqliteStore>,
    config: &Config,
    wiring: Wiring<'_>,
    (events, secrets): (EventBus, &Arc<Secrets>),
) -> Result<Ports, CodedError> {
    let Wiring {
        locations,
        messages,
        runner,
    } = wiring;
    let Analysis {
        discovery,
        analyzer,
        snippets,
        ..
    } = build_analysis(config, locations)?;
    let checker = ConventionChecker::new(discovery, analyzer, config.conventions.clone());
    let scanner: Arc<dyn ProjectScanner> =
        Arc::new(build_scanner(&store, config, locations, events, secrets)?);
    let projects: Arc<dyn ProjectRepository> = store.clone();
    let usages: Arc<dyn UsageIndex> = store.clone();
    let similarity: Arc<dyn SimilarityIndex> = store.clone();
    let catalog: Arc<dyn ProjectCatalog> = store.clone();
    let tool_check = tool_checker(&store, (locations, runner))?;
    let index: Arc<dyn CodeIndex> = store;
    Ok(Ports {
        projects,
        index,
        usages,
        similarity,
        snippets,
        checker: Arc::new(checker),
        clock: Arc::new(SystemClock),
        scanner,
        catalog,
        tool_check,
        messages,
    })
}
