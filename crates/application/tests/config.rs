//! Tests of the configuration: defaults, override chain and validation (blueprint section 14).

use codeskimr_application::config::{
    Config, ConfigError, ConfigLayer, CycleRule, LayerSource, Violation,
};
use codeskimr_codes::{Coded, ErrorCode};

fn user(text: &str) -> ConfigLayer {
    ConfigLayer::parse(LayerSource::User, text).unwrap_or_else(|e| unreachable!("{e:?}"))
}

fn project(text: &str) -> ConfigLayer {
    ConfigLayer::parse(LayerSource::Project, text).unwrap_or_else(|e| unreachable!("{e:?}"))
}

#[test]
fn the_budget_of_loading_the_language_packs_is_in_the_defaults() {
    let config = Config::defaults().expect("shipped defaults are valid");
    assert_eq!(config.budgets.pack_load_ms, 1500);
}

#[test]
fn the_languages_below_99_percent_are_marked_in_the_defaults() {
    let config = Config::defaults().expect("shipped defaults are valid");
    assert_eq!(
        config.ui.approximate_languages,
        ["python", "csharp", "kotlin"]
    );
}

#[test]
fn level_one_gives_up_a_bare_name_above_one_hundred_candidates() {
    let config = Config::defaults().expect("shipped defaults are valid");
    assert_eq!(config.scan.max_candidates, 100);
}

