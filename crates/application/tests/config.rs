//! Tests of the configuration: defaults, override chain and validation (blueprint section 14).

use codeskimr_application::config::{Config, ConfigError, ConfigLayer, LayerSource, Violation};
use codeskimr_codes::{Coded, ErrorCode};

fn user(text: &str) -> ConfigLayer {
    ConfigLayer::parse(LayerSource::User, text).unwrap_or_else(|e| unreachable!("{e:?}"))
}

fn project(text: &str) -> ConfigLayer {
    ConfigLayer::parse(LayerSource::Project, text).unwrap_or_else(|e| unreachable!("{e:?}"))
}

#[test]
fn defaults_match_the_blueprint() {
    let config = Config::defaults().expect("shipped defaults are valid");
    assert_eq!(config.metrics.cyclomatic_good_max, 5);
    assert_eq!(config.metrics.cyclomatic_medium_max, 10);
    assert_eq!(config.metrics.maintainability_good_min, 55);
    assert_eq!(config.metrics.maintainability_medium_min, 35);
    assert!((config.metrics.instability_stable_max - 0.5).abs() < f64::EPSILON);
    assert!((config.metrics.instability_medium_max - 0.8).abs() < f64::EPSILON);
    assert_eq!(config.budgets.cold_start_ms, 300);
    assert_eq!(config.budgets.full_scan_100k_loc_ms, 1000);
    for folder in ["bin", "obj", ".git", "node_modules", "target", "dist"] {
        assert!(config.scan.excluded_folders.iter().any(|f| f == folder));
    }
    assert_eq!(config.scan.parse_timeout_ms, 5000);
    assert_eq!(config.scan.max_source_mb, 2048);
}

#[test]
fn later_layers_win_and_untouched_values_stay() {
    let layers = [
        user("[metrics]\ncyclomatic_good_max = 4\ncyclomatic_medium_max = 8\n"),
        project("[metrics]\ncyclomatic_good_max = 3\n"),
        ConfigLayer::from_overrides([("mcp.default_limit", "7")]).expect("valid flag"),
    ];
    let config = Config::resolve(&layers).expect("valid chain");
    assert_eq!(config.metrics.cyclomatic_good_max, 3);
    assert_eq!(config.metrics.cyclomatic_medium_max, 8);
    assert_eq!(config.mcp.default_limit, 7);
    assert_eq!(
        config.watcher.debounce_ms,
        Config::defaults().expect("defaults").watcher.debounce_ms
    );
}

#[test]
fn lists_are_replaced_not_appended() {
    let layer = project("[scan]\nexcluded_folders = [\"out\"]\n");
    let config = Config::resolve(&[layer]).expect("valid");
    assert_eq!(config.scan.excluded_folders, ["out"]);
}

#[test]
fn flags_can_set_lists_and_nested_keys() {
    let layer = ConfigLayer::from_overrides([("scan.excluded_folders", "[\"x\", \"y\"]")])
        .expect("valid flag");
    assert_eq!(layer.source(), LayerSource::Cli);
    let config = Config::resolve(&[layer]).expect("valid");
    assert_eq!(config.scan.excluded_folders, ["x", "y"]);
}

#[test]
fn invalid_toml_is_cfg_102() {
    let error = ConfigLayer::parse(LayerSource::Project, "[metrics\n").expect_err("broken");
    assert_eq!(error.code(), ErrorCode::ConfigParseFailed);
    assert!(matches!(
        error,
        ConfigError::ParseFailed {
            source_layer: LayerSource::Project,
            ..
        }
    ));
    let flag = ConfigLayer::from_overrides([("mcp.max_limit", "not a value")]).expect_err("broken");
    assert_eq!(flag.code(), ErrorCode::ConfigParseFailed);
}

#[test]
fn unknown_keys_are_cfg_103_with_their_path() {
    let top = Config::resolve(&[user("typo = 1\n")]).expect_err("unknown");
    assert_eq!(top.code(), ErrorCode::ConfigUnknownKey);
    let nested = Config::resolve(&[project("[watcher]\ndebounce = 5\n")]).expect_err("unknown");
    let ConfigError::UnknownKey { source_layer, key } = nested else {
        unreachable!("expected an unknown key error");
    };
    assert_eq!(source_layer, LayerSource::Project);
    assert_eq!(key, "watcher.debounce");
}

