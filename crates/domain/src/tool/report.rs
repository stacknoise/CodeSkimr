//! The result of checking the tools of a project.

use std::collections::BTreeMap;

use crate::consts::{
    TOOL_STATE_MISSING, TOOL_STATE_READY, TOOL_STATE_TOO_OLD, TOOL_STATE_VERSION_UNKNOWN,
};
use crate::file::LanguageId;
use crate::snapshot::Timestamp;
use crate::tool::spec::ToolRole;
use crate::vocabulary::labeled_enum;

labeled_enum!(
    /// What the check found out about one tool.
    ToolState {
        /// It runs and its version is as new as required.
        Ready => TOOL_STATE_READY,
        /// It cannot be started: not installed, or not found on the search path.
        Missing => TOOL_STATE_MISSING,
        /// It runs, but its version is older than required.
        TooOld => TOOL_STATE_TOO_OLD,
        /// A version is required and the tool did not say one (or did not answer in time).
        VersionUnknown => TOOL_STATE_VERSION_UNKNOWN,
    }
);

/// The check of one tool for one language.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolFinding {
    /// The language the tool serves.
    pub language: LanguageId,
    /// Name of the tool.
    pub tool: String,
    /// Indexer or toolchain.
    pub role: ToolRole,
    /// The outcome.
    pub state: ToolState,
    /// The version that was found, if the tool said one.
    pub found: Option<String>,
    /// The version that is required, if any.
    pub required: Option<String>,
    /// Message key of what the tool is for.
    pub purpose_key: String,
    /// Message key of how to install it.
    pub install_key: String,
}

/// One tool for all the languages of the project that it serves.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolEntry {
    /// Name of the tool.
    pub tool: String,
    /// Indexer or toolchain.
    pub role: ToolRole,
    /// The outcome.
    pub state: ToolState,
    /// The version that was found.
    pub found: Option<String>,
    /// The version that is required.
    pub required: Option<String>,
    /// Message key of what the tool is for.
    pub purpose_key: String,
    /// Message key of how to install it.
    pub install_key: String,
    /// The languages of the project that need it, in order.
    pub languages: Vec<LanguageId>,
}

/// What a check of a project found, as it is stored with the project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolReport {
    /// When the check ran.
    pub checked_at: Timestamp,
    /// One finding per language and tool.
    pub findings: Vec<ToolFinding>,
}

/// Groups equal findings of different languages: TypeScript and JavaScript share their tools.
type EntryKey = (String, ToolState, Option<String>, Option<String>);

impl ToolReport {
    /// The languages that have tools, in order. A language without tools (it has no level 2)
    /// is not listed.
    #[must_use]
    pub fn languages(&self) -> Vec<LanguageId> {
        let mut languages: Vec<LanguageId> = Vec::new();
        for finding in &self.findings {
            if !languages.contains(&finding.language) {
                languages.push(finding.language.clone());
            }
        }
        languages.sort();
        languages
    }

    /// Whether every tool of `language` is ready, so that level 2 can run for it. A language
    /// without tools is not ready.
    #[must_use]
    pub fn level2_ready(&self, language: &LanguageId) -> bool {
        let mut own = self
            .findings
            .iter()
            .filter(|finding| &finding.language == language)
            .peekable();
        own.peek().is_some() && own.all(|finding| finding.state == ToolState::Ready)
    }

    /// All tools, one entry per tool and outcome, ordered by name.
    #[must_use]
    pub fn tools(&self) -> Vec<ToolEntry> {
        let mut entries: BTreeMap<EntryKey, ToolEntry> = BTreeMap::new();
        for finding in &self.findings {
            let key = (
                finding.tool.clone(),
                finding.state,
                finding.found.clone(),
                finding.required.clone(),
            );
            entries
                .entry(key)
                .or_insert_with(|| ToolEntry {
                    tool: finding.tool.clone(),
                    role: finding.role,
                    state: finding.state,
                    found: finding.found.clone(),
                    required: finding.required.clone(),
                    purpose_key: finding.purpose_key.clone(),
                    install_key: finding.install_key.clone(),
                    languages: Vec::new(),
                })
                .languages
                .push(finding.language.clone());
        }
        let mut tools: Vec<ToolEntry> = entries.into_values().collect();
        for entry in &mut tools {
            entry.languages.sort();
        }
        tools
    }

    /// The tools that are not ready: what the user has to install or update.
    #[must_use]
    pub fn problems(&self) -> Vec<ToolEntry> {
        self.tools()
            .into_iter()
            .filter(|entry| entry.state != ToolState::Ready)
            .collect()
    }
}
