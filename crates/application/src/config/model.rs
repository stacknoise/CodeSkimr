//! The typed configuration. There is no string-keyed access anywhere (blueprint section 14).

use serde::Deserialize;

/// The complete, validated configuration. Built by [`crate::config::Config::resolve`].
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Thresholds that rate metric values as good, medium or poor.
    pub metrics: MetricThresholds,
    /// Performance budgets.
    pub budgets: Budgets,
    /// Limits for MCP responses.
    pub mcp: McpLimits,
    /// Snapshot retention.
    pub snapshots: SnapshotRetention,
    /// File watcher behaviour.
    pub watcher: Watcher,
    /// Scan behaviour.
    pub scan: Scan,
}

/// Thresholds for the metric ratings.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetricThresholds {
    /// Cyclomatic complexity up to this value is good.
    pub cyclomatic_good_max: u32,
    /// Cyclomatic complexity up to this value is medium.
    pub cyclomatic_medium_max: u32,
    /// Maintainability index from this value is good.
    pub maintainability_good_min: u32,
    /// Maintainability index from this value is medium.
    pub maintainability_medium_min: u32,
    /// Instability up to this value is stable.
    pub instability_stable_max: f64,
    /// Instability up to this value is medium.
    pub instability_medium_max: f64,
}

/// Performance budgets of blueprint section 16.1, in the unit named by the field.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Budgets {
    /// Cold start until the UI is usable, in milliseconds.
    pub cold_start_ms: u32,
    /// Quick search per keystroke, in milliseconds.
    pub search_keystroke_ms: u32,
    /// Switching the selected type in the explorer, in milliseconds.
    pub explorer_switch_ms: u32,
    /// Usages query at depth 2, in milliseconds.
    pub usages_query_ms: u32,
    /// Graph layout of up to 200 nodes, in milliseconds.
    pub graph_layout_ms: u32,
    /// Full scan of 100k lines of code, in milliseconds.
    pub full_scan_100k_loc_ms: u32,
    /// Incremental re-scan after saving one file, in milliseconds.
    pub incremental_rescan_ms: u32,
    /// Idle CPU load in percent.
    pub idle_cpu_percent: u32,
    /// Memory for 1M lines of code, in megabytes.
    pub memory_1m_loc_mb: u32,
    /// Binary size, in megabytes.
    pub binary_size_mb: u32,
    /// Regression against the stored baseline that fails the benchmark, in percent.
    pub regression_tolerance_percent: u32,
}

/// Limits for MCP responses.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct McpLimits {
    /// Number of results when the caller gives no limit.
    pub default_limit: u32,
    /// Largest number of results a caller may ask for.
    pub max_limit: u32,
}

/// Snapshot retention.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotRetention {
    /// Number of snapshots kept per project.
    pub retention_count: u32,
}

/// File watcher behaviour.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Watcher {
    /// File events within this window are collapsed into one re-scan, in milliseconds.
    pub debounce_ms: u32,
}

/// Scan behaviour.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scan {
    /// Folder names that are never scanned, at any depth.
    pub excluded_folders: Vec<String>,
}
