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

/// The limit that counts: `scan.max_memory_mb`, but at most the share of the installed memory
/// that leaves the system room to work. Without a reading of the installed memory the configured
/// value counts alone.
pub fn effective_limit_mb(configured_mb: u32, installed_bytes: u64) -> u64 {
    let configured = u64::from(configured_mb);
    if installed_bytes == 0 {
        return configured;
    }
    let share = installed_bytes / BYTES_PER_MB * MEMORY_RAM_SHARE_PERCENT / PERCENT_WHOLE;
    configured.min(share)
}

/// The memory limit of the scans of this program.
pub fn scan_limit(config: &Config) -> MemoryLimit {
    // A system that cannot say how much memory it has leaves the configured value alone.
    let installed = System::new().map_or(0, |mut system| {
        system.refresh_memory();
        system.total_memory()
    });
    MemoryLimit::new(
        Arc::new(ProcessMemory),
        effective_limit_mb(config.scan.max_memory_mb, installed),
        Duration::from_millis(MEMORY_CHECK_INTERVAL_MS),
    )
}

#[cfg(test)]
mod tests {
    use super::effective_limit_mb;

    const GB: u64 = 1_073_741_824;

    #[test]
    fn the_configured_value_counts_on_a_large_machine() {
        assert_eq!(effective_limit_mb(16384, 128 * GB), 16384);
    }

    #[test]
    fn seventy_percent_of_a_small_machine_counts_when_it_is_lower() {
        assert_eq!(effective_limit_mb(16384, 8 * GB), 5734);
    }

    #[test]
    fn the_share_rounds_down_just_below_the_setting() {
        assert_eq!(effective_limit_mb(5735, 8 * GB), 5734);
    }

    #[test]
    fn a_tiny_setting_counts_as_it_is() {
        assert_eq!(effective_limit_mb(1, 8 * GB), 1);
    }

    #[test]
    fn an_unknown_amount_of_memory_leaves_the_configured_value() {
        assert_eq!(effective_limit_mb(4096, 0), 4096);
    }
}
