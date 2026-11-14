//! Range checks of the typed configuration (blueprint section 14: validated at start-up).

use std::collections::HashSet;

use super::consts::{
    PATH_SEPARATORS, PERCENT_MAX, PLACEHOLDER_PATH, SCHEME_EXTRA_CHARS, UNIT_MAX, UNIT_MIN,
    URL_SCHEME_SEPARATOR, keys,
};
use super::error::{ConfigError, Violation};
use super::model::{
    Budgets, Config, Editor, KeySources, Level2, McpLimits, MetricThresholds, Scan, Ui,
};

fn check_keys(sources: &KeySources) -> Result<(), ConfigError> {
    positive(keys::KEYS_MAX_FILES, sources.max_files)?;
    positive(keys::KEYS_MAX_FILE_KB, sources.max_file_kb)
}

/// Checks every value of `config`.
///
/// # Errors
/// `ConfigError::OutOfRange` (CFG-101) naming the first offending key.
pub(super) fn validate(config: &Config) -> Result<(), ConfigError> {
    check_metrics(&config.metrics)?;
    check_budgets(&config.budgets)?;
    check_mcp(&config.mcp)?;
    positive(keys::RETENTION_COUNT, config.snapshots.retention_count)?;
    positive(keys::DEBOUNCE_MS, config.watcher.debounce_ms)?;
    positive(keys::MAX_WAIT_MS, config.watcher.max_wait_ms)?;
    if config.watcher.max_wait_ms < config.watcher.debounce_ms {
        return fail(
            keys::MAX_WAIT_MS,
            Violation::MustNotBeBelow(keys::DEBOUNCE_MS),
        );
    }
    positive(keys::NGRAM_SIZE, config.similarity.ngram_size)?;
    positive(keys::WINDOW_SIZE, config.similarity.window_size)?;
    positive(keys::MIN_TOKENS, config.similarity.min_tokens)?;
    positive(keys::SERVER_PORT, u32::from(config.server.port))?;
    positive(
        keys::MAX_FUNCTION_LINES,
        config.conventions.max_function_lines,
    )?;
    positive(
        keys::DUPLICATE_SIMILARITY_PERCENT,
        config.conventions.duplicate_similarity_percent,
    )?;
    positive(
        keys::DUPLICATE_MIN_LINES,
        config.conventions.duplicate_min_lines,
    )?;
    percent(
        keys::DUPLICATE_SIMILARITY_PERCENT,
        config.conventions.duplicate_similarity_percent,
    )?;
    check_keys(&config.keys)?;
    check_layers(config)?;
    check_scan(&config.scan)?;
    check_level2(&config.level2)?;
    check_editor(&config.editor)?;
    check_ui(&config.ui)?;
    positive(keys::GRAPH_NODE_WIDTH, config.graph.node_width)?;
    positive(keys::GRAPH_NODE_HEIGHT, config.graph.node_height)
}

fn check_layers(config: &Config) -> Result<(), ConfigError> {
    let mut seen = HashSet::new();
    let valid = config.layers.iter().all(|rule| {
        !rule.namespaces.is_empty() && rule.to_layer().is_some() && seen.insert(rule.name.as_str())
    });
    if valid {
        Ok(())
    } else {
        fail(keys::LAYERS, Violation::MustBeLayerRule)
    }
}

fn fail(key: &'static str, violation: Violation) -> Result<(), ConfigError> {
    Err(ConfigError::OutOfRange { key, violation })
}

fn positive(key: &'static str, value: u32) -> Result<(), ConfigError> {
    if value == 0 {
        fail(key, Violation::MustBePositive)
    } else {
        Ok(())
    }
}

fn percent(key: &'static str, value: u32) -> Result<(), ConfigError> {
    if value > PERCENT_MAX {
        fail(key, Violation::MustBePercent)
    } else {
        Ok(())
    }
}

