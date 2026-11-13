//! The typed configuration. There is no string-keyed access anywhere (blueprint section 14).

use codeskimr_domain::{FingerprintParams, ModelError};
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
    /// Syntax fingerprints for similar-code search and duplicate detection.
    pub similarity: Similarity,
    /// Scan behaviour.
    pub scan: Scan,
    /// The local server.
    pub server: Server,
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
    /// Steps a usages walk goes when the caller gives no depth.
    pub default_depth: u32,
    /// Most steps a walk may go; the depth of the impact analysis.
    pub max_depth: u32,
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
    /// A batch of file events is handed over at the latest this long after its first event, in
    /// milliseconds.
    pub max_wait_ms: u32,
}

/// Scan behaviour.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scan {
    /// Folder names that are never scanned, at any depth.
    pub excluded_folders: Vec<String>,
    /// Time limit for parsing one file, in milliseconds.
    pub parse_timeout_ms: u32,
    /// Largest total size of the scanned files of one source, in megabytes.
    pub max_source_mb: u32,
}

/// The local server (blueprint 10.3).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Server {
    /// TCP port on the loopback address.
    pub port: u16,
}

/// Syntax fingerprints (blueprint 10.4).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Similarity {
    /// Number of syntax tokens per hashed n-gram.
    pub ngram_size: u32,
    /// Number of consecutive n-grams per winnowing window.
    pub window_size: u32,
    /// Functions with fewer syntax tokens get no fingerprints.
    pub min_tokens: u32,
}

impl Similarity {
    /// The winnowing parameters of this section.
    ///
    /// # Errors
    /// [`ModelError::NotPositive`] if a value is zero; a resolved [`Config`] never is.
    pub fn fingerprint_params(&self) -> Result<FingerprintParams, ModelError> {
        FingerprintParams::new(widen(self.ngram_size), widen(self.window_size))
    }
}

fn widen(value: u32) -> usize {
    usize::try_from(value).unwrap_or(usize::MAX)
}
