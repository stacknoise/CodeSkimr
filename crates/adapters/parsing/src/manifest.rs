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
    /// Kind labels of symbols that have no visibility at all, such as namespaces.
    #[serde(default)]
    pub unrated_kinds: Vec<String>,
    /// Visibility label of a declaration without a visibility keyword.
    pub default: Option<String>,
    /// Visibility label of a declaration without a keyword, by the kind label of its parent.
    #[serde(default)]
    pub parent_defaults: HashMap<String, String>,
    /// Keyword text to visibility label.
    #[serde(default)]
    pub keywords: HashMap<String, String>,
}

/// Tags that enclose the summary inside a documentation comment, such as `<summary>` in C#.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DocSummary {
    /// Opening tag.
    pub open: String,
    /// Closing tag.
    pub close: String,
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
    /// Documentation text between these tags is the doc of the symbol; without them the whole
    /// comment is.
    #[serde(default)]
    pub doc_summary: Option<DocSummary>,
    /// Modifier keyword to symbol kind label; a declaration with the keyword gets that kind.
    #[serde(default)]
    pub modifier_kinds: HashMap<String, String>,
    /// Modifier keywords that make a declaration static.
    #[serde(default)]
    pub static_keywords: Vec<String>,
    /// Syntax node kinds that are identifiers; fingerprints treat them all alike.
    #[serde(default)]
    pub identifier_nodes: Vec<String>,
    /// Syntax node kinds that are literals, taken as one token each; fingerprints ignore values.
    #[serde(default)]
    pub literal_nodes: Vec<String>,
    /// Visibility rules.
    #[serde(default)]
    pub visibility: VisibilityRules,
}
