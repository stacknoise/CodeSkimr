//! Typed, validated configuration with the override chain of blueprint section 14.
//!
//! `config/default.toml` is embedded in the binary and holds every tunable value.
//! Overrides are applied in this order: user config file, project config
//! (`.codeskimr.toml`), command line flags. Reading the files is the job of the caller
//! (composition root); this module only turns text into a validated [`Config`].

mod consts;
mod error;
mod layer;
mod model;
mod validate;

pub use consts::PROJECT_CONFIG_FILE;
pub use error::{ConfigError, LayerSource, Violation};
pub use layer::ConfigLayer;
pub use model::{
    Budgets, Config, Conventions, Editor, Graph, LayerRule, Level2, McpLimits, MetricThresholds,
    Scan, Similarity, SnapshotRetention, Ui, Watcher,
};

use consts::NO_PREFIX;
use toml::{Table, Value};

impl Config {
    /// The built-in defaults from `config/default.toml`.
    ///
    /// # Errors
    /// Any `ConfigError`; with the shipped file this never happens (a test guards it).
    pub fn defaults() -> Result<Self, ConfigError> {
        Self::resolve(&[])
    }

    /// Applies `layers` in the given order on top of the defaults and validates the result.
    /// Pass them as user file, project file, command line, so later layers win.
    ///
    /// # Errors
    /// - `ConfigError::ParseFailed` (CFG-102) if the embedded defaults are not valid TOML.
    /// - `ConfigError::UnknownKey` (CFG-103) if a layer sets a key that does not exist.
    /// - `ConfigError::WrongType` or `ConfigError::OutOfRange` (CFG-101) for invalid values.
    pub fn resolve(layers: &[ConfigLayer]) -> Result<Self, ConfigError> {
        let defaults = parse_defaults()?;
        let mut merged = defaults.clone();
        for layer in layers {
            let table = trusted_table(layer, &merged);
            if let Some(key) = layer::find_unknown_key(&table, &defaults, NO_PREFIX) {
                return Err(ConfigError::UnknownKey {
                    source_layer: layer.source(),
                    key,
                });
            }
            layer::deep_merge(&mut merged, &table);
        }
        let config: Self = Value::Table(merged)
            .try_into()
            .map_err(|cause| ConfigError::WrongType { cause })?;
        validate::validate(&config)?;
        Ok(config)
    }
}

impl Config {
    /// The layers of the project as the domain model has them. Entries that do not form a valid
    /// layer are left out; [`Config::resolve`] has rejected them already.
    #[must_use]
    pub fn project_layers(&self) -> Vec<codeskimr_domain::Layer> {
        self.layers.iter().filter_map(LayerRule::to_layer).collect()
    }
}

/// What of a layer counts. The project file may not touch `level2` (ADR 0033, decision 9): a
/// scanned repository must not switch on running its own build scripts or raise the limits, so the
/// table is dropped before anything is checked or merged. The scan limits of the project file
/// only count as far as they lower what `merged` (defaults and user file) already holds (#54).
fn trusted_table(layer: &ConfigLayer, merged: &Table) -> Table {
    let mut table = layer.table().clone();
    if layer.source() == LayerSource::Project {
        table.remove(consts::UNTRUSTED_PROJECT_TABLE);
        lower_only(&mut table, merged);
    }
    table
}

/// Takes every scan limit out of `table` that is not lower than the one in `merged`.
fn lower_only(table: &mut Table, merged: &Table) {
    let Some(Value::Table(scan)) = table.get_mut(consts::SCAN_TABLE) else {
        return;
    };
    let Some(Value::Table(current)) = merged.get(consts::SCAN_TABLE) else {
        return;
    };
    for key in consts::LOWER_ONLY_SCAN_KEYS {
        let raised = matches!(
            (scan.get(key), current.get(key)),
            (Some(Value::Integer(wanted)), Some(Value::Integer(now))) if wanted > now
        );
        if raised {
            scan.remove(key);
        }
    }
}

fn parse_defaults() -> Result<Table, ConfigError> {
    consts::DEFAULT_CONFIG
        .parse::<Table>()
        .map_err(|cause| ConfigError::ParseFailed {
            source_layer: LayerSource::Default,
            cause,
        })
}
