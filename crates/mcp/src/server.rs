//! The protocol layer: lists the tools and runs them for `rmcp`.

use std::future::Future;
use std::sync::Arc;

use codeskimr_application::access;
use codeskimr_application::error::CodedError;
use codeskimr_codes::ErrorCode;
use codeskimr_codes::i18n::{
    MCP_SERVER_INSTRUCTIONS, MCP_TOOL_CHECK_CONVENTIONS, MCP_TOOL_FIND_CONSTANTS,
    MCP_TOOL_FIND_SIMILAR_CODE, MCP_TOOL_FIND_USAGES, MCP_TOOL_GET_ARCHITECTURE,
    MCP_TOOL_GET_DEPENDENCIES, MCP_TOOL_GET_SYMBOL, MCP_TOOL_IMPACT_OF_CHANGE, MCP_TOOL_RESCAN,
    MCP_TOOL_SEARCH_SYMBOLS,
};
use codeskimr_domain::{Permission, Principal};
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

use crate::args::{
    ArchitectureArgs, ConstantsArgs, ConventionsArgs, DependenciesArgs, ImpactArgs, RescanArgs,
    SearchArgs, SimilarArgs, SymbolArgs, UsagesArgs,
};
use crate::consts::{
    READ_TOOL_PERMISSION, SCAN_TOOL_PERMISSION, SERVER_NAME, SERVER_VERSION, TEXT_LANGUAGE,
    TOOL_CHECK_CONVENTIONS, TOOL_FIND_CONSTANTS, TOOL_FIND_SIMILAR_CODE, TOOL_FIND_USAGES,
    TOOL_GET_ARCHITECTURE, TOOL_GET_DEPENDENCIES, TOOL_GET_SYMBOL, TOOL_IMPACT_OF_CHANGE,
    TOOL_RESCAN, TOOL_SEARCH_SYMBOLS,
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
    definition_with::<A>(messages, name, description_key, true)
}

/// A tool definition; `read_only` tells clients whether calling the tool changes anything.
fn definition_with<A: JsonSchema + 'static>(
    messages: &Messages,
    name: &'static str,
    description_key: &str,
    read_only: bool,
) -> Result<Tool, CodedError> {
    let description = messages
        .format(TEXT_LANGUAGE, description_key, None)
        .map_err(CodedError::caused_by)?;
    Ok(Tool::new(name, description, JsonObject::new())
        .with_input_schema::<A>()
        .with_annotations(ToolAnnotations::new().read_only(read_only)))
}

/// The arguments of a call as the type of the tool.
fn parse_arguments<A: DeserializeOwned>(arguments: Option<JsonObject>) -> Result<A, McpError> {
    serde_json::from_value(Value::Object(arguments.unwrap_or_default()))
        .map_err(|error| McpError::ArgumentsInvalid(error.to_string()))
}

/// Parses the arguments as the type the tool takes and runs the tool with them.
fn call<A: DeserializeOwned>(
    arguments: Option<JsonObject>,
    tool: impl FnOnce(A) -> Result<Value, McpError>,
) -> Result<Value, McpError> {
    parse_arguments(arguments).and_then(tool)
}

fn failure(error: &McpError) -> CallToolResult {
    let report = error.report();
    let mut result = CallToolResult::error(vec![ContentBlock::text(
        serde_json::to_string(&report).unwrap_or_default(),
    )]);
    result.structured_content = serde_json::to_value(&report).ok();
    result
}

/// The tools the server announces, in the order of the list.
fn definitions(messages: &Messages) -> Result<Vec<Tool>, CodedError> {
    Ok(vec![
        definition::<SearchArgs>(messages, TOOL_SEARCH_SYMBOLS, MCP_TOOL_SEARCH_SYMBOLS)?,
        definition::<SymbolArgs>(messages, TOOL_GET_SYMBOL, MCP_TOOL_GET_SYMBOL)?,
        definition::<ArchitectureArgs>(messages, TOOL_GET_ARCHITECTURE, MCP_TOOL_GET_ARCHITECTURE)?,
        definition::<UsagesArgs>(messages, TOOL_FIND_USAGES, MCP_TOOL_FIND_USAGES)?,
        definition::<ImpactArgs>(messages, TOOL_IMPACT_OF_CHANGE, MCP_TOOL_IMPACT_OF_CHANGE)?,
        definition::<DependenciesArgs>(messages, TOOL_GET_DEPENDENCIES, MCP_TOOL_GET_DEPENDENCIES)?,
        definition::<SimilarArgs>(messages, TOOL_FIND_SIMILAR_CODE, MCP_TOOL_FIND_SIMILAR_CODE)?,
        definition::<ConstantsArgs>(messages, TOOL_FIND_CONSTANTS, MCP_TOOL_FIND_CONSTANTS)?,
        definition::<ConventionsArgs>(
            messages,
            TOOL_CHECK_CONVENTIONS,
            MCP_TOOL_CHECK_CONVENTIONS,
        )?,
        definition_with::<RescanArgs>(messages, TOOL_RESCAN, MCP_TOOL_RESCAN, false)?,
    ])
}

/// The permission a call of the tool needs. An unknown name needs `read`, like most tools, so a
/// caller who may read learns nothing from the permission check alone.
fn required_permission(name: &str) -> Permission {
    if name == TOOL_RESCAN {
        SCAN_TOOL_PERMISSION
    } else {
        READ_TOOL_PERMISSION
    }
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
            definitions: definitions(messages)?,
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
        // The permission comes first: before the tool name or the arguments are looked at, so a
        // denied caller cannot tell valid names or argument shapes from invalid ones.
        if let Err(denied) = access::require(&principal, required_permission(name)) {
            return failure(&McpError::from(denied));
        }
        let outcome = match name {
            TOOL_SEARCH_SYMBOLS => call(arguments, |args| tools.search_symbols(&principal, &args)),
            TOOL_GET_SYMBOL => call(arguments, |args| tools.get_symbol(&principal, &args)),
            TOOL_GET_ARCHITECTURE => {
                call(arguments, |args| tools.get_architecture(&principal, &args))
            }
            TOOL_FIND_USAGES => call(arguments, |args| tools.find_usages(&principal, &args)),
            TOOL_IMPACT_OF_CHANGE => {
                call(arguments, |args| tools.impact_of_change(&principal, &args))
            }
            TOOL_GET_DEPENDENCIES => {
                call(arguments, |args| tools.get_dependencies(&principal, &args))
            }
            TOOL_RESCAN => call(arguments, |args| tools.rescan(&principal, &args)),
            TOOL_FIND_SIMILAR_CODE => {
                call(arguments, |args| tools.find_similar_code(&principal, &args))
            }
            TOOL_FIND_CONSTANTS => call(arguments, |args| tools.find_constants(&principal, &args)),
            TOOL_CHECK_CONVENTIONS => {
                call(arguments, |args| tools.check_conventions(&principal, &args))
            }
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
