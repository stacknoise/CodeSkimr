//! The command behind the status bar: real measurements of the running program, never decoration
//! (blueprint 15.5).

use std::sync::OnceLock;
use std::time::{Duration, Instant};

use codeskimr_application::access;
use codeskimr_application::config::Budgets;
use codeskimr_application::error::CodedError;
use codeskimr_application::pipeline::analyze_workers;
use codeskimr_application::registry::{Command, CommandDescriptor, CommandId, Request};
use codeskimr_domain::Permission;
use serde::Serialize;

use super::consts::{BYTES_PER_MEGABYTE, STATUS_DESCRIPTION, STATUS_INFO, STATUS_TITLE};

static STARTED: OnceLock<Instant> = OnceLock::new();
static READY: OnceLock<Duration> = OnceLock::new();

/// Remembers the moment the program started. Later calls keep the first moment.
pub fn mark_started() {
    STARTED.get_or_init(Instant::now);
}

/// Remembers that the server answers: the time since [`mark_started`] is the start time.
pub fn mark_ready() {
    if let Some(started) = STARTED.get() {
        READY.get_or_init(|| started.elapsed());
    }
}

/// The measurements of the status bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct StatusOut {
    /// Milliseconds from the start of the program until the server answered; `None` if it was
    /// not measured (the server of a test).
    pub startup_ms: Option<u64>,
    /// Memory of the program in megabytes; `None` if the system does not tell.
    pub memory_mb: Option<u64>,
    /// The time budgets of blueprint 16.1 in the configuration, for the performance screen.
    pub budgets: BudgetTargets,
    /// How many threads the analyze stage of a scan uses (one per core the system offers, at most `scan.max_workers`).
    pub workers: u32,
}

/// The targets of the time budgets, all in milliseconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct BudgetTargets {
    /// Cold start until the UI is usable.
    pub cold_start: u32,
    /// Quick search per keystroke.
    pub search_keystroke: u32,
    /// Switching the selected type in the explorer.
    pub explorer_switch: u32,
    /// Usages query at depth 2.
    pub usages_query: u32,
    /// Graph layout of up to 200 nodes.
    pub graph_layout: u32,
    /// Full scan of 100k lines of code.
    pub full_scan_100k_loc: u32,
    /// Incremental re-scan after saving one file.
    pub incremental_rescan: u32,
}

impl From<&Budgets> for BudgetTargets {
    fn from(budgets: &Budgets) -> Self {
        Self {
            cold_start: budgets.cold_start_ms,
            search_keystroke: budgets.search_keystroke_ms,
            explorer_switch: budgets.explorer_switch_ms,
            usages_query: budgets.usages_query_ms,
            graph_layout: budgets.graph_layout_ms,
            full_scan_100k_loc: budgets.full_scan_100k_loc_ms,
            incremental_rescan: budgets.incremental_rescan_ms,
        }
    }
}

fn startup_ms() -> Option<u64> {
    READY
        .get()
        .map(|ready| u64::try_from(ready.as_millis()).unwrap_or(u64::MAX))
}

fn memory_mb() -> Option<u64> {
    memory_stats::memory_stats().map(|usage| {
        let bytes = u64::try_from(usage.physical_mem).unwrap_or(u64::MAX);
        bytes.div_ceil(BYTES_PER_MEGABYTE)
    })
}

/// Reports how long the start took and how much memory the program holds.
pub struct StatusInfo {
    pub(super) budgets: BudgetTargets,
    /// `scan.max_workers` of the configuration.
    pub(super) max_workers: u32,
}

impl Command for StatusInfo {
    type Input = ();
    type Output = StatusOut;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(STATUS_INFO),
        title_key: STATUS_TITLE,
        description_key: STATUS_DESCRIPTION,
        shortcut: None,
        read_only: true,
        permission: Permission::Read,
    };

    fn execute(&self, request: Request<'_, ()>) -> Result<StatusOut, CodedError> {
        access::require(request.principal, Permission::Read)?;
        Ok(StatusOut {
            startup_ms: startup_ms(),
            memory_mb: memory_mb(),
            budgets: self.budgets,
            workers: u32::try_from(analyze_workers(self.max_workers)).unwrap_or(u32::MAX),
        })
    }
}
