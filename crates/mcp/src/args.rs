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

/// Arguments of `find_usages`.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(crate = "rmcp::schemars")]
pub struct UsagesArgs {
    /// A symbol id (digits only) or a qualified name such as `Namespace.Type.member`.
    pub symbol: String,
    /// `incoming` (who uses the symbol, the default) or `outgoing` (what the symbol uses).
    pub direction: Option<String>,
    /// Steps the walk goes, at least 1; the configured default if left out, never more than the configured maximum.
    pub depth: Option<u32>,
    /// Whether references from test code are listed; `false` if left out.
    pub include_tests: Option<bool>,
    /// Id of the project; the project of this server if left out.
    pub project: Option<u64>,
    /// Number of results; the configured default if left out, never more than the configured maximum.
    pub limit: Option<u32>,
}

/// Arguments of `impact_of_change`.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(crate = "rmcp::schemars")]
pub struct ImpactArgs {
    /// A symbol id (digits only) or a qualified name such as `Namespace.Type.member`.
    pub symbol: String,
    /// Id of the project; the project of this server if left out.
    pub project: Option<u64>,
    /// Number of symbols; the configured default if left out, never more than the configured maximum.
    pub limit: Option<u32>,
}

/// Arguments of `find_similar_code`. Exactly one of `symbol` and `snippet` is given.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(crate = "rmcp::schemars")]
pub struct SimilarArgs {
    /// A symbol id (digits only) or a qualified name of a function: find functions that resemble it.
    pub symbol: Option<String>,
    /// Code to compare with the project, ideally one complete function.
    pub snippet: Option<String>,
    /// Id of the language pack the snippet is written in, such as `csharp`; required with `snippet`.
    pub language: Option<String>,
    /// Smallest similarity in percent (1 to 100); the configured default if left out.
    pub min_similarity_percent: Option<u32>,
    /// Id of the project; the project of this server if left out.
    pub project: Option<u64>,
    /// Number of results; the configured default if left out, never more than the configured maximum.
    pub limit: Option<u32>,
}

/// Arguments of `find_constants`.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(crate = "rmcp::schemars")]
pub struct ConstantsArgs {
    /// Part of the constant name; the characters must appear in this order, case does not matter.
    pub query: String,
    /// Id of the project; the project of this server if left out.
    pub project: Option<u64>,
    /// Number of results; the configured default if left out, never more than the configured maximum.
    pub limit: Option<u32>,
}

/// Arguments of `check_conventions`.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(crate = "rmcp::schemars")]
pub struct ConventionsArgs {
    /// Files or folders of the project to check, relative to its root, for example the files
    /// of a change; everything if left out. A path that is not part of the project selects nothing.
    pub paths: Option<Vec<String>>,
    /// Id of the project; the project of this server if left out.
    pub project: Option<u64>,
    /// Number of findings; the configured default if left out, never more than the configured maximum.
    pub limit: Option<u32>,
}
