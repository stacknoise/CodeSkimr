//! The arguments of the tools. Their schemas are generated from these types.

use rmcp::schemars::JsonSchema;
use serde::Deserialize;

/// Arguments of `search_symbols`.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(crate = "rmcp::schemars")]
pub struct SearchArgs {
    /// Part of the symbol name; the characters must appear in this order, case does not matter.
    pub query: String,
    /// Only symbols of this kind, such as `class`, `function` or `constant`.
    pub kind: Option<String>,
    /// Id of the project; the project of this server if left out.
    pub project: Option<u64>,
    /// Number of results; the configured default if left out, never more than the configured maximum.
    pub limit: Option<u32>,
}

/// Arguments of `get_symbol`.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(crate = "rmcp::schemars")]
pub struct SymbolArgs {
    /// A symbol id (digits only) or a qualified name such as `Namespace.Type.member`.
    pub symbol: String,
    /// Id of the project; the project of this server if left out.
    pub project: Option<u64>,
}

/// Arguments of `get_architecture`.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(crate = "rmcp::schemars")]
pub struct ArchitectureArgs {
    /// Id of the project; the project of this server if left out.
    pub project: Option<u64>,
}
