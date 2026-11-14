//! The report in the form the driving adapters send out: the UI commands, `codeskimr scan --json`
//! and the MCP tool `get_project` all use these types, so they cannot drift apart.

use codeskimr_domain::{ToolEntry, ToolReport};
use serde::Serialize;

/// One tool and the languages of the project that need it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ToolView {
    /// Name of the tool, such as `scip-dotnet`.
    pub tool: String,
    /// `indexer` or `toolchain`.
    pub role: &'static str,
    /// `ready`, `missing`, `too_old` or `version_unknown`.
    pub state: &'static str,
    /// The languages of the project that need the tool.
    pub languages: Vec<String>,
    /// The version that was found.
    pub found: Option<String>,
    /// The version that is required.
    pub required: Option<String>,
    /// Message key of what the tool is needed for.
    pub purpose_key: String,
    /// Message key of how to install it.
    pub install_key: String,
}

/// Whether level 2 can run for one language.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Readiness {
    /// The language.
    pub language: String,
    /// True if every tool of the language is ready.
    pub level2_ready: bool,
}

/// What the last check of a project found.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct ToolsView {
    /// When the check ran, in seconds since the Unix epoch; `None` if the project was never
    /// checked (or the check could not be stored).
    pub checked_at: Option<i64>,
    /// Level 2 readiness of every language that has tools.
    pub languages: Vec<Readiness>,
    /// All tools of the project, the ones that are not ready too.
    pub tools: Vec<ToolView>,
}

impl ToolView {
    fn of(entry: &ToolEntry) -> Self {
        Self {
            tool: entry.tool.clone(),
            role: entry.role.label(),
            state: entry.state.label(),
            languages: entry.languages.iter().map(ToString::to_string).collect(),
            found: entry.found.clone(),
            required: entry.required.clone(),
            purpose_key: entry.purpose_key.clone(),
            install_key: entry.install_key.clone(),
        }
    }
}

impl ToolsView {
    /// The view of a report.
    #[must_use]
    pub fn of(report: &ToolReport) -> Self {
        Self {
            checked_at: Some(report.checked_at.0),
            languages: report
                .languages()
                .iter()
                .map(|language| Readiness {
                    language: language.to_string(),
                    level2_ready: report.level2_ready(language),
                })
                .collect(),
            tools: report.tools().iter().map(ToolView::of).collect(),
        }
    }

    /// The view of a project that has no report.
    #[must_use]
    pub fn unchecked() -> Self {
        Self::default()
    }

    /// The tools that are not ready.
    pub fn problems(&self) -> impl Iterator<Item = &ToolView> {
        self.tools
            .iter()
            .filter(|tool| tool.state != codeskimr_domain::ToolState::Ready.label())
    }
}
