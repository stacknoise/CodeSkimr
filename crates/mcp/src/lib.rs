//! MCP server for code agents. Never depends on secrets.
//!
//! The server is a driving adapter (blueprint 4.1, 10.4): it reads through the ports of
//! `codeskimr_application` and never touches a database itself. [`Tools`] holds the synchronous
//! logic of `search_symbols`, `get_symbol`, `get_architecture`, `find_usages` and
//! `impact_of_change`; [`McpServer`] wraps it for the protocol and [`serve_stdio`] runs it over
//! standard input and output.

mod args;
mod consts;
mod error;
mod server;
mod tools;

pub use args::{ArchitectureArgs, ImpactArgs, SearchArgs, SymbolArgs, UsagesArgs};
pub use error::{CodeReport, McpError};
pub use server::McpServer;
pub use tools::{Ports, SnapshotInfo, Tools};

use codeskimr_application::error::CodedError;
use codeskimr_codes::ErrorCode;
use rmcp::ServiceExt;
use rmcp::transport::stdio;

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
