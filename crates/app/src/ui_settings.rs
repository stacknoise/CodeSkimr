//! The settings the UI saves in the data folder (ADR 0103, ADR 0118): the memory limit per indexer
//! and the numbers and switches of the tables that `UI_EDIT_*_TABLES` name.
//!
//! They sit in one small file beside the preferences. The configuration reads the file as a layer
//! between the user file and the project file (`LayerSource::Ui`), so a value saved here wins over
//! the user file and loses against a `--set` flag. The file is read again at every start; a file
//! that is missing, too large or not valid is the same as no settings, and a name or value that
//! the configuration would refuse is dropped, so that no file can keep the program from starting.
//! The commands that write the file are in `ui_commands`.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use codeskimr_application::config::{Config, ConfigLayer, LayerSource, MAX_INDEXER_MEMORY_MB};
use serde::{Deserialize, Serialize};

use crate::consts::{
    UI_EDIT_LOCKED_KEYS, UI_EDIT_SCAN_TABLES, UI_EDIT_START_TABLES, UI_KEY_SEPARATOR,
    UI_LAYER_ASSIGN, UI_LAYER_HEADER, UI_LAYER_NEWLINE, UI_LAYER_TABLE, UI_SETTINGS_FILE,
    UI_SETTINGS_MAX_BYTES,
};

/// A value the UI saves for a setting: a switch or a number.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UiValue {
    /// A switch.
    Bool(bool),
    /// A whole number.
    Int(i64),
    /// A number with a fraction.
    Float(f64),
}

/// The kind of value a setting holds, from its default.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Bool,
    Int,
    Float,
}

/// When a saved value counts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Applies {
    /// Every scan, check and export reads the files again, so it counts from the next one.
    Scan,
    /// The running program read the value when it started, so it counts from the next start.
    Start,
}

impl Kind {
    /// The kind of a default written as TOML text; `None` for text, lists and tables.
    fn of(text: &str) -> Option<Self> {
        if text.parse::<bool>().is_ok() {
            Some(Self::Bool)
        } else if text.parse::<i64>().is_ok() {
            Some(Self::Int)
        } else if text.parse::<f64>().is_ok_and(f64::is_finite) {
            Some(Self::Float)
        } else {
            None
        }
    }

    /// The value in the form this kind keeps, or `None` if it does not fit.
    fn fit(self, value: UiValue) -> Option<UiValue> {
        match (self, value) {
            (Self::Bool, UiValue::Bool(_)) | (Self::Int, UiValue::Int(_)) => Some(value),
            (Self::Float, UiValue::Float(number)) => number.is_finite().then_some(value),
            #[allow(clippy::cast_precision_loss)]
            (Self::Float, UiValue::Int(number)) => Some(UiValue::Float(number as f64)),
            _ => None,
        }
    }
}

impl UiValue {
    /// The value as TOML text.
    fn toml_text(self) -> String {
        match self {
            Self::Bool(flag) => flag.to_string(),
            Self::Int(number) => number.to_string(),
            Self::Float(number) => format!("{number:?}"),
        }
    }
}

/// How the UI may change one setting: the kind of its value and when the change counts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Editable {
    kind: Kind,
    /// When a saved value counts.
    pub applies: Applies,
}

/// Whether the UI may save a value for `key`, and how. Only numbers and switches of the tables the
/// constants name; everything else (the switch of level 2, the port, the editor link, folders and
/// texts) stays with the configuration file.
#[must_use]
pub fn editable(key: &str) -> Option<Editable> {
    if UI_EDIT_LOCKED_KEYS.contains(&key) {
        return None;
    }
    let table = key.split(UI_KEY_SEPARATOR).next()?;
    let applies = if UI_EDIT_SCAN_TABLES.contains(&table) {
        Applies::Scan
    } else if UI_EDIT_START_TABLES.contains(&table) {
        Applies::Start
    } else {
        return None;
    };
    let default = Config::effective(&[])
        .ok()?
        .into_iter()
        .find(|setting| setting.key == key)?;
    Some(Editable {
        kind: Kind::of(&default.value)?,
        applies,
    })
}

impl Editable {
    /// The value in the form of this setting, or `None` if its kind does not fit.
    #[must_use]
    pub fn fit(self, value: UiValue) -> Option<UiValue> {
        self.kind.fit(value)
    }
}

