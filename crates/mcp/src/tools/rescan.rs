//! `rescan`.

use std::time::Instant;

use codeskimr_application::access;
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
}

impl Tools {
    /// Scans the project again, processing only the files that changed since the last scan, and
    /// returns the new snapshot. This is the only tool that changes anything.
    ///
    /// # Errors
    /// `AUTH-401` if the principal may not scan, `MCP-201` if the project is not registered, or
    /// the code of the stage that failed.
    pub fn rescan(&self, principal: &Principal, args: &RescanArgs) -> Result<Value, McpError> {
        access::require(principal, SCAN_TOOL_PERMISSION)?;
        let project = self.project(args.project)?;
        let started = Instant::now();
        let snapshot = self.scanner().rescan(principal, project.id)?;
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
        })
    }
}
