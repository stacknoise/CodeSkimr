//! The memory limit of a scan: the measure of the program and the limit that counts (ADR 0087).

use std::sync::Arc;
use std::time::Duration;

use codeskimr_application::config::Config;
use codeskimr_application::ports::{MemoryGauge, MemoryLimit};
use sysinfo::System;

use crate::consts::{
    BYTES_PER_MB, MEMORY_CHECK_INTERVAL_MS, MEMORY_RAM_SHARE_PERCENT, PERCENT_WHOLE,
};

/// The resident memory of this program, as the system reports it.
#[derive(Debug, Clone, Copy, Default)]
pub struct ProcessMemory;

impl MemoryGauge for ProcessMemory {
    fn used_mb(&self) -> Option<u64> {
        memory_stats::memory_stats().map(|usage| {
            let bytes = u64::try_from(usage.physical_mem).unwrap_or(u64::MAX);
            bytes / BYTES_PER_MB
        })
    }
}

/// The share of the installed memory that leaves the system room to work, in megabytes; `None`
/// without a reading of the installed memory.
pub fn ram_ceiling_mb(installed_bytes: u64) -> Option<u64> {
    (installed_bytes != 0)
        .then(|| installed_bytes / BYTES_PER_MB * MEMORY_RAM_SHARE_PERCENT / PERCENT_WHOLE)
}

/// The memory limit of the scans of this program.
pub fn scan_limit(config: &Config) -> MemoryLimit {
    // A system that cannot say how much memory it has leaves the configured value alone.
    let installed = System::new().map_or(0, |mut system| {
        system.refresh_memory();
        system.total_memory()
    });
    let limit = MemoryLimit::new(
        Arc::new(ProcessMemory),
        u64::from(config.scan.max_memory_mb),
        Duration::from_millis(MEMORY_CHECK_INTERVAL_MS),
    );
    ram_ceiling_mb(installed).map_or(limit.clone(), |ceiling| limit.with_ceiling(ceiling))
}

#[cfg(test)]
mod tests {
    use super::ram_ceiling_mb;

    const GB: u64 = 1_073_741_824;

    #[test]
    fn seventy_percent_of_the_installed_memory_is_the_ceiling() {
        assert_eq!(ram_ceiling_mb(8 * GB), Some(5734));
        assert_eq!(ram_ceiling_mb(128 * GB), Some(91750));
    }

    #[test]
    fn an_unknown_amount_of_memory_gives_no_ceiling() {
        assert_eq!(ram_ceiling_mb(0), None);
    }
}
