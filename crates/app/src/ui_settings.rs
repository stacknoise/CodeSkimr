//! The settings the UI saves in the data folder (ADR 0103): today the memory limit per indexer.
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
    UI_LAYER_ASSIGN, UI_LAYER_HEADER, UI_LAYER_NEWLINE, UI_SETTINGS_FILE, UI_SETTINGS_MAX_BYTES,
};

/// What the UI has saved: the memory limit in megabytes per indexer, by the name of the indexer
/// (`0` means the indexer follows `level2.max_memory_mb`). An indexer that is not in the map has
/// no saved value.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct UiSettings {
    /// Memory limit per indexer in megabytes.
    pub indexer_memory_mb: BTreeMap<String, u32>,
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
        self
    }

    /// The text of the configuration layer these settings make.
    fn layer_text(&self) -> String {
        let lines = self
            .indexer_memory_mb
            .iter()
            .map(|(name, megabytes)| format!("{name}{UI_LAYER_ASSIGN}{megabytes}"));
        std::iter::once(UI_LAYER_HEADER.to_owned())
            .chain(lines)
            .collect::<Vec<_>>()
            .join(UI_LAYER_NEWLINE)
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
        let settings = self.read();
        if settings.indexer_memory_mb.is_empty() {
            return None;
        }
        ConfigLayer::parse(LayerSource::Ui, &settings.layer_text()).ok()
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
}
