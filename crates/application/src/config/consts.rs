//! Fixed texts of the configuration: file names, key paths and the embedded defaults.

/// The default configuration, embedded at build time (blueprint section 14).
pub const DEFAULT_CONFIG: &str = include_str!("../../../../config/default.toml");

/// File name of the project configuration inside a repository.
pub const PROJECT_CONFIG_FILE: &str = ".codeskimr.toml";

/// Separator of dotted key paths such as `metrics.cyclomatic_good_max`.
pub const KEY_SEPARATOR: char = '.';

/// Key under which a raw command-line value is wrapped so that it parses as TOML.
pub const OVERRIDE_VALUE_KEY: &str = "value";

/// Joins a key and its value in a TOML line.
pub const ASSIGNMENT: &str = " = ";

/// Key path prefix of the top level of a table.
pub const NO_PREFIX: &str = "";

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
    /// `mcp.default_depth`
    pub const MCP_DEFAULT_DEPTH: &str = "mcp.default_depth";
    /// `mcp.max_depth`
    pub const MCP_MAX_DEPTH: &str = "mcp.max_depth";
    /// `mcp.default_similarity_percent`
    pub const MCP_DEFAULT_SIMILARITY_PERCENT: &str = "mcp.default_similarity_percent";
    /// `mcp.max_snippet_bytes`
    pub const MCP_MAX_SNIPPET_BYTES: &str = "mcp.max_snippet_bytes";
    /// `snapshots.retention_count`
    pub const RETENTION_COUNT: &str = "snapshots.retention_count";
    /// `graph.node_width`
    pub const GRAPH_NODE_WIDTH: &str = "graph.node_width";
    /// `graph.node_height`
    pub const GRAPH_NODE_HEIGHT: &str = "graph.node_height";
    /// `similarity.ngram_size`
    pub const NGRAM_SIZE: &str = "similarity.ngram_size";
    /// `similarity.window_size`
    pub const WINDOW_SIZE: &str = "similarity.window_size";
    /// `similarity.min_tokens`
    pub const MIN_TOKENS: &str = "similarity.min_tokens";
    /// `layers`
    pub const LAYERS: &str = "layers";
    /// `conventions.max_function_lines`
    pub const MAX_FUNCTION_LINES: &str = "conventions.max_function_lines";
    /// `conventions.duplicate_similarity_percent`
    pub const DUPLICATE_SIMILARITY_PERCENT: &str = "conventions.duplicate_similarity_percent";
    /// `conventions.duplicate_min_lines`
    pub const DUPLICATE_MIN_LINES: &str = "conventions.duplicate_min_lines";
    /// `watcher.debounce_ms`
    pub const DEBOUNCE_MS: &str = "watcher.debounce_ms";
    /// `watcher.max_wait_ms`
    pub const MAX_WAIT_MS: &str = "watcher.max_wait_ms";
    /// `scan.excluded_folders`
    pub const EXCLUDED_FOLDERS: &str = "scan.excluded_folders";
    /// `scan.parse_timeout_ms`
    pub const PARSE_TIMEOUT_MS: &str = "scan.parse_timeout_ms";
    /// `scan.max_source_mb`
    pub const MAX_SOURCE_MB: &str = "scan.max_source_mb";
    /// `level2.max_index_bytes`
    pub const LEVEL2_MAX_INDEX_BYTES: &str = "level2.max_index_bytes";
    /// `level2.timeout_seconds`
    pub const LEVEL2_TIMEOUT_SECONDS: &str = "level2.timeout_seconds";
    /// `level2.max_memory_mb`
    pub const LEVEL2_MAX_MEMORY_MB: &str = "level2.max_memory_mb";
    /// `level2.max_output_bytes`
    pub const LEVEL2_MAX_OUTPUT_BYTES: &str = "level2.max_output_bytes";
    /// `level2.min_mapped`
    pub const LEVEL2_MIN_MAPPED: &str = "level2.min_mapped";
    /// `server.port`
    pub const SERVER_PORT: &str = "server.port";
    /// `editor.url_template`
    pub const EDITOR_URL_TEMPLATE: &str = "editor.url_template";
    /// `editor.allowed_schemes`
    pub const EDITOR_ALLOWED_SCHEMES: &str = "editor.allowed_schemes";
    /// `ui.default_theme`
    pub const UI_DEFAULT_THEME: &str = "ui.default_theme";
    /// `ui.dir`
    pub const UI_DIR: &str = "ui.dir";
    /// `ui.launch_code_seconds`
    pub const UI_LAUNCH_CODE_SECONDS: &str = "ui.launch_code_seconds";
}

/// Characters that make a folder name a path.
pub const PATH_SEPARATORS: [char; 2] = ['/', '\\'];

/// Separates the scheme of a URL from the rest.
pub const URL_SCHEME_SEPARATOR: char = ':';
/// Placeholder of the editor template that stands for the file path.
pub const PLACEHOLDER_PATH: &str = "{path}";
/// Characters besides lower-case letters and digits that a URL scheme may contain.
pub const SCHEME_EXTRA_CHARS: [char; 3] = ['+', '-', '.'];

/// Name of the table of the project file that is never read (ADR 0033, decision 9).
pub const UNTRUSTED_PROJECT_TABLE: &str = "level2";

/// Table that holds the scan limits.
pub const SCAN_TABLE: &str = "scan";
/// Settings of [`SCAN_TABLE`] that the project file may only lower: a scanned repository must not
/// lift the protection of the machine that scans it (#54). The user file and the flags may still
/// raise them.
pub const LOWER_ONLY_SCAN_KEYS: [&str; 2] = ["max_source_mb", "parse_timeout_ms"];
