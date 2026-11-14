//! CPU time of this process, the energy proxy of blueprint 16.2 next to the wall time of a scan.

use std::time::Duration;

use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

/// Total CPU time (all threads) this process has used so far; `None` if the system does not tell.
#[must_use]
pub fn process_cpu_time() -> Option<Duration> {
    let pid = Pid::from_u32(std::process::id());
    let mut system = System::new().ok()?;
    system.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[pid]),
        true,
        ProcessRefreshKind::nothing().with_cpu(),
    );
    let millis = system.process(pid)?.accumulated_cpu_time();
    Some(Duration::from_millis(millis))
}

/// CPU time used since `before`, in seconds; 0 if either measurement is missing.
#[must_use]
pub fn seconds_since(before: Option<Duration>) -> f64 {
    match (before, process_cpu_time()) {
        (Some(start), Some(now)) => now.saturating_sub(start).as_secs_f64(),
        _ => 0.0,
    }
}
