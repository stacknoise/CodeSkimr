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
/// Shortest pause after a level 2 run of the browser, in seconds (`level2.ui_cooldown_seconds`).
pub const UI_COOLDOWN_MIN_SECONDS: u32 = 10;
/// Value of `level2.indexer_memory_mb.<indexer>` that means "use `level2.max_memory_mb`".
pub const MEMORY_FOLLOWS_GLOBAL: u32 = 0;
/// Most memory in megabytes that `level2.indexer_memory_mb.<indexer>` may name (1 TiB).
pub const MAX_INDEXER_MEMORY_MB: u32 = 1_048_576;
/// The name that stands for no indexer in particular (the global limits).
pub const NO_INDEXER: &str = "";
/// Most level 2 runs at the same time that the configuration may allow.
pub const MAX_PARALLEL_RUNS_LIMIT: u32 = 8;

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
    /// `budgets.pack_load_ms`
    pub const PACK_LOAD_MS: &str = "budgets.pack_load_ms";
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
    /// `mcp.context_items_short`
    pub const MCP_CONTEXT_ITEMS_SHORT: &str = "mcp.context_items_short";
    /// `mcp.context_items_medium`
    pub const MCP_CONTEXT_ITEMS_MEDIUM: &str = "mcp.context_items_medium";
    /// `mcp.context_items_long`
    pub const MCP_CONTEXT_ITEMS_LONG: &str = "mcp.context_items_long";
    /// `snapshots.retention_count`
    pub const RETENTION_COUNT: &str = "snapshots.retention_count";
    /// `annotations.dismissed_retention_days`
    pub const ANNOTATIONS_RETENTION_DAYS: &str = "annotations.dismissed_retention_days";
    /// `annotations.max_text_chars`
    pub const ANNOTATIONS_MAX_TEXT_CHARS: &str = "annotations.max_text_chars";
    /// `annotations.default_page_size`
    pub const ANNOTATIONS_DEFAULT_PAGE: &str = "annotations.default_page_size";
    /// `annotations.max_page_size`
    pub const ANNOTATIONS_MAX_PAGE: &str = "annotations.max_page_size";
    /// `annotations.max_assignee_chars`
    pub const ANNOTATIONS_MAX_ASSIGNEE_CHARS: &str = "annotations.max_assignee_chars";
    /// `annotations.max_name_chars`
    pub const ANNOTATIONS_MAX_NAME_CHARS: &str = "annotations.max_name_chars";
    /// `annotations.max_parameter_chars`
    pub const ANNOTATIONS_MAX_PARAMETER_CHARS: &str = "annotations.max_parameter_chars";
    /// `annotations.max_parameters`
    pub const ANNOTATIONS_MAX_PARAMETERS: &str = "annotations.max_parameters";
    /// `annotations.max_title_chars`
    pub const ANNOTATIONS_MAX_TITLE_CHARS: &str = "annotations.max_title_chars";
    /// `graph.node_width`
    pub const GRAPH_NODE_WIDTH: &str = "graph.node_width";
    /// `graph.node_height`
    pub const GRAPH_NODE_HEIGHT: &str = "graph.node_height";
    /// `graph.row_boxes`
    pub const GRAPH_ROW_BOXES: &str = "graph.row_boxes";
    /// `graph.risk_max`
    pub const GRAPH_RISK_MAX: &str = "graph.risk_max";
    /// `graph.max_modules`
    pub const GRAPH_MAX_MODULES: &str = "graph.max_modules";
    /// `graph.max_places`
    pub const GRAPH_MAX_PLACES: &str = "graph.max_places";
    /// `export.max_references`
    pub const EXPORT_MAX_REFERENCES: &str = "export.max_references";
    /// `compare.max_symbols`
    pub const COMPARE_MAX_SYMBOLS: &str = "compare.max_symbols";
    /// `compare.max_usages`
    pub const COMPARE_MAX_USAGES: &str = "compare.max_usages";
    /// `compare.metric_min_cc`
    pub const COMPARE_METRIC_MIN_CC: &str = "compare.metric_min_cc";
    /// `compare.metric_min_mi`
    pub const COMPARE_METRIC_MIN_MI: &str = "compare.metric_min_mi";
    /// `compare.metric_min_ce`
    pub const COMPARE_METRIC_MIN_CE: &str = "compare.metric_min_ce";
    /// `compare.metric_min_instability`
    pub const COMPARE_METRIC_MIN_INSTABILITY: &str = "compare.metric_min_instability";
    /// `compare.metric_loc_growth`
    pub const COMPARE_METRIC_LOC_GROWTH: &str = "compare.metric_loc_growth";
    /// `similarity.ngram_size`
    pub const NGRAM_SIZE: &str = "similarity.ngram_size";
    /// `similarity.window_size`
    pub const WINDOW_SIZE: &str = "similarity.window_size";
    /// `similarity.min_tokens`
    pub const MIN_TOKENS: &str = "similarity.min_tokens";
    /// `layers`
    pub const LAYERS: &str = "layers";
    /// `unused.ignore`
    pub const UNUSED_IGNORE: &str = "unused.ignore";
    /// `rules`
    pub const RULES: &str = "rules";
    /// `unused`
    pub const UNUSED: &str = "unused";
    /// `ignore`, the key of the exception list inside `unused`.
    pub const IGNORE: &str = "ignore";
    /// `conventions.max_function_lines`
    pub const MAX_FUNCTION_LINES: &str = "conventions.max_function_lines";
    /// `conventions.max_cognitive_complexity`
    pub const MAX_COGNITIVE_COMPLEXITY: &str = "conventions.max_cognitive_complexity";
    /// `conventions.max_parameters`
    pub const MAX_PARAMETERS: &str = "conventions.max_parameters";
    /// `keys.max_files`
    pub const KEYS_MAX_FILES: &str = "keys.max_files";
    /// `keys.max_depth`
    pub const KEYS_MAX_DEPTH: &str = "keys.max_depth";
    /// `keys.max_file_kb`
    pub const KEYS_MAX_FILE_KB: &str = "keys.max_file_kb";
    /// `conventions.duplicate_similarity_percent`
    pub const DUPLICATE_SIMILARITY_PERCENT: &str = "conventions.duplicate_similarity_percent";
    /// `conventions.duplicate_min_lines`
    pub const DUPLICATE_MIN_LINES: &str = "conventions.duplicate_min_lines";
    /// `conventions.duplicate_max_owners`
    pub const DUPLICATE_MAX_OWNERS: &str = "conventions.duplicate_max_owners";
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
    /// `scan.max_signature_chars`
    pub const MAX_SIGNATURE_CHARS: &str = "scan.max_signature_chars";
    /// `scan.max_candidates`
    pub const MAX_CANDIDATES: &str = "scan.max_candidates";
    /// `scan.max_memory_mb`
    pub const MAX_MEMORY_MB: &str = "scan.max_memory_mb";
    /// `scan.max_left_out_listed`
    pub const MAX_LEFT_OUT_LISTED: &str = "scan.max_left_out_listed";
    /// `scan.max_scope_entries`
    pub const MAX_SCOPE_ENTRIES: &str = "scan.max_scope_entries";
    /// `scan.overview_depth`
    pub const OVERVIEW_DEPTH: &str = "scan.overview_depth";
    /// `scan.overview_max_folders`
    pub const OVERVIEW_MAX_FOLDERS: &str = "scan.overview_max_folders";
    /// `level2.max_index_bytes`
    pub const LEVEL2_MAX_INDEX_BYTES: &str = "level2.max_index_bytes";
    /// `level2.timeout_seconds`
    pub const LEVEL2_TIMEOUT_SECONDS: &str = "level2.timeout_seconds";
    /// `level2.max_memory_mb`
    pub const LEVEL2_MAX_MEMORY_MB: &str = "level2.max_memory_mb";
    /// `level2.indexer_memory_mb`
    pub const LEVEL2_INDEXER_MEMORY_MB: &str = "level2.indexer_memory_mb";
    /// `level2.max_output_bytes`
    pub const LEVEL2_MAX_OUTPUT_BYTES: &str = "level2.max_output_bytes";
    /// `level2.min_mapped`
    pub const LEVEL2_MIN_MAPPED: &str = "level2.min_mapped";
    /// `level2.max_created_symbols`
    pub const LEVEL2_MAX_CREATED_SYMBOLS: &str = "level2.max_created_symbols";
    /// `level2.confirm_window_cooldown_seconds`
    pub const LEVEL2_CONFIRM_WINDOW_COOLDOWN_SECONDS: &str =
        "level2.confirm_window_cooldown_seconds";
    /// `level2.ui_cooldown_seconds`
    pub const LEVEL2_UI_COOLDOWN_SECONDS: &str = "level2.ui_cooldown_seconds";
    /// `level2.max_parallel_runs`
    pub const LEVEL2_MAX_PARALLEL_RUNS: &str = "level2.max_parallel_runs";
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
    /// `ui.level2_poll_seconds`
    pub const UI_LEVEL2_POLL_SECONDS: &str = "ui.level2_poll_seconds";
    /// `ui.approximate_languages`
    pub const UI_APPROXIMATE_LANGUAGES: &str = "ui.approximate_languages";
    /// `ui.level2_poll_max_seconds`
    pub const UI_LEVEL2_POLL_MAX_SECONDS: &str = "ui.level2_poll_max_seconds";
}

