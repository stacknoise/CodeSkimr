//! `get_project`.

use codeskimr_application::access;
use codeskimr_application::tools::{ToolView, ToolsView};
use codeskimr_domain::Principal;
use serde::Serialize;
use serde_json::Value;

use super::{SnapshotInfo, Tools, to_json};
use crate::args::ProjectArgs;
use crate::consts::READ_TOOL_PERMISSION;
use crate::error::McpError;

#[derive(Serialize)]
struct LanguageInfo {
    language: String,
    files: u32,
    loc: u32,
    /// True if every tool level 2 needs for the language is ready; false for a language without
    /// level 2 (no tool names it).
    level2_ready: bool,
}

#[derive(Serialize)]
pub(super) struct ToolInfo {
    tool: String,
    role: &'static str,
    /// `ready`, `missing`, `too_old` or `version_unknown`.
    state: &'static str,
    languages: Vec<String>,
    found: Option<String>,
    required: Option<String>,
    /// What the tool is needed for.
    purpose: String,
    /// How to install it.
    install: String,
}

#[derive(Serialize)]
struct Answer {
    project: u64,
    name: String,
    /// The newest snapshot; `None` before the first scan.
    snapshot: Option<SnapshotInfo>,
    languages: Vec<LanguageInfo>,
    /// Seconds since the Unix epoch of the check of the tools below.
    tools_checked_at: Option<i64>,
    tools: Vec<ToolInfo>,
}

impl Tools {
    fn tool_info(&self, tool: &ToolView) -> ToolInfo {
        ToolInfo {
            tool: tool.tool.clone(),
            role: tool.role,
            state: tool.state,
            languages: tool.languages.clone(),
            found: tool.found.clone(),
            required: tool.required.clone(),
            purpose: self.text(&tool.purpose_key),
            install: self.text(&tool.install_key),
        }
    }

    /// The tools of a project with their texts, from a check that runs now.
    pub(super) fn tool_infos(&self, view: &ToolsView) -> Vec<ToolInfo> {
        view.tools.iter().map(|tool| self.tool_info(tool)).collect()
    }

    /// Checks the tools the project needs for level 2 and returns them with its languages. A
    /// missing tool is an answer, not an error. The check starts only version queries of the
    /// tools and stores its result with the project.
    ///
    /// # Errors
    /// `AUTH-401` if the principal may not read, `MCP-201` if the project is not registered, or
    /// the code of a failing port.
    pub fn get_project(
        &self,
        principal: &Principal,
        args: &ProjectArgs,
    ) -> Result<Value, McpError> {
        access::require(principal, READ_TOOL_PERMISSION)?;
        let project = self.project(args.project)?;
        let report = self.tool_check().check(project.id)?;
        let view = ToolsView::of(&report);
        let ready = |language: &str| {
            view.languages
                .iter()
                .any(|entry| entry.language == language && entry.level2_ready)
        };
        let languages = self
            .catalog()
            .languages(project.id)?
            .iter()
            .map(|share| LanguageInfo {
                language: share.language.to_string(),
                files: share.files,
                loc: share.loc,
                level2_ready: ready(share.language.as_str()),
            })
            .collect();
        let snapshot = self.index().latest_snapshot(project.id)?;
        to_json(&Answer {
            project: project.id.get(),
            name: project.name.as_str().to_owned(),
            snapshot: snapshot.as_ref().map(|found| self.snapshot_info(found)),
            languages,
            tools_checked_at: view.checked_at,
            tools: self.tool_infos(&view),
        })
    }
}
