//! One layer of the override chain: user file, project file or command line flags.

use toml::{Table, Value};

use super::consts::{KEY_SEPARATOR, OVERRIDE_VALUE_KEY};
use super::error::{ConfigError, LayerSource};

/// A parsed, not yet validated set of overrides.
#[derive(Debug, Clone, PartialEq)]
pub struct ConfigLayer {
    source: LayerSource,
    table: Table,
}

impl ConfigLayer {
    /// Parses the text of a config file as one layer.
    ///
    /// # Errors
    /// `ConfigError::ParseFailed` (CFG-102) when the text is not valid TOML.
    pub fn parse(source: LayerSource, text: &str) -> Result<Self, ConfigError> {
        let table = text
            .parse::<Table>()
            .map_err(|cause| ConfigError::ParseFailed {
                source_layer: source,
                cause,
            })?;
        Ok(Self { source, table })
    }

    /// Builds the command line layer from `(dotted key, raw TOML value)` pairs,
    /// for example `("metrics.cyclomatic_good_max", "4")`.
    ///
    /// # Errors
    /// `ConfigError::ParseFailed` (CFG-102) when a raw value is not a valid TOML value.
    pub fn from_overrides<'a>(
        pairs: impl IntoIterator<Item = (&'a str, &'a str)>,
    ) -> Result<Self, ConfigError> {
        let mut table = Table::new();
        for (key, raw) in pairs {
            insert_path(&mut table, key, parse_raw_value(raw)?);
        }
        Ok(Self {
            source: LayerSource::Cli,
            table,
        })
    }

    /// The layer this table belongs to.
    #[must_use]
    pub const fn source(&self) -> LayerSource {
        self.source
    }

    pub(super) const fn table(&self) -> &Table {
        &self.table
    }
}

/// Parses a bare TOML value by giving it a key to live under.
fn parse_raw_value(raw: &str) -> Result<Value, ConfigError> {
    let wrapped = format!("{OVERRIDE_VALUE_KEY} = {raw}");
    let mut table = wrapped
        .parse::<Table>()
        .map_err(|cause| ConfigError::ParseFailed {
            source_layer: LayerSource::Cli,
            cause,
        })?;
    Ok(table
        .remove(OVERRIDE_VALUE_KEY)
        .unwrap_or(Value::Table(Table::new())))
}

/// Stores `value` below a dotted key path, creating the tables on the way.
fn insert_path(table: &mut Table, key: &str, value: Value) {
    let Some((head, rest)) = key.split_once(KEY_SEPARATOR) else {
        table.insert(key.to_owned(), value);
        return;
    };
    let entry = table
        .entry(head.to_owned())
        .or_insert_with(|| Value::Table(Table::new()));
    if !entry.is_table() {
        *entry = Value::Table(Table::new());
    }
    if let Value::Table(inner) = entry {
        insert_path(inner, rest, value);
    }
}

/// Returns the dotted path of the first key in `layer` that `reference` does not know.
pub(super) fn find_unknown_key(layer: &Table, reference: &Table, prefix: &str) -> Option<String> {
    for (key, value) in layer {
        let path = if prefix.is_empty() {
            key.clone()
        } else {
            format!("{prefix}{KEY_SEPARATOR}{key}")
        };
        match (reference.get(key), value) {
            (None, _) => return Some(path),
            (Some(Value::Table(known)), Value::Table(inner)) => {
                if let Some(found) = find_unknown_key(inner, known, &path) {
                    return Some(found);
                }
            }
            _ => {}
        }
    }
    None
}

/// Overlays `over` onto `base`: tables merge key by key, everything else is replaced.
pub(super) fn deep_merge(base: &mut Table, over: &Table) {
    for (key, value) in over {
        match (base.get_mut(key), value) {
            (Some(Value::Table(existing)), Value::Table(incoming)) => {
                deep_merge(existing, incoming);
            }
            _ => {
                base.insert(key.clone(), value.clone());
            }
        }
    }
}
