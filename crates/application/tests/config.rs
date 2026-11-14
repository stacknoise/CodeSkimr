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
    assert_eq!(config.scan.max_signature_chars, 400);
    assert_eq!(config.server.port, 7421);
}

#[test]
fn level2_defaults_are_256_mib_and_half() {
    let config = Config::defaults().expect("shipped defaults are valid");
    assert_eq!(config.level2.max_index_bytes, 256 * 1024 * 1024);
    assert!((config.level2.min_mapped - 0.5).abs() < f64::EPSILON);
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
fn the_default_depth_must_not_exceed_the_maximum() {
    assert_rejected(&[(
        "[mcp]\ndefault_depth = 9\n",
        "mcp.default_depth",
        Violation::MustNotExceed("mcp.max_depth"),
    )]);
}

#[test]
fn the_similar_code_settings_are_checked() {
    assert_rejected(&[
        (
            "[mcp]\ndefault_similarity_percent = 0\n",
            "mcp.default_similarity_percent",
            Violation::MustBePositive,
        ),
        (
            "[mcp]\ndefault_similarity_percent = 101\n",
            "mcp.default_similarity_percent",
            Violation::MustBePercent,
        ),
        (
            "[mcp]\nmax_snippet_bytes = 0\n",
            "mcp.max_snippet_bytes",
            Violation::MustBePositive,
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
        (
            "[server]\nport = 0\n",
            "server.port",
            Violation::MustBePositive,
        ),
    ]);
}

#[test]
fn level2_limits_are_checked() {
    assert_rejected(&[
        (
            "[level2]\nmax_index_bytes = 0\n",
            "level2.max_index_bytes",
            Violation::MustBePositive,
        ),
        (
            "[level2]\nmin_mapped = 1.5\n",
            "level2.min_mapped",
            Violation::MustBeShare,
        ),
    ]);
}

#[test]
fn the_editor_link_and_the_ui_are_checked() {
    assert_rejected(&[
        (
            "[editor]\nurl_template = \"javascript:alert({path})\"\n",
            "editor.url_template",
            Violation::MustUseAllowedScheme("editor.allowed_schemes"),
        ),
        (
            "[editor]\nurl_template = \"no-scheme-{path}\"\n",
            "editor.url_template",
            Violation::MustUseAllowedScheme("editor.allowed_schemes"),
        ),
        (
            "[editor]\nurl_template = \"vscode://file/x\"\n",
            "editor.url_template",
            Violation::MustContainPath,
        ),
        (
            "[editor]\nallowed_schemes = []\n",
            "editor.allowed_schemes",
            Violation::MustNotBeEmpty,
        ),
        (
            "[editor]\nallowed_schemes = [\"VSCode\"]\n",
            "editor.allowed_schemes",
            Violation::MustBeUrlScheme,
        ),
        (
            "[ui]\nlaunch_code_seconds = 0\n",
            "ui.launch_code_seconds",
            Violation::MustBePositive,
        ),
        (
            "[ui]\ndefault_theme = \"\"\n",
            "ui.default_theme",
            Violation::MustBeFolderName,
        ),
        ("[ui]\ndir = \"\"\n", "ui.dir", Violation::MustNotBeEmpty),
    ]);
}

#[test]
fn the_size_of_a_graph_box_must_be_positive() {
    assert_rejected(&[
        (
            "[graph]\nnode_width = 0\n",
            "graph.node_width",
            Violation::MustBePositive,
        ),
        (
            "[graph]\nnode_height = 0\n",
            "graph.node_height",
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

#[test]
fn layers_are_empty_by_default_and_come_from_the_project_file() {
    assert_eq!(Config::defaults().expect("defaults").layers.len(), 0);
    let text = "[[layers]]\nname = \"domain\"\nnamespaces = [\"crates::domain\"]\n\n[[layers]]\nname = \"app\"\nnamespaces = [\"crates::app\", \"crates::cli\"]\n";
    let config = Config::resolve(&[project(text)]).expect("valid layers");
    let layers = config.project_layers();
    let names: Vec<&str> = layers.iter().map(|layer| layer.name.as_str()).collect();
    assert_eq!(names, ["domain", "app"]);
    assert!(layers[1].owns("crates::cli::src::main"));
    assert!(!layers[1].owns("crates::application"));
}

#[test]
fn a_layer_needs_a_unique_name_and_a_namespace() {
    assert_rejected(&[
        (
            "[[layers]]\nname = \"a\"\nnamespaces = []\n",
            "layers",
            Violation::MustBeLayerRule,
        ),
        (
            "[[layers]]\nname = \"a\"\nnamespaces = [\"x\"]\n[[layers]]\nname = \"a\"\nnamespaces = [\"y\"]\n",
            "layers",
            Violation::MustBeLayerRule,
        ),
        (
            "[[layers]]\nname = \"\"\nnamespaces = [\"x\"]\n",
            "layers",
            Violation::MustBeLayerRule,
        ),
    ]);
}

#[test]
fn the_signature_limit_must_be_positive() {
    assert_rejected(&[(
        "[scan]\nmax_signature_chars = 0\n",
        "scan.max_signature_chars",
        Violation::MustBePositive,
    )]);
}
