//! `codeskimr mcp`: serves the MCP tools for one scanned project over standard input and output.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{CodeIndex, ProjectRepository};
use codeskimr_cli::{GlobalOptions, McpArgs};
use codeskimr_codes::ErrorCode;
use codeskimr_i18n::Messages;
use codeskimr_mcp::{McpServer, Tools, serve_stdio};

use crate::clock::SystemClock;
use crate::config_files;
use crate::locations::Locations;
use crate::source_arg::folder_spec;
use crate::wiring::open_store;

/// Serves MCP until the client disconnects.
///
/// The project is the folder given with `--project`, or the current folder. It must have been
/// scanned with `codeskimr scan` before. Nothing but the protocol is written to standard output.
///
/// # Errors
/// - `PRJ-201` if the folder is not a registered project.
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
    let projects: Arc<dyn ProjectRepository> = store.clone();
    let index: Arc<dyn CodeIndex> = store;
    let tools = Tools::new(
        projects,
        index,
        Arc::new(SystemClock),
        config.mcp,
        project.id,
    );
    let messages = Messages::load().map_err(CodedError::caused_by)?;
    let server = McpServer::new(tools, &messages)?;
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| CodedError::new(ErrorCode::UnexpectedInternalError))?
        .block_on(serve_stdio(server))
}