/// Characters that make a folder name a path.
pub const PATH_SEPARATORS: [char; 2] = ['/', '\\'];

/// Separates the scheme of a URL from the rest.
pub const URL_SCHEME_SEPARATOR: char = ':';
/// Placeholder of the editor template that stands for the file path.
pub const PLACEHOLDER_PATH: &str = "{path}";
/// Characters besides lower-case letters and digits that a URL scheme may contain.
pub const SCHEME_EXTRA_CHARS: [char; 3] = ['+', '-', '.'];

/// Tables of the project file that are never read: a scanned repository must not switch on
/// running its own build scripts (`level2`, ADR 0033, decision 9), allow other URL schemes for
/// "open in editor" (`editor`), point the server at a folder (`ui`) (ADR 0030, decisions 7, 12) or
/// lift the size limit of the HTML export (`export`, ADR 0090).
pub const UNTRUSTED_PROJECT_TABLES: [&str; 4] = ["level2", "editor", "ui", "export"];

/// URL schemes that run script or open local files in a browser. They are refused in
/// `editor.allowed_schemes` in every layer (ADR 0030).
pub const FORBIDDEN_EDITOR_SCHEMES: [&str; 4] = ["javascript", "data", "file", "vbscript"];

/// Table that holds the scan limits.
pub const SCAN_TABLE: &str = "scan";
/// Settings of [`SCAN_TABLE`] that the project file may only lower: a scanned repository must not
/// lift the protection of the machine that scans it (#54). The user file and the flags may still
/// raise them.
pub const LOWER_ONLY_SCAN_KEYS: [&str; 4] = [
    "max_source_mb",
    "parse_timeout_ms",
    "max_candidates",
    "max_memory_mb",
];

