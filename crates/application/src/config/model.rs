//! The typed configuration. There is no string-keyed access anywhere (blueprint section 14).

use codeskimr_domain::{
    AnnotationLimits, FingerprintParams, Layer, LayerName, LayoutParams, ModelError, NodeSize,
    QualifiedName,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::time::Duration;

use crate::consts::BYTES_PER_MIB;
use crate::ports::IndexLimits;

/// The complete, validated configuration. Built by [`crate::config::Config::resolve`].
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The architecture layers of the scanned project, in order (ADR 0031). Empty means none.
    pub layers: Vec<LayerRule>,
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
    /// Limits of the convention checks.
    pub conventions: Conventions,
    /// Scan behaviour.
    pub scan: Scan,
    /// Level 2 references from SCIP indexes.
    pub level2: Level2,
    /// The local server.
    pub server: Server,
    /// The link that opens a source position in the editor.
    pub editor: Editor,
    /// The web UI.
    pub ui: Ui,
    /// The picture of the dependency map.
    pub graph: Graph,
    /// The HTML export (ADR 0090).
    pub export: Export,
    /// Limits of annotations and alerts (ADR 0068).
    pub annotations: Annotations,
    /// Where `find_constants` looks for configuration and translation keys.
    pub keys: KeySources,
}

/// The folders of a project whose files hold keys that `find_constants` lists (blueprint 10.4):
/// the keys of TOML files below `config_folders` and the message ids of Fluent files below
/// `i18n_folders`. Folders are relative to the project root; one that is not there is skipped.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeySources {
    /// Folders with `.toml` files.
    pub config_folders: Vec<String>,
    /// Folders with `.ftl` files.
    pub i18n_folders: Vec<String>,
    /// Most files read per call; the rest is not looked at.
    pub max_files: u32,
    /// Largest file that is read, in kilobytes; larger ones are skipped.
    pub max_file_kb: u32,
    /// Folder levels below a configured folder that are looked into.
    pub max_depth: u32,
}

/// One architecture layer as written in `.codeskimr.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LayerRule {
    /// Name of the layer, unique in the project.
    pub name: String,
    /// Qualified-name prefixes that belong to the layer; the first layer that owns a name wins.
    pub namespaces: Vec<String>,
    /// Names of the layers this layer may depend on (ADR 0054). Missing means no limit; an empty
    /// list means none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub may_use: Option<Vec<String>>,
}

impl LayerRule {
    /// The domain layer, or `None` if the name, a namespace or an allowed layer is not valid.
    #[must_use]
    pub fn to_layer(&self) -> Option<Layer> {
        let namespaces: Result<Vec<_>, _> =
            self.namespaces.iter().map(QualifiedName::new).collect();
        let may_use = match &self.may_use {
            Some(names) => Some(
                names
                    .iter()
                    .map(LayerName::new)
                    .collect::<Result<Vec<_>, _>>()
                    .ok()?,
            ),
            None => None,
        };
        Some(Layer {
            name: LayerName::new(&self.name).ok()?,
            namespaces: namespaces.ok()?,
            may_use,
        })
    }

