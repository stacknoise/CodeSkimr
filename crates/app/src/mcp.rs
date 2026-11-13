//! `codeskimr mcp`: serves the MCP tools for one scanned project over standard input and output.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use codeskimr_application::config::Config;
use codeskimr_application::conventions::ConventionChecker;
use codeskimr_application::error::CodedError;
use codeskimr_application::events::EventBus;
use codeskimr_application::ports::{
    CodeIndex, ProjectRepository, ProjectScanner, SimilarityIndex, UsageIndex,
};
use codeskimr_cli::{GlobalOptions, McpArgs};
use codeskimr_codes::ErrorCode;
use codeskimr_i18n::Messages;
use codeskimr_mcp::{McpServer, Ports, Tools, serve_stdio};
use codeskimr_storage::SqliteStore;

use crate::clock::SystemClock;
use crate::config_files;
use crate::locations::Locations;
use crate::principal;
use crate::source_arg::folder_spec;
use crate::wiring::{Analysis, build_analysis, build_scanner, open_store};

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
    let ports = build_ports(store, &config, locations)?;
    let tools = Tools::new(ports, config.mcp, project.id);
    let messages = Messages::load().map_err(CodedError::caused_by)?;
    let server = McpServer::new(tools, &messages, principal::resolve())?;
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| CodedError::new(ErrorCode::UnexpectedInternalError))?
        .block_on(serve_stdio(server))
}

/// The ports of the tools: the store for everything it implements, the analysis ports the
/// convention check and the snippet fingerprints read files and language packs through, and the
/// scanner for `rescan`.
fn build_ports(
    store: Arc<SqliteStore>,
    config: &Config,
    locations: &Locations,
) -> Result<Ports, CodedError> {
    let Analysis {
        discovery,
        analyzer,
        snippets,
    } = build_analysis(config, locations)?;
    let checker = ConventionChecker::new(discovery, analyzer, config.conventions.clone());
    let scanner: Arc<dyn ProjectScanner> =
        Arc::new(build_scanner(&store, config, locations, EventBus::new())?);
    let projects: Arc<dyn ProjectRepository> = store.clone();
    let usages: Arc<dyn UsageIndex> = store.clone();
    let similarity: Arc<dyn SimilarityIndex> = store.clone();
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
    })
}
