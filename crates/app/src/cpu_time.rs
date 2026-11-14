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
        _ => f64::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::{process_cpu_time, seconds_since};
    use std::time::Duration;

    #[test]
    fn without_a_first_measurement_the_time_is_zero() {
        assert!(seconds_since(None).abs() < f64::EPSILON);
    }

    #[test]
    fn the_time_since_a_later_start_is_never_negative() {
        let later = process_cpu_time().map(|now| now + Duration::from_secs(60));
        assert!(seconds_since(later).abs() < f64::EPSILON);
    }
}
