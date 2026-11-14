//! What a language pack says about the project files that name the version of a tool.

use crate::consts::{
    REQUIREMENT_FORMAT_JSON, REQUIREMENT_FORMAT_TEXT, REQUIREMENT_FORMAT_TOML, TOOL_ROLE_INDEXER,
    TOOL_ROLE_TOOLCHAIN,
};
use crate::vocabulary::labeled_enum;

labeled_enum!(
    /// What a tool is for.
    ToolRole {
        /// Produces the index that level 2 reads (ADR 0033).
        Indexer => TOOL_ROLE_INDEXER,
        /// The compiler, runtime or SDK that the indexer and the project need.
        Toolchain => TOOL_ROLE_TOOLCHAIN,
    }
);

labeled_enum!(
    /// How a file of the project spells the version it requires.
    RequirementFormat {
        /// A JSON document; `path` names the keys down to a string.
        Json => REQUIREMENT_FORMAT_JSON,
        /// A TOML document; `path` names the keys down to a string.
        Toml => REQUIREMENT_FORMAT_TOML,
        /// Plain text; the first line that is not empty and not a comment is the version.
        Text => REQUIREMENT_FORMAT_TEXT,
    }
);

/// Where a project states the version of a tool it needs, such as `global.json`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequirementSpec {
    /// The file, relative to the project root.
    pub file: String,
    /// How to read it.
    pub format: RequirementFormat,
    /// The keys down to the version for `Json` and `Toml`; empty for `Text`.
    pub path: Vec<String>,
}