    /// Whether every allowed layer is one of the declared `names`.
    pub(super) fn uses_only(&self, names: &HashSet<&str>) -> bool {
        self.may_use
            .iter()
            .flatten()
            .all(|used| names.contains(used.as_str()))
    }
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
    /// Loading the Rust and C# packs on a first scan, in milliseconds.
    pub pack_load_ms: u32,
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
    /// Similarity in percent from which `find_similar_code` lists a function, when the caller
    /// gives none.
    pub default_similarity_percent: u32,
    /// Largest snippet `find_similar_code` takes, in bytes.
    pub max_snippet_bytes: u32,
    /// Entries in each list of the answer of `get_context` for the budget `short`.
    pub context_items_short: u32,
    /// Entries in each list of the answer of `get_context` for the budget `medium`.
    pub context_items_medium: u32,
    /// Entries in each list of the answer of `get_context` for the budget `long`.
    pub context_items_long: u32,
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

/// Limits of the convention checks (blueprint 3, `check_conventions`).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Conventions {
    /// Longest allowed function in code lines.
    pub max_function_lines: u32,
    /// Highest allowed cognitive complexity of a function (ADR 0048).
    pub max_cognitive_complexity: u32,
    /// Most parameters a function may have (ADR 0048).
    pub max_parameters: u32,
    /// Fingerprint similarity in percent from which two functions count as duplicates.
    pub duplicate_similarity_percent: u32,
    /// Fewest code lines a function needs to be compared for duplicates; shorter ones are too
    /// small for a copy to mean anything (measured on this repository: 5 lines give 21 findings, 8 give 9).
    pub duplicate_min_lines: u32,
    /// Most functions that may share a fingerprint for it to make them candidates for the duplicate
    /// check; a fingerprint that more functions share is boilerplate and costs the square of that
    /// number.
    pub duplicate_max_owners: u32,
    /// Folder names whose files are tests; no convention check looks at them.
    pub test_folders: Vec<String>,
}

/// Scan behaviour.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scan {
    /// Folder names that are never scanned, at any depth.
    pub excluded_folders: Vec<String>,
    /// Time limit for parsing one file, in milliseconds.
    pub parse_timeout_ms: u32,
    /// Most threads that analyse files at the same time; 0 means one per core.
    pub max_workers: u32,
    /// Largest total size of the scanned files of one source, in megabytes.
    pub max_source_mb: u32,
    /// Most characters of a signature (for a constant: its declaration with the value); a longer
    /// one is cut and ends with `…`.
    pub max_signature_chars: u32,
    /// Most symbols a place may reach through the bare name; more and level 1 gives the place up
    /// (`RES-304`, ADR 0084).
    pub max_candidates: u32,
    /// Most memory a scan may add to what the program held when it started, in megabytes; more
    /// and the scan is stopped (`PRJ-503`, ADR 0087).
    pub max_memory_mb: u32,
    /// Most files a scan names when it left them out; the others are only counted.
    pub max_left_out_listed: u32,
}

/// Settings and limits of level 2: running indexers and importing their indexes (ADR 0033).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Level2 {
    /// Master switch for running indexers. Only the user config and command line flags can set
    /// it; the project file cannot (ADR 0033, decision 9).
    pub enabled: bool,
    /// Longest time one indexer may run, in seconds.
    pub timeout_seconds: u32,
    /// Most memory one indexer with its child processes may hold, in megabytes.
    pub max_memory_mb: u32,
    /// Most bytes one indexer may write in total (console and output directory).
    pub max_output_bytes: u64,
    /// Largest index file that is read, in bytes.
    pub max_index_bytes: u64,
    /// Smallest share `mapped / (mapped + unmapped)` among the uses of project symbols (symbols
    /// the index defines itself; library symbols are not counted), from 0 to 1.
    pub min_mapped: f64,
    /// Most symbols one index may make level 2 create (ADR 0055); an index that wants more fails
    /// as a whole.
    pub max_created_symbols: u32,
    /// Pause in seconds after the console window of the confirmation was opened, during which no
    /// other window is opened for any project (ADR 0071, decision 4a).
    pub confirm_window_cooldown_seconds: u32,
    /// Most level 2 runs at the same time over all projects (ADR 0072).
    pub max_parallel_runs: u32,
    /// Pause in seconds after a run that the browser started has ended, before the same project
    /// may start another one from the browser (ADR 0072).
    pub ui_cooldown_seconds: u32,
}

/// The local server (blueprint 10.3).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Server {
    /// TCP port on the loopback address.
    pub port: u16,
}

/// The editor link of the UI (blueprint 15.4, ADR 0028).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Editor {
    /// URL template with the placeholders `{path}` and `{line}`.
    pub url_template: String,
    /// The schemes a template may use.
    pub allowed_schemes: Vec<String>,
}

