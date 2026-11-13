//! Fixed texts of the configuration: file names, key paths and the embedded defaults.

/// The default configuration, embedded at build time (blueprint section 14).
pub const DEFAULT_CONFIG: &str = include_str!("../../../../config/default.toml");

/// File name of the project configuration inside a repository.
pub const PROJECT_CONFIG_FILE: &str = ".codeskimr.toml";

/// Separator of dotted key paths such as `metrics.cyclomatic_good_max`.
pub const KEY_SEPARATOR: char = '.';

/// Key under which a raw command-line value is wrapped so that it parses as TOML.
pub const OVERRIDE_VALUE_KEY: &str = "value";

/// Smallest allowed share (instability).
pub const UNIT_MIN: f64 = 0.0;
/// Largest allowed share (instability).
pub const UNIT_MAX: f64 = 1.0;
/// Largest allowed percentage.
pub const PERCENT_MAX: u32 = 100;

/// Key paths used in errors. They mirror `config/default.toml`.
pub mod keys {
    /// `metrics.cyclomatic_good_max`
    pub const CYCLOMATIC_GOOD_MAX: &str = "metrics.cyclomatic_good_max";
    /// `metrics.cyclomatic_medium_max`
    pub const CYCLOMATIC_MEDIUM_MAX: &str = "metrics.cyclomatic_medium_max";
    /// `metrics.maintainability_good_min`
    pub const MAINTAINABILITY_GOOD_MIN: &str = "metrics.maintainability_good_min";
    /// `metrics.maintainability_medium_min`
    pub const MAINTAINABILITY_MEDIUM_MIN: &str = "metrics.maintainability_medium_min";
    /// `metrics.instability_stable_max`
    pub const INSTABILITY_STABLE_MAX: &str = "metrics.instability_stable_max";
    /// `metrics.instability_medium_max`
    pub const INSTABILITY_MEDIUM_MAX: &str = "metrics.instability_medium_max";
    /// `budgets.cold_start_ms`
    pub const COLD_START_MS: &str = "budgets.cold_start_ms";
    /// `budgets.search_keystroke_ms`
    pub const SEARCH_KEYSTROKE_MS: &str = "budgets.search_keystroke_ms";
    /// `budgets.explorer_switch_ms`
    pub const EXPLORER_SWITCH_MS: &str = "budgets.explorer_switch_ms";
    /// `budgets.usages_query_ms`
    pub const USAGES_QUERY_MS: &str = "budgets.usages_query_ms";
    /// `budgets.graph_layout_ms`
    pub const GRAPH_LAYOUT_MS: &str = "budgets.graph_layout_ms";
    /// `budgets.full_scan_100k_loc_ms`
    pub const FULL_SCAN_MS: &str = "budgets.full_scan_100k_loc_ms";
    /// `budgets.incremental_rescan_ms`
    pub const INCREMENTAL_RESCAN_MS: &str = "budgets.incremental_rescan_ms";
    /// `budgets.idle_cpu_percent`
    pub const IDLE_CPU_PERCENT: &str = "budgets.idle_cpu_percent";
    /// `budgets.memory_1m_loc_mb`
    pub const MEMORY_MB: &str = "budgets.memory_1m_loc_mb";
    /// `budgets.binary_size_mb`
    pub const BINARY_SIZE_MB: &str = "budgets.binary_size_mb";
    /// `budgets.regression_tolerance_percent`
    pub const REGRESSION_TOLERANCE_PERCENT: &str = "budgets.regression_tolerance_percent";
    /// `mcp.default_limit`
    pub const MCP_DEFAULT_LIMIT: &str = "mcp.default_limit";
    /// `mcp.max_limit`
    pub const MCP_MAX_LIMIT: &str = "mcp.max_limit";
    /// `snapshots.retention_count`
    pub const RETENTION_COUNT: &str = "snapshots.retention_count";
    /// `watcher.debounce_ms`
    pub const DEBOUNCE_MS: &str = "watcher.debounce_ms";
    /// `scan.excluded_folders`
    pub const EXCLUDED_FOLDERS: &str = "scan.excluded_folders";
}

/// Characters that make a folder name a path.
pub const PATH_SEPARATORS: [char; 2] = ['/', '\\'];
