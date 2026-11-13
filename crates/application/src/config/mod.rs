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
    Budgets, Config, McpLimits, MetricThresholds, Scan, Similarity, SnapshotRetention, Watcher,
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
            if let Some(key) = layer::find_unknown_key(layer.table(), &defaults, NO_PREFIX) {
                return Err(ConfigError::UnknownKey {
                    source_layer: layer.source(),
                    key,
                });
            }
            layer::deep_merge(&mut merged, layer.table());
        }
        let config: Self = Value::Table(merged)
            .try_into()
            .map_err(|cause| ConfigError::WrongType { cause })?;
        validate::validate(&config)?;
        Ok(config)
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