/// What the UI has saved: the memory limit in megabytes per indexer, by the name of the indexer
/// (`0` means the indexer follows `level2.max_memory_mb`), and the other values by the dotted key
/// of the setting. An entry that is not in a map has no saved value.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct UiSettings {
    /// Memory limit per indexer in megabytes.
    pub indexer_memory_mb: BTreeMap<String, u32>,
    /// Saved values of the settings the UI may change, by dotted key.
    pub values: BTreeMap<String, UiValue>,
}

/// The names of the indexers the configuration knows, from its defaults.
pub fn known_indexers() -> Vec<String> {
    Config::defaults()
        .map(|config| config.level2.indexer_memory_mb.into_keys().collect())
        .unwrap_or_default()
}

impl UiSettings {
    /// Keeps only what the configuration would accept: a known indexer and a value in range.
    fn sanitized(mut self) -> Self {
        let known = known_indexers();
        self.indexer_memory_mb
            .retain(|name, megabytes| known.contains(name) && *megabytes <= MAX_INDEXER_MEMORY_MB);
        self.values = std::mem::take(&mut self.values)
            .into_iter()
            .filter_map(|(key, value)| {
                let fitted = editable(&key)?.fit(value)?;
                Some((key, fitted))
            })
            .collect();
        self
    }

    /// Whether nothing is saved.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.indexer_memory_mb.is_empty() && self.values.is_empty()
    }

    /// The text of the configuration layer these settings make.
    fn layer_text(&self) -> String {
        let memory = self
            .indexer_memory_mb
            .iter()
            .map(|(name, megabytes)| format!("{name}{UI_LAYER_ASSIGN}{megabytes}"));
        let mut tables: BTreeMap<&str, Vec<String>> = BTreeMap::new();
        for (key, value) in &self.values {
            if let Some((table, name)) = key.rsplit_once(UI_KEY_SEPARATOR) {
                tables
                    .entry(table)
                    .or_default()
                    .push(format!("{name}{UI_LAYER_ASSIGN}{}", value.toml_text()));
            }
        }
        let mut lines = vec![UI_LAYER_HEADER.to_owned()];
        lines.extend(memory);
        for (table, entries) in tables {
            lines.push(format!("{}{table}{}", UI_LAYER_TABLE.0, UI_LAYER_TABLE.1));
            lines.extend(entries);
        }
        lines.join(UI_LAYER_NEWLINE)
    }

    /// The configuration layer these settings make; `None` if nothing is saved or the
    /// configuration would not read it.
    #[must_use]
    pub fn to_layer(&self) -> Option<ConfigLayer> {
        if self.is_empty() {
            return None;
        }
        ConfigLayer::parse(LayerSource::Ui, &self.layer_text()).ok()
    }

    /// The text that is written to the file; `None` if the settings cannot be turned into text.
    pub fn to_file_text(&self) -> Option<String> {
        serde_json::to_string(self).ok()
    }
}

/// The file of the saved settings in the data folder.
#[derive(Debug, Clone)]
pub struct UiSettingsFile {
    path: PathBuf,
}

impl UiSettingsFile {
    /// The file of the installation whose data folder is `data_dir`.
    #[must_use]
    pub fn in_dir(data_dir: &Path) -> Self {
        Self {
            path: data_dir.join(UI_SETTINGS_FILE),
        }
    }

    /// Where the file is.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The saved settings; none if there is no usable file.
    #[must_use]
    pub fn read(&self) -> UiSettings {
        let usable = fs::symlink_metadata(&self.path)
            .is_ok_and(|meta| meta.is_file() && meta.len() <= UI_SETTINGS_MAX_BYTES);
        usable
            .then(|| fs::read_to_string(&self.path).ok())
            .flatten()
            .and_then(|text| serde_json::from_str::<UiSettings>(&text).ok())
            .map(UiSettings::sanitized)
            .unwrap_or_default()
    }