/// Header of a table in an array of tables, as it starts a layer in `.codeskimr.toml`.
pub const LAYER_TABLE_HEADER: &str = "[[layers]]";
/// Header of an entry of the exception list of the unused symbols.
pub const UNUSED_TABLE_HEADER: &str = "[[unused.ignore]]";
/// What may follow `[rules` or `[[rules` in a header: the end of the name or a sub-table.
pub const RULES_NAME_END: [char; 2] = [']', '.'];
/// Start of the header of the table `rules` and of its sub-tables, such as `[rules]`.
pub const RULES_TABLE_HEADER: &str = "[rules";
/// Start of the header of an array of tables below `rules`, such as `[[rules.forbid]]`.
pub const RULES_ARRAY_HEADER: &str = "[[rules";
/// First character of every table header line.
pub const TABLE_HEADER_START: char = '[';
/// Separates the kept part of the project file from the written layers.
pub const BLOCK_SEPARATOR: &str = "\n\n";
/// Ends a line.
pub const LINE_END: char = '\n';
/// Ends a line in a file written on Windows.
pub const WINDOWS_LINE_END: &str = "\r\n";

/// Words of a key name that mark a value as secret (a word is a part between `.` or `_`). Such a
/// value is never shown by [`super::EffectiveSetting`]: the configuration holds no secret today,
/// and this keeps one that is added later from reaching a screen.
pub const SECRET_KEY_WORDS: [&str; 9] = [
    "token",
    "secret",
    "password",
    "passwd",
    "credential",
    "credentials",
    "private",
    "key",
    "apikey",
];

/// Separator of the words inside a key name.
pub const WORD_SEPARATOR: char = '_';

/// Hyphen, which also separates the words of a key name.
pub const HYPHEN: char = '-';