#[test]
fn a_scan_names_twenty_files_it_left_out_by_default() {
    let config = Config::defaults().expect("shipped defaults are valid");
    assert_eq!(config.scan.max_left_out_listed, 20);
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
fn defaults_use_all_cores_and_limit_shared_fingerprints() {
    let config = Config::defaults().expect("shipped defaults are valid");
    assert_eq!(config.scan.max_workers, 0);
    assert_eq!(config.conventions.duplicate_max_owners, 50);
}

#[test]
fn level2_defaults_are_256_mib_and_half() {
    let config = Config::defaults().expect("shipped defaults are valid");
    assert_eq!(config.level2.max_index_bytes, 256 * 1024 * 1024);
    assert!((config.level2.min_mapped - 0.5).abs() < f64::EPSILON);
    assert_eq!(config.level2.max_created_symbols, 100_000);
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
fn the_longest_wait_of_the_watcher_is_not_shorter_than_its_quiet_window() {
    assert_rejected(&[(
        "[watcher]\ndebounce_ms = 500\nmax_wait_ms = 100\n",
        "watcher.max_wait_ms",
        Violation::MustNotBeBelow("watcher.debounce_ms"),
    )]);
}

#[test]
fn annotation_limits_are_checked() {
    assert_rejected(&[
        (
            "[annotations]\ndismissed_retention_days = 0\n",
            "annotations.dismissed_retention_days",
            Violation::MustBePositive,
        ),
        (
            "[annotations]\nmax_text_chars = 0\n",
            "annotations.max_text_chars",
            Violation::MustBePositive,
        ),
        (
            "[annotations]\nmax_title_chars = 0\n",
            "annotations.max_title_chars",
            Violation::MustBePositive,
        ),
        (
            "[annotations]\nmax_title_chars = 5000\n",
            "annotations.max_title_chars",
            Violation::MustNotExceed("annotations.max_text_chars"),
        ),
        (
            "[annotations]\ndefault_page_size = 0\n",
            "annotations.default_page_size",
            Violation::MustBePositive,
        ),
        (
            "[annotations]\ndefault_page_size = 600\n",
            "annotations.default_page_size",
            Violation::MustNotExceed("annotations.max_page_size"),
        ),
    ]);
    let limits = Config::defaults().unwrap().annotations.limits();
    assert_eq!((limits.max_text_chars, limits.max_title_chars), (4000, 500));
    let pages = Config::defaults().unwrap().annotations;
    assert_eq!((pages.default_page_size, pages.max_page_size), (100, 500));
}

#[test]
fn the_limits_of_assignee_name_and_parameters_are_checked() {
    assert_rejected(&[
        (
            "[annotations]\nmax_assignee_chars = 0\n",
            "annotations.max_assignee_chars",
            Violation::MustBePositive,
        ),
        (
            "[annotations]\nmax_name_chars = 0\n",
            "annotations.max_name_chars",
            Violation::MustBePositive,
        ),
        (
            "[annotations]\nmax_parameter_chars = 0\n",
            "annotations.max_parameter_chars",
            Violation::MustBePositive,
        ),
        (
            "[annotations]\nmax_parameters = 0\n",
            "annotations.max_parameters",
            Violation::MustBePositive,
        ),
    ]);
    let defaults = Config::defaults().unwrap().annotations;
    assert_eq!(
        (
            defaults.max_assignee_chars,
            defaults.max_name_chars,
            defaults.max_parameter_chars,
            defaults.max_parameters
        ),
        (100, 500, 200, 32)
    );
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
fn the_limit_of_shared_fingerprints_is_checked() {
    assert_rejected(&[(
        "[conventions]\nduplicate_max_owners = 0\n",
        "conventions.duplicate_max_owners",
        Violation::MustBePositive,
    )]);
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
            "[level2]\nmax_created_symbols = 0\n",
            "level2.max_created_symbols",
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
fn an_empty_language_id_in_the_approximate_list_is_rejected() {
    assert_rejected(&[(
        "[ui]\napproximate_languages = [\"python\", \" \"]\n",
        "ui.approximate_languages",
        Violation::MustNotBeEmpty,
    )]);
}

#[test]
fn the_polling_of_the_switch_and_the_pause_of_the_window_are_checked() {
    assert_rejected(&[
        (
            "[ui]\nlevel2_poll_seconds = 0\n",
            "ui.level2_poll_seconds",
            Violation::MustBePositive,
        ),
        (
            "[ui]\nlevel2_poll_seconds = 10\nlevel2_poll_max_seconds = 5\n",
            "ui.level2_poll_max_seconds",
            Violation::MustNotBeBelow("ui.level2_poll_seconds"),
        ),
        (
            "[level2]\nconfirm_window_cooldown_seconds = 0\n",
            "level2.confirm_window_cooldown_seconds",
            Violation::MustBePositive,
        ),
        (
            "[level2]\nmax_parallel_runs = 0\n",
            "level2.max_parallel_runs",
            Violation::MustBePositive,
        ),
        (
            "[level2]\nmax_parallel_runs = 9\n",
            "level2.max_parallel_runs",
            Violation::MustNotPass(8),
        ),
        (
            "[level2]\nui_cooldown_seconds = 9\n",
            "level2.ui_cooldown_seconds",
            Violation::MustReach(10),
        ),
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
        (
            "[graph]\nrow_boxes = 0\n",
            "graph.row_boxes",
            Violation::MustBePositive,
        ),
        (
            "[graph]\nrisk_max = 0\n",
            "graph.risk_max",
            Violation::MustBePositive,
        ),
        (
            "[graph]\nmax_modules = 0\n",
            "graph.max_modules",
            Violation::MustBePositive,
        ),
    ]);
}

#[test]
fn the_export_limit_must_be_positive() {
    assert_rejected(&[(
        "[export]\nmax_references = 0\n",
        "export.max_references",
        Violation::MustBePositive,
    )]);
}

#[test]
fn the_comparison_limits_must_be_positive() {
    assert_rejected(&[
        (
            "[compare]\nmax_symbols = 0\n",
            "compare.max_symbols",
            Violation::MustBePositive,
        ),
        (
            "[compare]\nmax_usages = 0\n",
            "compare.max_usages",
            Violation::MustBePositive,
        ),
    ]);
}

#[test]
fn the_comparison_limits_have_defaults() {
    let config = Config::resolve(&[]).expect("valid");
    assert_eq!(config.compare.max_symbols, 200);
    assert_eq!(config.compare.max_usages, 50);
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
        (
            "[[layers]]\nname = \"a\"\nnamespaces = [\"x\"]\nmay_use = [\"missing\"]\n",
            "layers",
            Violation::MustBeLayerRule,
        ),
    ]);
}

#[test]
fn a_layer_may_limit_its_dependencies_to_declared_layers() {
    let text = "[[layers]]\nname = \"a\"\nnamespaces = [\"x\"]\nmay_use = [\"b\"]\n\n[[layers]]\nname = \"b\"\nnamespaces = [\"y\"]\nmay_use = []\n\n[[layers]]\nname = \"c\"\nnamespaces = [\"z\"]\n";
    let layers = Config::resolve(&[project(text)])
        .expect("valid")
        .project_layers();
    assert_eq!(layers[0].may_use.as_ref().map(Vec::len), Some(1));
    assert_eq!(layers[1].may_use, Some(Vec::new()));
    assert_eq!(layers[2].may_use, None);
}

#[test]
fn the_project_file_may_lower_the_scan_limits_but_not_raise_them() {
    let defaults = Config::defaults().expect("defaults").scan;
    let lowered = project("[scan]\nmax_source_mb = 10\nparse_timeout_ms = 100\n");
    let scan = Config::resolve(&[lowered]).expect("valid").scan;
    assert_eq!((scan.max_source_mb, scan.parse_timeout_ms), (10, 100));
    let raised = project("[scan]\nmax_source_mb = 999999\nparse_timeout_ms = 999999\n");
    let scan = Config::resolve(&[raised]).expect("valid").scan;
    assert_eq!(scan.max_source_mb, defaults.max_source_mb);
    assert_eq!(scan.parse_timeout_ms, defaults.parse_timeout_ms);
}

#[test]
fn the_project_file_may_lower_the_memory_limit_but_not_raise_it() {
    let defaults = Config::defaults().expect("defaults").scan;
    assert_eq!(defaults.max_memory_mb, 16384);
    let lowered = project("[scan]\nmax_memory_mb = 2048\n");
    let scan = Config::resolve(&[lowered]).expect("valid").scan;
    assert_eq!(scan.max_memory_mb, 2048);
    let raised = project("[scan]\nmax_memory_mb = 4000000000\n");
    let scan = Config::resolve(&[raised]).expect("valid").scan;
    assert_eq!(scan.max_memory_mb, defaults.max_memory_mb);
    let user_file = user("[scan]\nmax_memory_mb = 32768\n");
    let scan = Config::resolve(&[user_file]).expect("valid").scan;
    assert_eq!(scan.max_memory_mb, 32768);
}

#[test]
fn the_project_file_may_lower_the_candidate_limit_but_not_raise_it() {
    let defaults = Config::defaults().expect("defaults").scan;
    let lowered = project("[scan]\nmax_candidates = 10\n");
    let scan = Config::resolve(&[lowered]).expect("valid").scan;
    assert_eq!(scan.max_candidates, 10);
    let raised = project("[scan]\nmax_candidates = 4000000000\n");
    let scan = Config::resolve(&[raised]).expect("valid").scan;
    assert_eq!(scan.max_candidates, defaults.max_candidates);
    let user_file = user("[scan]\nmax_candidates = 500\n");
    let scan = Config::resolve(&[user_file]).expect("valid").scan;
    assert_eq!(scan.max_candidates, 500);
}

#[test]
fn the_user_file_and_flags_can_raise_the_scan_limits_and_the_project_stays_below_them() {
    let user_file = user("[scan]\nmax_source_mb = 4096\n");
    let raised_by_project = project("[scan]\nmax_source_mb = 8192\n");
    let scan = Config::resolve(&[user_file.clone(), raised_by_project])
        .expect("valid")
        .scan;
    assert_eq!(
        scan.max_source_mb, 4096,
        "the project may not pass the user's value"
    );
    let flag = ConfigLayer::from_overrides([("scan.max_source_mb", "8192")]).expect("flag");
    let scan = Config::resolve(&[user_file, flag]).expect("valid").scan;
    assert_eq!(scan.max_source_mb, 8192, "a flag may raise it");
}

#[test]
fn the_project_file_cannot_lift_the_export_limit() {
    let raised = project("[export]\nmax_references = 999999999\n");
    let config =
        Config::resolve(&[user("[export]\nmax_references = 1000\n"), raised]).expect("valid");
    assert_eq!(config.export.max_references, 1000);
    let config = Config::resolve(&[project("[export]\nmax_references = 5\n")]).expect("valid");
    assert_eq!(
        config.export.max_references, 600_000,
        "the table is not read at all"
    );
}

#[test]
fn the_signature_limit_must_be_positive() {
    assert_rejected(&[(
        "[scan]\nmax_signature_chars = 0\n",
        "scan.max_signature_chars",
        Violation::MustBePositive,
    )]);
}

#[test]
fn a_project_file_cannot_change_the_editor_or_the_ui() {
    let text = "[editor]\nurl_template = \"evil://{path}\"\nallowed_schemes = [\"evil\"]\n[ui]\ndir = \"/tmp/elsewhere\"\nlaunch_code_seconds = 3600\n";
    let from_project = Config::resolve(&[project(text)]).expect("the tables are dropped");
    let defaults = Config::defaults().expect("defaults");
    assert_eq!(
        from_project.editor.url_template,
        defaults.editor.url_template
    );
    assert_eq!(
        from_project.editor.allowed_schemes,
        defaults.editor.allowed_schemes
    );
    assert_eq!(from_project.ui.dir, defaults.ui.dir);
    assert_eq!(
        from_project.ui.launch_code_seconds,
        defaults.ui.launch_code_seconds
    );
    let from_user = Config::resolve(&[user("[ui]\ndir = \"/tmp/elsewhere\"\n")]).expect("valid");
    assert_eq!(from_user.ui.dir, "/tmp/elsewhere");
}

#[test]
fn schemes_that_run_code_or_read_files_are_never_editor_schemes() {
    assert_rejected(&[
        (
            "[editor]\nallowed_schemes = [\"vscode\", \"javascript\"]\n",
            "editor.allowed_schemes",
            Violation::MustNotRunCode,
        ),
        (
            "[editor]\nallowed_schemes = [\"file\"]\nurl_template = \"file://{path}\"\n",
            "editor.allowed_schemes",
            Violation::MustNotRunCode,
        ),
        (
            "[editor]\nallowed_schemes = [\"vbscript\", \"data\"]\n",
            "editor.allowed_schemes",
            Violation::MustNotRunCode,
        ),
    ]);
}

#[test]
fn the_project_file_keeps_an_exception_list_for_unused_symbols() {
    let defaults = Config::defaults().expect("defaults");
    assert_eq!(defaults.unused.ignore.len(), 0);
    let text = "[[unused.ignore]]\npaths = [\"Data/Migrations/**\"]\nreason = \"called by the migration tool\"\n\n[[unused.ignore]]\nsymbols = [\"*.DesignTimeFactory\"]\npaths = [\"Data/*\"]\nreason = \"only for dotnet ef\"\n";
    let config = Config::resolve(&[project(text)]).expect("valid list");
    let [first, second] = config.unused.ignore.as_slice() else {
        panic!("two entries");
    };
    assert!(first.covers(
        "Data/Migrations/20260101_Init.cs",
        "Data.Migrations.Init.Up"
    ));
    assert!(!first.covers("Api/Startup.cs", "Api.Startup.ConfigureServices"));
    assert!(second.covers("Data/DesignTimeFactory.cs", "Data.DesignTimeFactory"));
    assert!(!second.covers("Api/DesignTimeFactory.cs", "Api.DesignTimeFactory"));
}

#[test]
fn an_exception_needs_a_reason_and_a_pattern() {
    assert_rejected(&[
        (
            "[[unused.ignore]]\npaths = [\"a/**\"]\nreason = \"\"\n",
            "unused.ignore",
            Violation::MustBeUnusedIgnore,
        ),
        (
            "[[unused.ignore]]\nreason = \"why\"\n",
            "unused.ignore",
            Violation::MustBeUnusedIgnore,
        ),
        (
            "[[unused.ignore]]\nsymbols = [\" \"]\nreason = \"why\"\n",
            "unused.ignore",
            Violation::MustBeUnusedIgnore,
        ),
    ]);
}

#[test]
fn the_project_file_holds_architecture_rules() {
    let defaults = Config::defaults().expect("defaults");
    assert_eq!(defaults.rules.cycles, CycleRule::Off);
    assert_eq!(defaults.rules.forbid.len(), 0);
    let text = "[rules]\ncycles = \"deny\"\n\n[[rules.forbid]]\nfrom = \"A.Reports\"\nto = \"A.Checkout\"\nreason = \"split\"\n\n[[rules.public_api]]\nmodule = \"A.Data\"\ninternal = [\"A.Data.Internal\"]\n";
    let config = Config::resolve(&[project(text)]).expect("valid rules");
    assert_eq!(config.rules.cycles, CycleRule::Deny);
    assert_eq!(config.rules.forbid[0].reason.as_deref(), Some("split"));
    assert_eq!(config.rules.public_api[0].internal, ["A.Data.Internal"]);
}

#[test]
fn an_architecture_rule_needs_valid_prefixes_that_make_sense() {
    let bad = [
        "[[rules.forbid]]\nfrom = \"A\"\nto = \"A.B\"\n",
        "[[rules.forbid]]\nfrom = \"A.B\"\nto = \"A.B\"\n",
        "[[rules.forbid]]\nfrom = \"\"\nto = \"A\"\n",
        "[[rules.public_api]]\nmodule = \"A\"\ninternal = []\n",
        "[[rules.public_api]]\nmodule = \"A\"\ninternal = [\"A\"]\n",
        "[[rules.public_api]]\nmodule = \"A\"\ninternal = [\"B.Inner\"]\n",
        "[[rules.forbid]]\nfrom = \"A\"\nto = \"B\"\n\n[[rules.forbid]]\nfrom = \"A\"\nto = \"B\"\n",
        "[[rules.public_api]]\nmodule = \"A\"\ninternal = [\"A.X\"]\n\n[[rules.public_api]]\nmodule = \"A\"\ninternal = [\"A.Y\"]\n",
    ];
    let cases: Vec<_> = bad
        .iter()
        .map(|text| (*text, "rules", Violation::MustBeArchitectureRule))
        .collect();
    assert_rejected(&cases);
    assert!(Config::resolve(&[project("[rules]\ncycles = \"sometimes\"\n")]).is_err());
}
