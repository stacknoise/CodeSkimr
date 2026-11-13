//! `rescan`.

use std::time::Instant;

use codeskimr_application::access;
use codeskimr_domain::Principal;
use serde::Serialize;
use serde_json::Value;

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
        to_json(&Answer {
            snapshot: self.snapshot_info(&snapshot),
            mode: snapshot.mode.label(),
            files: snapshot.stats.files,
            symbols: snapshot.stats.symbols,
            references: snapshot.stats.references,
            duration_ms: u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX),
        })
    }
}
