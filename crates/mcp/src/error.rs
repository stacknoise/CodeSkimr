//! Errors of the MCP tools and how they reach the caller (blueprint 11.5).

use std::collections::BTreeMap;
use std::fmt;

use codeskimr_application::error::CodedError;
use codeskimr_codes::{Coded, ErrorCode};
use serde::Serialize;

use crate::consts::{ID_SEPARATOR, PARAM_DETAIL, PARAM_PROJECT, PARAM_SYMBOL};

/// Why a tool call failed. Each variant has exactly one registered code.
#[derive(Debug)]
pub enum McpError {
    /// An argument is missing or has a wrong value (`MCP-101`); the text says which.
    ArgumentsInvalid(String),
    /// The project is not registered (`MCP-201`).
    ProjectNotRegistered(u64),
    /// The symbol does not exist in the newest snapshot (`MCP-202`).
    SymbolNotFound(String),
    /// The project has no snapshot yet (`MCP-203`).
    ProjectNotScanned(u64),
    /// A port failed; its own code is kept.
    Port(CodedError),
}

impl From<CodedError> for McpError {
    fn from(error: CodedError) -> Self {
        Self::Port(error)
    }
}

impl Coded for McpError {
    fn code(&self) -> ErrorCode {
        match self {
            Self::ArgumentsInvalid(_) => ErrorCode::ToolArgumentsInvalid,
            Self::ProjectNotRegistered(_) => ErrorCode::ProjectNotRegistered,
            Self::SymbolNotFound(_) => ErrorCode::SymbolNotFound,
            Self::ProjectNotScanned(_) => ErrorCode::ProjectNotScanned,
            Self::Port(error) => error.code(),
        }
    }
}

impl fmt::Display for McpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code().id())?;
        f.write_str(ID_SEPARATOR)?;
        f.write_str(self.code().description())
    }
}

impl std::error::Error for McpError {}

/// A code with its English description and parameters: the shape of a tool error and of a notice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CodeReport {
    /// Registered code such as `MCP-202`.
    pub code: &'static str,
    /// English developer description of the code.
    pub description: &'static str,
    /// Values that explain this occurrence.
    pub params: BTreeMap<&'static str, String>,
}

impl CodeReport {
    /// A report of `code` with the given parameters.
    #[must_use]
    pub fn new<const N: usize>(code: ErrorCode, params: [(&'static str, String); N]) -> Self {
        Self {
            code: code.id(),
            description: code.description(),
            params: BTreeMap::from(params),
        }
    }
}

impl McpError {
    /// The report sent to the caller.
    #[must_use]
    pub fn report(&self) -> CodeReport {
        let code = self.code();
        match self {
            Self::ArgumentsInvalid(detail) => {
                CodeReport::new(code, [(PARAM_DETAIL, detail.clone())])
            }
            Self::ProjectNotRegistered(project) | Self::ProjectNotScanned(project) => {
                CodeReport::new(code, [(PARAM_PROJECT, project.to_string())])
            }
            Self::SymbolNotFound(symbol) => CodeReport::new(code, [(PARAM_SYMBOL, symbol.clone())]),
            Self::Port(_) => CodeReport::new(code, []),
        }
    }
}
