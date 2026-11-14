//! The `language.toml` of a language pack.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::consts::{MANIFEST_FILE, TIER_BASIC, TIER_FULL};
use crate::error::ParsingError;
use crate::indexer::IndexerManifest;

/// How much a language pack delivers (blueprint 8.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tier {
    /// Symbols with members, doc comments and references.
    Full,
    /// Types and functions only.
    Basic,
}

impl Tier {
    /// The word for the tier in the language table and on the wire.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Full => TIER_FULL,
            Self::Basic => TIER_BASIC,
        }
    }
}

/// A visibility that follows from the spelling of a name, such as the leading underscore of Python.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NameRule {
    /// The name starts with this text.
    pub prefix: String,
    /// The name also ends with this text; without it the end does not matter.
    #[serde(default)]
    pub suffix: Option<String>,
    /// Visibility label of a matching name.
    pub visibility: String,
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
    /// Visibility label of a declaration without a keyword, by its own kind label. It wins over
    /// the default of the parent kind.
    #[serde(default)]
    pub kind_defaults: HashMap<String, String>,
    /// Keyword text to visibility label.
    #[serde(default)]
    pub keywords: HashMap<String, String>,
    /// Rules by the spelling of the name; the first matching rule counts. They apply when no
    /// keyword does.
    #[serde(default)]
    pub name_rules: Vec<NameRule>,
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

/// Delimiters of a block documentation comment such as `JSDoc` (`/** ... */`, ADR 0021).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DocBlock {
    /// Text that opens the comment, such as `/**`.
    pub open: String,
    /// Text that closes the comment, such as `*/`.
    pub close: String,
    /// Marker that may start every line inside the comment, such as `*`.
    pub line_prefix: String,
    /// A line that starts with this text, such as `@`, ends the description.
    pub stop_at: Option<String>,
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
    /// Whether the path of the file is the first part of every qualified name inside it. True
    /// for languages whose module path is the file path (Rust, Python, `TypeScript`); false for
    /// languages with their own namespaces (ADR 0036).
    #[serde(default = "file_in_names_default")]
    pub file_in_names: bool,
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
    /// Documentation is the run of comments directly before the declaration; the listed node kinds
    /// (attributes) may stand between. Without it the query captures the comments with `@doc`.
    /// A query with a quantified comment in front of a declaration is slow, this is not.
    #[serde(default)]
    pub doc_before_declaration: Option<Vec<String>>,
    /// Documentation text between these tags is the doc of the symbol; without them the whole
    /// comment is.
    #[serde(default)]
    pub doc_summary: Option<DocSummary>,
    /// Block documentation comments (`JSDoc`); with it and without `doc_prefixes` only block
    /// comments are documentation.
    #[serde(default)]
    pub doc_block: Option<DocBlock>,
    /// Modifier keyword to symbol kind label; a declaration with the keyword gets that kind.
    #[serde(default)]
    pub modifier_kinds: HashMap<String, String>,
    /// Kind label of the parent to (kind label of a declaration to kind label it gets there), such
    /// as a `function` inside a `class` being a `method`.
    #[serde(default)]
    pub kinds_by_parent: HashMap<String, HashMap<String, String>>,
    /// Modifier keywords that make a declaration static.
    #[serde(default)]
    pub static_keywords: Vec<String>,
    /// Syntax node kinds that make a child a body when the grammar has no `body` field; a callable
    /// with such a child is measured.
    #[serde(default)]
    pub body_nodes: Vec<String>,
    /// Syntax node kinds that are identifiers; fingerprints treat them all alike.
    #[serde(default)]
    pub identifier_nodes: Vec<String>,
    /// Syntax node kinds that are literals, taken as one token each; fingerprints ignore values.
    #[serde(default)]
    pub literal_nodes: Vec<String>,
    /// Visibility rules.
    #[serde(default)]
    pub visibility: VisibilityRules,
    /// The indexer that gives level 2 references (ADR 0033); without it the language stays on
    /// level 1.
    #[serde(default)]
    pub indexer: Option<IndexerManifest>,
}

impl PackManifest {
    /// Reads and parses the `language.toml` in `dir`.
    ///
    /// # Errors
    /// [`ParsingError::PackInvalid`] if the file is missing or malformed.
    pub fn read(dir: &Path) -> Result<Self, ParsingError> {
        let folder = dir
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        let invalid = |reason: String| ParsingError::invalid(folder.clone(), reason);
        let text =
            fs::read_to_string(dir.join(MANIFEST_FILE)).map_err(|e| invalid(e.to_string()))?;
        toml::from_str(&text).map_err(|e| invalid(e.to_string()))
    }
}

/// The default of `file_in_names`: the file is part of the names.
const fn file_in_names_default() -> bool {
    true
}
