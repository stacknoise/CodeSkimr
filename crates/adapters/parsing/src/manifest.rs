//! The `language.toml` of a language pack.

use std::collections::HashMap;

use serde::Deserialize;

/// How much a language pack delivers (blueprint 8.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tier {
    /// Symbols with members, doc comments and references.
    Full,
    /// Types and functions only.
    Basic,
}

/// Visibility rules of a language.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VisibilityRules {
    /// Visibility label of a declaration without a visibility keyword.
    pub default: Option<String>,
    /// Keyword text to visibility label.
    #[serde(default)]
    pub keywords: HashMap<String, String>,
}

/// Parsed `language.toml`. Unknown keys are rejected so typos do not pass silently.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackManifest {
    /// Id of the language, equal to the folder name.
    pub id: String,
    /// Name of the built-in grammar.
    pub grammar: String,
    /// Tier of the pack.
    pub tier: Tier,
    /// File extensions without the dot.
    pub extensions: Vec<String>,
    /// Separator inside qualified names.
    pub separator: String,
    /// Syntax node kinds that are comments.
    pub comment_nodes: Vec<String>,
    /// Project manifest files such as `Cargo.toml`; used by later stages, not by parsing.
    #[serde(default)]
    pub manifest_files: Vec<String>,
    /// Glob patterns of test files; used by later stages, not by parsing.
    #[serde(default)]
    pub test_patterns: Vec<String>,
    /// Prefixes removed from documentation comment lines.
    #[serde(default)]
    pub doc_prefixes: Vec<String>,
    /// Visibility rules.
    #[serde(default)]
    pub visibility: VisibilityRules,
}
