//! MCP server for code agents. Never depends on secrets.
//!
//! The server is a driving adapter (blueprint 4.1, 10.4): it reads through the ports of
//! `codeskimr_application` and never touches a database itself. [`Tools`] holds the synchronous
//! logic of `search_symbols`, `get_symbol`, `get_architecture`, `find_usages`,
//! `impact_of_change`, `get_dependencies`, `find_similar_code`, `find_constants`,
//! `check_conventions` and `rescan`; [`McpServer`] wraps it for the protocol, [`serve_stdio`]
//! runs it over standard input and output and [`http_service`] offers it over streamable HTTP.

mod args;
mod consts;
mod error;
mod server;
mod tools;

pub use args::{
    ArchitectureArgs, ConstantsArgs, ConventionsArgs, DependenciesArgs, ImpactArgs, RescanArgs,
    SearchArgs, SimilarArgs, SymbolArgs, UsagesArgs,
};
pub use error::{CodeReport, McpError};
pub use server::McpServer;
pub use tools::{Ports, SnapshotInfo, Tools};

use std::sync::Arc;

use codeskimr_application::error::CodedError;
use codeskimr_codes::ErrorCode;
use rmcp::ServiceExt;
use rmcp::transport::stdio;
use rmcp::transport::streamable_http_server::session::never::NeverSessionManager;
use rmcp::transport::streamable_http_server::{StreamableHttpServerConfig, StreamableHttpService};

/// The streamable HTTP transport of the server (blueprint 10.4): a `tower` service the web
/// server mounts at `/mcp`. It decides nothing about who may call it; the web server checks the
/// token and the `Host` and `Origin` headers first.
pub type HttpService = StreamableHttpService<McpServer, NeverSessionManager>;

/// Serves `server` over streamable HTTP, stateless: every request is answered on its own with a
/// JSON body, so no session outlives a request and a restart of the app loses nothing. All
/// requests run as the principal `server` was built with.
#[must_use]
pub fn http_service(server: &McpServer) -> HttpService {
    let handler = server.clone();
    let config = StreamableHttpServerConfig::default()
        .with_legacy_session_mode(false)
        .with_json_response(true)
        .with_max_request_body_bytes(consts::HTTP_MAX_BODY_BYTES);
    StreamableHttpService::new(
        move || Ok(handler.clone()),
        Arc::new(NeverSessionManager::default()),
        config,
    )
}

/// Serves `server` over standard input and output until the client disconnects.
///
/// # Errors
/// `APP-901` if the protocol handshake or the transport fails.
pub async fn serve_stdio(server: McpServer) -> Result<(), CodedError> {
    let running = server
        .serve(stdio())
        .await
        .map_err(|_| CodedError::new(ErrorCode::UnexpectedInternalError))?;
    running
        .waiting()
        .await
        .map(|_| ())
        .map_err(|_| CodedError::new(ErrorCode::UnexpectedInternalError))
}
