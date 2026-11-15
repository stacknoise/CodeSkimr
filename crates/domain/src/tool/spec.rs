//! What a language pack says about the project files that name the version of a tool.

use std::collections::BTreeMap;

use crate::consts::{
    CEILING_ANY, CEILING_EXACT, CEILING_SAME_BAND, CEILING_SAME_MAJOR, CEILING_SAME_MINOR,
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
    /// How far above the named version the project lets a tool go, if the file says so.
    pub roll_forward: Option<RollForwardSpec>,
}

labeled_enum!(
    /// The newest version that a project lets through, counted from the version it names.
    Ceiling {
        /// Only the named version.
        Exact => CEILING_EXACT,
        /// The same major, the same minor and the same hundreds of the patch number (the feature
        /// band of the .NET SDK: 8.0.1xx).
        SameBand => CEILING_SAME_BAND,
        /// The same major and the same minor.
        SameMinor => CEILING_SAME_MINOR,
        /// The same major.
        SameMajor => CEILING_SAME_MAJOR,
        /// Any newer version.
        Any => CEILING_ANY,
    }
);

/// Where a project file limits how new a tool may be, such as `rollForward` of `global.json`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RollForwardSpec {
    /// The keys down to the policy in the same file.
    pub path: Vec<String>,
    /// What counts if the file names no policy.
    pub default: Ceiling,
    /// What each policy of the file means. A policy that is not listed limits nothing: it is not
    /// guessed.
    pub values: BTreeMap<String, Ceiling>,
}

impl RollForwardSpec {
    /// The ceiling for the policy text a file holds, or the default for no text.
    #[must_use]
    pub fn ceiling_of(&self, policy: Option<&str>) -> Ceiling {
        match policy {
            None => self.default,
            Some(text) => self.values.get(text).copied().unwrap_or(Ceiling::Any),
        }
    }
}