fn share(key: &'static str, value: f64) -> Result<(), ConfigError> {
    if (UNIT_MIN..=UNIT_MAX).contains(&value) {
        Ok(())
    } else {
        fail(key, Violation::MustBeShare)
    }
}

fn check_metrics(metrics: &MetricThresholds) -> Result<(), ConfigError> {
    positive(keys::CYCLOMATIC_GOOD_MAX, metrics.cyclomatic_good_max)?;
    if metrics.cyclomatic_good_max >= metrics.cyclomatic_medium_max {
        return fail(
            keys::CYCLOMATIC_GOOD_MAX,
            Violation::MustBeBelow(keys::CYCLOMATIC_MEDIUM_MAX),
        );
    }
    percent(
        keys::MAINTAINABILITY_GOOD_MIN,
        metrics.maintainability_good_min,
    )?;
    positive(
        keys::MAINTAINABILITY_MEDIUM_MIN,
        metrics.maintainability_medium_min,
    )?;
    if metrics.maintainability_good_min <= metrics.maintainability_medium_min {
        return fail(
            keys::MAINTAINABILITY_GOOD_MIN,
            Violation::MustBeAbove(keys::MAINTAINABILITY_MEDIUM_MIN),
        );
    }
    share(keys::INSTABILITY_STABLE_MAX, metrics.instability_stable_max)?;
    share(keys::INSTABILITY_MEDIUM_MAX, metrics.instability_medium_max)?;
    if metrics.instability_stable_max > metrics.instability_medium_max {
        return fail(
            keys::INSTABILITY_STABLE_MAX,
            Violation::MustNotExceed(keys::INSTABILITY_MEDIUM_MAX),
        );
    }
    Ok(())
}

fn check_budgets(budgets: &Budgets) -> Result<(), ConfigError> {
    let timings = [
        (keys::COLD_START_MS, budgets.cold_start_ms),
        (keys::SEARCH_KEYSTROKE_MS, budgets.search_keystroke_ms),
        (keys::EXPLORER_SWITCH_MS, budgets.explorer_switch_ms),
        (keys::USAGES_QUERY_MS, budgets.usages_query_ms),
        (keys::GRAPH_LAYOUT_MS, budgets.graph_layout_ms),
        (keys::FULL_SCAN_MS, budgets.full_scan_100k_loc_ms),
        (keys::INCREMENTAL_RESCAN_MS, budgets.incremental_rescan_ms),
        (keys::MEMORY_MB, budgets.memory_1m_loc_mb),
        (keys::BINARY_SIZE_MB, budgets.binary_size_mb),
    ];
    for (key, value) in timings {
        positive(key, value)?;
    }
    percent(keys::IDLE_CPU_PERCENT, budgets.idle_cpu_percent)?;
    percent(
        keys::REGRESSION_TOLERANCE_PERCENT,
        budgets.regression_tolerance_percent,
    )
}

fn check_mcp(mcp: &McpLimits) -> Result<(), ConfigError> {
    positive(keys::MCP_DEFAULT_LIMIT, mcp.default_limit)?;
    if mcp.default_limit > mcp.max_limit {
        return fail(
            keys::MCP_DEFAULT_LIMIT,
            Violation::MustNotExceed(keys::MCP_MAX_LIMIT),
        );
    }
    positive(keys::MCP_DEFAULT_DEPTH, mcp.default_depth)?;
    if mcp.default_depth > mcp.max_depth {
        return fail(
            keys::MCP_DEFAULT_DEPTH,
            Violation::MustNotExceed(keys::MCP_MAX_DEPTH),
        );
    }
    positive(
        keys::MCP_DEFAULT_SIMILARITY_PERCENT,
        mcp.default_similarity_percent,
    )?;
    percent(
        keys::MCP_DEFAULT_SIMILARITY_PERCENT,
        mcp.default_similarity_percent,
    )?;
    positive(keys::MCP_MAX_SNIPPET_BYTES, mcp.max_snippet_bytes)
}

