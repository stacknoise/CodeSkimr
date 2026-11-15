//! `rescan`.

use std::time::Instant;

use codeskimr_application::access;
use codeskimr_application::ports::Rescanned;
use codeskimr_application::tools::ToolsView;
use codeskimr_domain::Principal;
use serde::Serialize;
use serde_json::Value;

use super::project::ToolInfo;
use super::{SnapshotInfo, Tools, to_json};
use crate::args::RescanArgs;
use crate::consts::SCAN_TOOL_PERMISSION;
use crate::error::McpError;

#[derive(Serialize)]
struct Answer {
    /// The snapshot the scan wrote; every other tool answers from it from now on.
    snapshot: SnapshotInfo,
    mode: &'static str,
    files: u32,
    symbols: u32,
    references: u32,
    duration_ms: u64,
    /// The tools level 2 needs, checked after the scan; empty if the check failed or the project
    /// has no languages with level 2.
    tools: Vec<ToolInfo>,
    /// The paths the scan was restricted to; empty if it looked at the whole project.
    paths: Vec<String>,
    /// Paths that name no file of the project, a typo for one; the scan changed nothing for them.
    unmatched_paths: Vec<String>,
    /// Whether the scan had to be a full one and so ignored `paths`.
    paths_ignored: bool,
}

impl Tools {
    /// Scans the project again, processing only the files that changed since the last scan (only
    /// those below `paths` if given), and returns the new snapshot. This is the only tool that changes anything.
    ///
    /// # Errors
    /// `AUTH-401` if the principal may not scan, `MCP-201` if the project is not registered, or
    /// the code of the stage that failed.
    pub fn rescan(&self, principal: &Principal, args: &RescanArgs) -> Result<Value, McpError> {
        access::require(principal, SCAN_TOOL_PERMISSION)?;
        let project = self.project(args.project)?;
        let started = Instant::now();
        let Rescanned { snapshot, paths } = self.scanner().rescan_paths(
            principal,
            project.id,
            args.paths.as_deref().unwrap_or_default(),
        )?;
        let elapsed = started.elapsed();
        // A failing check never fails the scan: the answer then lists no tools.
        let view = self
            .tool_check()
            .check(project.id)
            .map_or_else(|_| ToolsView::unchecked(), |report| ToolsView::of(&report));
        to_json(&Answer {
            snapshot: self.snapshot_info(&snapshot),
            mode: snapshot.mode.label(),
            files: snapshot.stats.files,
            symbols: snapshot.stats.symbols,
            references: snapshot.stats.references,
            duration_ms: u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX),
            tools: self.tool_infos(&view),
            paths: args.paths.clone().unwrap_or_default(),
            unmatched_paths: paths.unmatched,
            paths_ignored: paths.ignored,
        })
    }
}
