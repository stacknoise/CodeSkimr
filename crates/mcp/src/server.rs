//! The protocol layer: lists the tools and runs them for `rmcp`.

use std::future::Future;
use std::sync::Arc;

use codeskimr_application::error::CodedError;
use codeskimr_codes::ErrorCode;
use codeskimr_codes::i18n::{
    MCP_SERVER_INSTRUCTIONS, MCP_TOOL_GET_ARCHITECTURE, MCP_TOOL_GET_SYMBOL,
    MCP_TOOL_SEARCH_SYMBOLS,
};
use codeskimr_domain::Principal;
use codeskimr_i18n::Messages;
use rmcp::ErrorData;
use rmcp::ServerHandler;
use rmcp::model::{
    CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock, Implementation,
    JsonObject, ListToolsResult, PaginatedRequestParams, ServerCapabilities, ServerConfig, Tool,
    ToolAnnotations,
};
use rmcp::schemars::JsonSchema;
use rmcp::service::{RequestContext, RoleServer};
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::args::{ArchitectureArgs, SearchArgs, SymbolArgs};
use crate::consts::{
    SERVER_NAME, SERVER_VERSION, TEXT_LANGUAGE, TOOL_GET_ARCHITECTURE, TOOL_GET_SYMBOL,
    TOOL_SEARCH_SYMBOLS,
};
use crate::error::McpError;
use crate::tools::Tools;

/// The MCP server: the tools plus the texts it announces to clients.
pub struct McpServer {
    tools: Arc<Tools>,
    definitions: Vec<Tool>,
    instructions: String,
    principal: Principal,
}

fn definition<A: JsonSchema + 'static>(
    messages: &Messages,
    name: &'static str,
    description_key: &str,
) -> Result<Tool, CodedError> {
    let description = messages
        .format(TEXT_LANGUAGE, description_key, None)
        .map_err(CodedError::caused_by)?;
    Ok(Tool::new(name, description, JsonObject::new())
        .with_input_schema::<A>()
        .with_annotations(ToolAnnotations::new().read_only(true)))
}

/// The arguments of a call as the type of the tool.
fn parse_arguments<A: DeserializeOwned>(arguments: Option<JsonObject>) -> Result<A, McpError> {
    serde_json::from_value(Value::Object(arguments.unwrap_or_default()))
        .map_err(|error| McpError::ArgumentsInvalid(error.to_string()))
}

fn failure(error: &McpError) -> CallToolResult {
    let report = error.report();
    let mut result = CallToolResult::error(vec![ContentBlock::text(
        serde_json::to_string(&report).unwrap_or_default(),
    )]);
    result.structured_content = serde_json::to_value(&report).ok();
    result
}

impl McpServer {
    /// Builds the server around `tools`. Every call runs as `principal`, which the composition
    /// root resolved: stdio is single-user, so it is the local owner. A transport that
    /// authenticates its callers builds one server per caller.
    ///
    /// # Errors
    /// `I18N-201` if a text of the tool list is missing.
    pub fn new(
        tools: Tools,
        messages: &Messages,
        principal: Principal,
    ) -> Result<Self, CodedError> {
        Ok(Self {
            principal,
            tools: Arc::new(tools),
            definitions: vec![
                definition::<SearchArgs>(messages, TOOL_SEARCH_SYMBOLS, MCP_TOOL_SEARCH_SYMBOLS)?,
                definition::<SymbolArgs>(messages, TOOL_GET_SYMBOL, MCP_TOOL_GET_SYMBOL)?,
                definition::<ArchitectureArgs>(
                    messages,
                    TOOL_GET_ARCHITECTURE,
                    MCP_TOOL_GET_ARCHITECTURE,
                )?,
            ],
            instructions: messages
                .format(TEXT_LANGUAGE, MCP_SERVER_INSTRUCTIONS, None)
                .map_err(CodedError::caused_by)?,
        })
    }

    /// Runs one tool. A failing tool is an answer with `is_error`, not a protocol error.
    fn run(
        tools: &Tools,
        principal: Principal,
        name: &str,
        arguments: Option<JsonObject>,
    ) -> CallToolResult {
        let outcome = match name {
            TOOL_SEARCH_SYMBOLS => {
                parse_arguments(arguments).and_then(|args| tools.search_symbols(&principal, &args))
            }
            TOOL_GET_SYMBOL => {
                parse_arguments(arguments).and_then(|args| tools.get_symbol(&principal, &args))
            }
            TOOL_GET_ARCHITECTURE => parse_arguments(arguments)
                .and_then(|args| tools.get_architecture(&principal, &args)),
            _ => Err(McpError::ArgumentsInvalid(name.to_owned())),
        };
        outcome.map_or_else(|error| failure(&error), CallToolResult::structured)
    }
}

impl ServerHandler for McpServer {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new(SERVER_NAME, SERVER_VERSION))
            .with_instructions(self.instructions.clone())
    }

    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListToolsResult, ErrorData>> + Send + '_ {
        std::future::ready(Ok(ListToolsResult {
            tools: self.definitions.clone(),
            ..ListToolsResult::default()
        }))
    }

    fn get_tool(&self, name: &str) -> Option<Tool> {
        self.definitions
            .iter()
            .find(|tool| tool.name == name)
            .cloned()
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, ErrorData> {
        let tools = Arc::clone(&self.tools);
        let principal = self.principal;
        let result = tokio::task::spawn_blocking(move || {
            Self::run(&tools, principal, &request.name, request.arguments)
        })
        .await
        .map_err(|_| {
            ErrorData::internal_error(ErrorCode::UnexpectedInternalError.description(), None)
        })?;
        Ok(result.into())
    }
}