/// The web UI (blueprint 15.3, ADR 0028).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ui {
    /// Name of the theme used until the user chooses one.
    pub default_theme: String,
    /// Folder with the built UI.
    pub dir: String,
    /// How long a one-time launch code is valid, in seconds.
    pub launch_code_seconds: u32,
    /// How often the projects screen asks for the level 2 state while the confirmation window is
    /// open, in seconds (ADR 0071, decision 3).
    pub level2_poll_seconds: u32,
    /// How long the projects screen keeps asking, in seconds.
    pub level2_poll_max_seconds: u32,
    /// Ids of the languages whose usages the screens mark as an approximation.
    pub approximate_languages: Vec<String>,
}

/// Limits of annotations and alerts (blueprint 14, ADR 0068).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Annotations {
    /// Days a dismissed alert is kept before it is purged.
    pub dismissed_retention_days: u32,
    /// Most characters of the text of an annotation.
    pub max_text_chars: u32,
    /// Most characters of the title of a task.
    pub max_title_chars: u32,
    /// Most characters of the assignee of a task.
    pub max_assignee_chars: u32,
    /// Most characters of the qualified name of the symbol an annotation is put on.
    pub max_name_chars: u32,
    /// Most characters of one parameter type of that symbol.
    pub max_parameter_chars: u32,
    /// Most parameter types of that symbol.
    pub max_parameters: u32,
    /// Rows of a list of annotations or alerts when the caller gives no limit.
    pub default_page_size: u32,
    /// Most rows of one page of such a list.
    pub max_page_size: u32,
}

impl Annotations {
    /// The limits for the texts of annotations.
    #[must_use]
    pub const fn limits(&self) -> AnnotationLimits {
        AnnotationLimits {
            max_text_chars: self.max_text_chars,
            max_title_chars: self.max_title_chars,
        }
    }
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

/// The HTML export of a project (blueprint 15, ADR 0090).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Export {
    /// Most references a project may have to be exported. The page holds every reference once
    /// more per direction, so a larger project gives a file of hundreds of megabytes that no
    /// browser opens; the export is refused with `EXP-502` instead.
    pub max_references: u32,
}

/// The picture of the dependency map (blueprint 15, screen 3), in layout units.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Graph {
    /// Width of the box of one layer.
    pub node_width: u32,
    /// Height of the box of one layer.
    pub node_height: u32,
    /// Vertical space between two rows.
    pub rank_gap: u32,
    /// Horizontal space between two boxes of a row.
    pub node_gap: u32,
    /// Most sweeps against crossing edges.
    pub sweeps: u32,
    /// Number of passes that move the boxes towards their neighbours.
    pub placement_passes: u32,
    /// Most boxes in one line of a row; a longer row wraps into several lines, so that a project
    /// with hundreds of namespaces stays readable.
    pub row_boxes: u32,
}

impl Graph {
    /// The size of one box.
    #[must_use]
    pub const fn node_size(&self) -> NodeSize {
        NodeSize {
            width: self.node_width,
            height: self.node_height,
        }
    }

    /// The spacing and effort of the layout.
    #[must_use]
    pub const fn layout_params(&self) -> LayoutParams {
        LayoutParams {
            rank_gap: self.rank_gap,
            node_gap: self.node_gap,
            sweeps: self.sweeps,
            placement_passes: self.placement_passes,
        }
    }
}

impl Level2 {
    /// The limits of one indexer run.
    #[must_use]
    pub fn limits(&self) -> IndexLimits {
        IndexLimits {
            timeout: Duration::from_secs(u64::from(self.timeout_seconds)),
            max_memory_bytes: u64::from(self.max_memory_mb).saturating_mul(BYTES_PER_MIB),
            max_output_bytes: self.max_output_bytes,
        }
    }
}
