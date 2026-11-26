//! The arguments of the tools. Their schemas are generated from these types.

use rmcp::schemars::JsonSchema;
use serde::{Deserialize, Deserializer};

/// A symbol reference as an agent sends it: the id from a search answer is a JSON number there, so
/// a number is as good as the same digits in a string (#399).
#[derive(Deserialize, JsonSchema)]
#[serde(untagged)]
#[schemars(crate = "rmcp::schemars", inline)]
enum SymbolRef {
    /// A qualified name, or an id written as a string.
    Text(String),
    /// An id as a JSON number.
    Id(u64),
}

impl SymbolRef {
    /// The reference as the text the tools take.
    fn into_text(self) -> String {
        match self {
            Self::Text(text) => text,
            Self::Id(id) => id.to_string(),
        }
    }
}

/// Reads a required `symbol` that is a string or a number.
fn symbol_text<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    SymbolRef::deserialize(deserializer).map(SymbolRef::into_text)
}

/// Reads an optional `symbol` that is a string or a number.
fn symbol_text_opt<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<String>, D::Error> {
    Ok(Option::<SymbolRef>::deserialize(deserializer)?.map(SymbolRef::into_text))
}

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
    /// A symbol id (digits, as a string or a number) or a qualified name such as `Namespace.Type.member`.
    #[serde(deserialize_with = "symbol_text")]
    #[schemars(with = "SymbolRef")]
    pub symbol: String,
    /// Id of the project; the project of this server if left out.
    pub project: Option<u64>,
}

/// Arguments of `get_context`.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(crate = "rmcp::schemars")]
pub struct ContextArgs {
    /// A symbol id (digits, as a string or a number) or a qualified name such as `Namespace.Type.member`.
    #[serde(deserialize_with = "symbol_text")]
    #[schemars(with = "SymbolRef")]
    pub symbol: String,
    #[schemars(extend("enum" = crate::consts::BUDGETS))]
    /// How much room the answer may take: `short` (a few entries per list, no documentation),
    /// `medium` (the default) or `long`. The number of entries per budget is configured.
    pub budget: Option<String>,
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

/// Arguments of `get_project`.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(crate = "rmcp::schemars")]
pub struct ProjectArgs {
    /// Id of the project; the project of this server if left out.
    pub project: Option<u64>,
}

/// Arguments of `find_usages`.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(crate = "rmcp::schemars")]
pub struct UsagesArgs {
    /// A symbol id (digits, as a string or a number) or a qualified name such as `Namespace.Type.member`.
    #[serde(deserialize_with = "symbol_text")]
    #[schemars(with = "SymbolRef")]
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
    /// A symbol id (digits, as a string or a number) or a qualified name such as `Namespace.Type.member`.
    #[serde(deserialize_with = "symbol_text")]
    #[schemars(with = "SymbolRef")]
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
    /// A symbol id (digits, as a string or a number) or a qualified name of a function: find functions that resemble it.
    #[serde(default, deserialize_with = "symbol_text_opt")]
    #[schemars(with = "Option<SymbolRef>")]
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

/// Arguments of `find_unused`.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(crate = "rmcp::schemars")]
pub struct UnusedArgs {
    /// `all` (the default), `types` or `functions`.
    pub kind: Option<String>,
    /// Id of a language pack, such as `csharp`: only symbols of this language.
    pub language: Option<String>,
    /// `likely`, `public` or `approximate`: only symbols with this assessment.
    pub assessment: Option<String>,
    /// Whether test code counts; `false` if left out.
    pub include_tests: Option<bool>,
    /// Id of the project; the project of this server if left out.
    pub project: Option<u64>,
    /// Number of results; the configured default if left out, never more than the configured
    /// maximum.
    pub limit: Option<u32>,
}

/// Arguments of `preview_rename`.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(crate = "rmcp::schemars")]
pub struct RenameArgs {
    /// Id or qualified name of the symbol that would be renamed.
    pub symbol: String,
    /// The name it would get: letters, digits and underscores, not starting with a digit.
    pub new_name: String,
    /// Id of the project; the project of this server if left out.
    pub project: Option<u64>,
    /// Number of places and of text lines listed; the configured default if left out, never more
    /// than the configured maximum. The counts are always whole.
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

/// Arguments of `get_dependencies`. Exactly one of `symbol` and `layer` is given.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(crate = "rmcp::schemars")]
pub struct DependenciesArgs {
    /// A symbol id (digits, as a string or a number) or a qualified name of a namespace, type or member; a namespace
    /// or type includes everything it contains.
    #[serde(default, deserialize_with = "symbol_text_opt")]
    #[schemars(with = "Option<SymbolRef>")]
    pub symbol: Option<String>,
    /// Name of a layer of the project, as `get_architecture` lists them: the dependencies between
    /// this layer and the others.
    pub layer: Option<String>,
    /// `outgoing` (what the target depends on, the default) or `incoming` (what depends on it).
    pub direction: Option<String>,
    /// Whether references from test code count; `false` if left out. Applies to `symbol` only.
    pub include_tests: Option<bool>,
    /// Id of the project; the project of this server if left out.
    pub project: Option<u64>,
    /// Number of listed dependencies; the configured default if left out, never more than the configured maximum.
    pub limit: Option<u32>,
}

/// Arguments of `list_annotations`.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(crate = "rmcp::schemars")]
pub struct AnnotationsArgs {
    /// A symbol id (digits, as a string or a number) or a qualified name: only the annotations of
    /// this symbol (every overload of the name). All annotations of the project if left out.
    #[serde(default, deserialize_with = "symbol_text_opt")]
    #[schemars(with = "Option<SymbolRef>")]
    pub symbol: Option<String>,
    /// Only this kind: `critical`, `note` or `task`.
    pub kind: Option<String>,
    /// Only tasks in this status: `open`, `in_progress` or `done`.
    pub status: Option<String>,
    /// Id of the project; the project of this server if left out.
    pub project: Option<u64>,
    /// Number of results; the configured default if left out, never more than the configured maximum.
    pub limit: Option<u32>,
    /// Results to skip, to read the next page; 0 if left out.
    pub offset: Option<u32>,
}

/// Arguments of `get_alerts`.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(crate = "rmcp::schemars")]
pub struct AlertsArgs {
    /// `open` (the default) or `dismissed`.
    pub state: Option<String>,
    /// Only alerts of this kind: `changed`, `removed` or `orphaned`.
    pub kind: Option<String>,
    /// Id of the project; the project of this server if left out.
    pub project: Option<u64>,
    /// Number of results; the configured default if left out, never more than the configured maximum.
    pub limit: Option<u32>,
    /// Results to skip, to read the next page; 0 if left out.
    pub offset: Option<u32>,
}

/// Arguments of `rescan`.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(crate = "rmcp::schemars")]
pub struct RescanArgs {
    /// Id of the project; the project of this server if left out.
    pub project: Option<u64>,
    /// Files and folders, relative to the project root, to look at for changes; everything else
    /// stays as the last scan has it. The whole project if left out.
    pub paths: Option<Vec<String>>,
}