    /// The configuration layer of the saved settings; none if nothing is saved.
    #[must_use]
    pub fn layer(&self) -> Option<ConfigLayer> {
        self.read().to_layer()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn saved(entries: &[(&str, u32)]) -> UiSettings {
        UiSettings {
            indexer_memory_mb: entries
                .iter()
                .map(|(name, megabytes)| ((*name).to_owned(), *megabytes))
                .collect(),
            values: BTreeMap::new(),
        }
    }

    fn with_values(entries: &[(&str, UiValue)]) -> UiSettings {
        UiSettings {
            values: entries
                .iter()
                .map(|(key, value)| ((*key).to_owned(), *value))
                .collect(),
            ..UiSettings::default()
        }
    }

    #[test]
    fn nothing_stored_means_no_settings_and_no_layer() {
        let dir = tempfile::tempdir().unwrap();
        let file = UiSettingsFile::in_dir(dir.path());
        assert_eq!(file.read(), UiSettings::default());
        assert!(file.layer().is_none());
    }

    #[test]
    fn what_is_stored_makes_a_layer_the_configuration_accepts() {
        let dir = tempfile::tempdir().unwrap();
        let file = UiSettingsFile::in_dir(dir.path());
        let settings = saved(&[("scip-typescript", 8192), ("scip-java", 0)]);
        fs::write(file.path(), settings.to_file_text().unwrap()).unwrap();
        assert_eq!(file.read(), settings);
        let layer = file.layer().unwrap();
        assert_eq!(layer.source(), LayerSource::Ui);
        let config = Config::resolve(&[layer]).unwrap();
        assert_eq!(config.level2.indexer_memory_mb["scip-typescript"], 8192);
    }

    #[test]
    fn a_damaged_foreign_or_out_of_range_file_never_reaches_the_configuration() {
        let dir = tempfile::tempdir().unwrap();
        let file = UiSettingsFile::in_dir(dir.path());
        for content in [
            "not json",
            "{\"other\":1}",
            "{\"indexer_memory_mb\":{\"made-up\":5}}",
            "{\"indexer_memory_mb\":{\"scip-java\":4294967295}}",
            "{\"indexer_memory_mb\":{\"scip-java\":-1}}",
        ] {
            fs::write(file.path(), content).unwrap();
            assert_eq!(file.read(), UiSettings::default(), "{content}");
            assert!(file.layer().is_none(), "{content}");
        }
        fs::write(file.path(), "x".repeat(10_000)).unwrap();
        assert_eq!(file.read(), UiSettings::default());
    }

    #[test]
    fn one_good_value_survives_next_to_a_bad_one() {
        let dir = tempfile::tempdir().unwrap();
        let file = UiSettingsFile::in_dir(dir.path());
        fs::write(
            file.path(),
            "{\"indexer_memory_mb\":{\"scip-java\":2048,\"made-up\":5}}",
        )
        .unwrap();
        assert_eq!(file.read(), saved(&[("scip-java", 2048)]));
    }

    #[test]
    fn numbers_and_switches_of_the_open_tables_make_a_layer() {
        let settings = with_values(&[
            ("scan.max_workers", UiValue::Int(4)),
            ("metrics.instability_stable_max", UiValue::Float(0.4)),
            ("conventions.max_parameters", UiValue::Int(6)),
        ]);
        let config = Config::resolve(&[settings.to_layer().unwrap()]).unwrap();
        assert_eq!(config.scan.max_workers, 4);
        assert_eq!(config.conventions.max_parameters, 6);
    }

    #[test]
    fn a_whole_number_for_a_fractional_setting_is_kept_as_a_fraction() {
        let dir = tempfile::tempdir().unwrap();
        let file = UiSettingsFile::in_dir(dir.path());
        let settings = with_values(&[("metrics.instability_stable_max", UiValue::Int(1))]);
        fs::write(file.path(), settings.to_file_text().unwrap()).unwrap();
        assert_eq!(
            file.read().values["metrics.instability_stable_max"],
            UiValue::Float(1.0)
        );
        assert!(file.layer().is_some());
    }

    #[test]
    fn only_the_open_tables_and_matching_kinds_are_kept() {
        let dir = tempfile::tempdir().unwrap();
        let file = UiSettingsFile::in_dir(dir.path());
        let settings = with_values(&[
            ("level2.enabled", UiValue::Bool(true)),
            ("server.port", UiValue::Int(1)),
            ("scan.excluded_folders", UiValue::Int(1)),
            ("scan.max_workers", UiValue::Bool(true)),
            ("scan.nothing_like_it", UiValue::Int(1)),
            ("scan.parse_timeout_ms", UiValue::Int(8000)),
        ]);
        fs::write(file.path(), settings.to_file_text().unwrap()).unwrap();
        assert_eq!(
            file.read(),
            with_values(&[("scan.parse_timeout_ms", UiValue::Int(8000))])
        );
    }

    #[test]
    fn a_setting_says_when_a_saved_value_counts() {
        assert_eq!(editable("scan.max_workers").unwrap().applies, Applies::Scan);
        assert_eq!(editable("mcp.max_limit").unwrap().applies, Applies::Start);
        assert!(editable("level2.enabled").is_none());
        assert!(editable("editor.url_template").is_none());
        assert!(editable("ui.default_theme").is_none());
    }
}
