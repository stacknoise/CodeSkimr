//! The command behind the status bar: real measurements of the running program, never decoration
//! (blueprint 15.5).

use std::sync::OnceLock;
use std::time::{Duration, Instant};

use codeskimr_application::access;
use codeskimr_application::error::CodedError;
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
pub struct StatusInfo;

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
        })
    }
}
