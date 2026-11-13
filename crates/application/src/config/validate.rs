//! Range checks of the typed configuration (blueprint section 14: validated at start-up).

use super::consts::{PATH_SEPARATORS, PERCENT_MAX, UNIT_MAX, UNIT_MIN, keys};
use super::error::{ConfigError, Violation};
use super::model::{Budgets, Config, McpLimits, MetricThresholds, Scan};

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
    positive(keys::NGRAM_SIZE, config.similarity.ngram_size)?;
    positive(keys::WINDOW_SIZE, config.similarity.window_size)?;
    positive(keys::MIN_TOKENS, config.similarity.min_tokens)?;
    positive(keys::SERVER_PORT, u32::from(config.server.port))?;
    check_scan(&config.scan)
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
    Ok(())
}

fn check_scan(scan: &Scan) -> Result<(), ConfigError> {
    positive(keys::PARSE_TIMEOUT_MS, scan.parse_timeout_ms)?;
    positive(keys::MAX_SOURCE_MB, scan.max_source_mb)?;
    if scan.excluded_folders.is_empty() {
        return fail(keys::EXCLUDED_FOLDERS, Violation::MustNotBeEmpty);
    }
    let is_bad = |name: &String| name.is_empty() || name.contains(PATH_SEPARATORS);
    if scan.excluded_folders.iter().any(is_bad) {
        return fail(keys::EXCLUDED_FOLDERS, Violation::MustBeFolderName);
    }
    Ok(())
}