fn check_level2(level2: &Level2) -> Result<(), ConfigError> {
    if level2.max_index_bytes == 0 {
        return fail(keys::LEVEL2_MAX_INDEX_BYTES, Violation::MustBePositive);
    }
    positive(keys::LEVEL2_TIMEOUT_SECONDS, level2.timeout_seconds)?;
    positive(keys::LEVEL2_MAX_MEMORY_MB, level2.max_memory_mb)?;
    if level2.max_output_bytes < level2.max_index_bytes {
        return fail(
            keys::LEVEL2_MAX_OUTPUT_BYTES,
            Violation::MustNotBeBelow(keys::LEVEL2_MAX_INDEX_BYTES),
        );
    }
    share(keys::LEVEL2_MIN_MAPPED, level2.min_mapped)
}

fn check_scan(scan: &Scan) -> Result<(), ConfigError> {
    positive(keys::PARSE_TIMEOUT_MS, scan.parse_timeout_ms)?;
    positive(keys::MAX_SOURCE_MB, scan.max_source_mb)?;
    positive(keys::MAX_SIGNATURE_CHARS, scan.max_signature_chars)?;
    if scan.excluded_folders.is_empty() {
        return fail(keys::EXCLUDED_FOLDERS, Violation::MustNotBeEmpty);
    }
    let is_bad = |name: &String| name.is_empty() || name.contains(PATH_SEPARATORS);
    if scan.excluded_folders.iter().any(is_bad) {
        return fail(keys::EXCLUDED_FOLDERS, Violation::MustBeFolderName);
    }
    Ok(())
}

/// A URL scheme in the form the allow-list needs: a lower-case letter, then lower-case letters,
/// digits, `+`, `-` or `.` (RFC 3986, lower case so that the comparison is plain).
fn is_scheme(text: &str) -> bool {
    let mut chars = text.chars();
    chars.next().is_some_and(|first| first.is_ascii_lowercase())
        && chars.all(|c| {
            c.is_ascii_lowercase() || c.is_ascii_digit() || SCHEME_EXTRA_CHARS.contains(&c)
        })
}

fn check_editor(editor: &Editor) -> Result<(), ConfigError> {
    if editor.allowed_schemes.is_empty() {
        return fail(keys::EDITOR_ALLOWED_SCHEMES, Violation::MustNotBeEmpty);
    }
    if !editor.allowed_schemes.iter().all(|name| is_scheme(name)) {
        return fail(keys::EDITOR_ALLOWED_SCHEMES, Violation::MustBeUrlScheme);
    }
    let scheme = editor
        .url_template
        .split_once(URL_SCHEME_SEPARATOR)
        .map(|(scheme, _)| scheme.to_ascii_lowercase());
    let allowed = scheme
        .as_ref()
        .is_some_and(|scheme| editor.allowed_schemes.contains(scheme));
    if !allowed {
        return fail(
            keys::EDITOR_URL_TEMPLATE,
            Violation::MustUseAllowedScheme(keys::EDITOR_ALLOWED_SCHEMES),
        );
    }
    if !editor.url_template.contains(PLACEHOLDER_PATH) {
        return fail(keys::EDITOR_URL_TEMPLATE, Violation::MustContainPath);
    }
    Ok(())
}

fn check_ui(ui: &Ui) -> Result<(), ConfigError> {
    positive(keys::UI_LAUNCH_CODE_SECONDS, ui.launch_code_seconds)?;
    let is_bad = |name: &str| name.is_empty() || name.contains(PATH_SEPARATORS);
    if is_bad(&ui.default_theme) {
        return fail(keys::UI_DEFAULT_THEME, Violation::MustBeFolderName);
    }
    if ui.dir.is_empty() {
        return fail(keys::UI_DIR, Violation::MustNotBeEmpty);
    }
    Ok(())
}