#[test]
fn wrong_types_are_cfg_101() {
    let error = Config::resolve(&[user("[watcher]\ndebounce_ms = \"fast\"\n")]).expect_err("type");
    assert_eq!(error.code(), ErrorCode::ConfigInvalidValue);
    assert!(matches!(error, ConfigError::WrongType { .. }));
    let negative = Config::resolve(&[user("[mcp]\nmax_limit = -1\n")]).expect_err("type");
    assert_eq!(negative.code(), ErrorCode::ConfigInvalidValue);
}

fn out_of_range(text: &str) -> Option<(&'static str, Violation)> {
    match Config::resolve(&[user(text)]) {
        Err(ConfigError::OutOfRange { key, violation }) => Some((key, violation)),
        _ => None,
    }
}

fn assert_rejected(cases: &[(&str, &'static str, Violation)]) {
    for (text, key, violation) in cases {
        assert_eq!(out_of_range(text), Some((*key, *violation)), "{text}");
    }
}

#[test]
fn counts_and_percentages_are_checked() {
    assert_rejected(&[
        (
            "[watcher]\ndebounce_ms = 0\n",
            "watcher.debounce_ms",
            Violation::MustBePositive,
        ),
        (
            "[similarity]\nngram_size = 0\n",
            "similarity.ngram_size",
            Violation::MustBePositive,
        ),
        (
            "[similarity]\nwindow_size = 0\n",
            "similarity.window_size",
            Violation::MustBePositive,
        ),
        (
            "[snapshots]\nretention_count = 0\n",
            "snapshots.retention_count",
            Violation::MustBePositive,
        ),
        (
            "[budgets]\nidle_cpu_percent = 101\n",
            "budgets.idle_cpu_percent",
            Violation::MustBePercent,
        ),
        (
            "[budgets]\ncold_start_ms = 0\n",
            "budgets.cold_start_ms",
            Violation::MustBePositive,
        ),
    ]);
}

#[test]
fn metric_thresholds_are_checked() {
    assert_rejected(&[
        (
            "[metrics]\ncyclomatic_good_max = 10\n",
            "metrics.cyclomatic_good_max",
            Violation::MustBeBelow("metrics.cyclomatic_medium_max"),
        ),
        (
            "[metrics]\nmaintainability_good_min = 30\n",
            "metrics.maintainability_good_min",
            Violation::MustBeAbove("metrics.maintainability_medium_min"),
        ),
        (
            "[metrics]\ninstability_medium_max = 1.5\n",
            "metrics.instability_medium_max",
            Violation::MustBeShare,
        ),
        (
            "[metrics]\ninstability_stable_max = nan\n",
            "metrics.instability_stable_max",
            Violation::MustBeShare,
        ),
        (
            "[metrics]\ninstability_stable_max = 0.9\n",
            "metrics.instability_stable_max",
            Violation::MustNotExceed("metrics.instability_medium_max"),
        ),
    ]);
}

#[test]
fn limits_and_folders_are_checked() {
    assert_rejected(&[
        (
            "[mcp]\ndefault_limit = 500\n",
            "mcp.default_limit",
            Violation::MustNotExceed("mcp.max_limit"),
        ),
        (
            "[scan]\nexcluded_folders = []\n",
            "scan.excluded_folders",
            Violation::MustNotBeEmpty,
        ),
        (
            "[scan]\nexcluded_folders = [\"a/b\"]\n",
            "scan.excluded_folders",
            Violation::MustBeFolderName,
        ),
        (
            "[scan]\nexcluded_folders = [\"\"]\n",
            "scan.excluded_folders",
            Violation::MustBeFolderName,
        ),
        (
            "[scan]\nparse_timeout_ms = 0\n",
            "scan.parse_timeout_ms",
            Violation::MustBePositive,
        ),
        (
            "[scan]\nmax_source_mb = 0\n",
            "scan.max_source_mb",
            Violation::MustBePositive,
        ),
    ]);
}

#[test]
fn every_error_maps_to_a_registered_code() {
    let error = Config::resolve(&[user("[watcher]\ndebounce_ms = 0\n")]).expect_err("range");
    assert_eq!(error.code().id(), "CFG-101");
    assert_eq!(
        error.to_string(),
        ErrorCode::ConfigInvalidValue.description()
    );
}
