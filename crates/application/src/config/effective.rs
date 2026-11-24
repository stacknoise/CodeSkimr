//! The settings that are in effect, each with the layer that set it (read-only view for the UI).

use toml::{Table, Value};

use super::consts::{KEY_SEPARATOR, NO_PREFIX, SECRET_KEY_WORDS, WORD_SEPARATOR};
use super::error::{ConfigError, LayerSource};
use super::layer::ConfigLayer;
use super::{Config, merge_layers, typed};

/// One setting as it holds now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectiveSetting {
    /// The dotted key path, such as `scan.parse_timeout_ms`.
    pub key: String,
    /// The value as TOML text; empty if the value is secret (`hidden`).
    pub value: String,
    /// The layer that set the value; `Default` if no file and no flag did.
    pub origin: LayerSource,
    /// The value is secret and is not given out.
    pub hidden: bool,
}

impl EffectiveSetting {
    /// Whether a key path names a secret, so that its value must not be shown anywhere.
    #[must_use]
    pub fn is_secret_key(key: &str) -> bool {
        let lower = key.to_lowercase();
        lower
            .split([KEY_SEPARATOR, WORD_SEPARATOR])
            .any(|word| SECRET_KEY_WORDS.contains(&word))
    }
}

impl Config {
    /// Every setting with its value and origin, in the order of the key paths. The chain is
    /// checked exactly as [`Config::resolve`] checks it, so only a valid configuration is listed.
    ///
    /// # Errors
    /// The same as [`Config::resolve`].
    pub fn effective(layers: &[ConfigLayer]) -> Result<Vec<EffectiveSetting>, ConfigError> {
        let merged = merge_layers(layers)?;
        typed(&merged.table)?;
        let mut found = Vec::new();
        collect(&merged.table, NO_PREFIX, &mut found);
        Ok(found
            .into_iter()
            .map(|(key, value)| {
                let hidden = EffectiveSetting::is_secret_key(&key);
                let origin = merged
                    .origins
                    .get(&key)
                    .copied()
                    .unwrap_or(LayerSource::Default);
                EffectiveSetting {
                    value: if hidden {
                        String::new()
                    } else {
                        value.to_string()
                    },
                    key,
                    origin,
                    hidden,
                }
            })
            .collect())
    }
}

/// The leaves of `table` with their dotted key paths.
fn collect(table: &Table, prefix: &str, found: &mut Vec<(String, Value)>) {
    for (key, value) in table {
        let path = if prefix.is_empty() {
            key.clone()
        } else {
            format!("{prefix}{KEY_SEPARATOR}{key}")
        };
        match value {
            Value::Table(inner) => collect(inner, &path, found),
            leaf => found.push((path, leaf.clone())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn find<'a>(all: &'a [EffectiveSetting], key: &str) -> &'a EffectiveSetting {
        all.iter().find(|setting| setting.key == key).unwrap()
    }

    #[test]
    fn without_layers_every_value_comes_from_the_defaults() {
        let all = Config::effective(&[]).unwrap();
        assert!(!all.is_empty());
        assert!(all.iter().all(|s| s.origin == LayerSource::Default));
        assert_eq!(find(&all, "server.port").value, "7421");
    }

    #[test]
    fn the_last_layer_that_set_a_value_is_named_as_its_origin() {
        let user = ConfigLayer::parse(
            LayerSource::User,
            "[scan]\nmax_workers = 2\nparse_timeout_ms = 6000\n",
        )
        .unwrap();
        let cli = ConfigLayer::from_overrides([("scan.parse_timeout_ms", "7000")]).unwrap();
        let all = Config::effective(&[user, cli]).unwrap();
        assert_eq!(find(&all, "scan.max_workers").origin, LayerSource::User);
        let timeout = find(&all, "scan.parse_timeout_ms");
        assert_eq!(
            (timeout.value.as_str(), timeout.origin),
            ("7000", LayerSource::Cli)
        );
        assert_eq!(
            find(&all, "scan.max_source_mb").origin,
            LayerSource::Default
        );
    }

    #[test]
    fn a_project_value_that_was_not_taken_is_not_named_as_origin() {
        let project = ConfigLayer::parse(
            LayerSource::Project,
            "[scan]\nparse_timeout_ms = 999999\nmax_candidates = 50\n[server]\nport = 1\n",
        )
        .unwrap();
        let all = Config::effective(&[project]).unwrap();
        assert_eq!(
            find(&all, "scan.parse_timeout_ms").origin,
            LayerSource::Default
        );
        assert_eq!(
            find(&all, "scan.max_candidates").origin,
            LayerSource::Project
        );
        assert_eq!(find(&all, "server.port").origin, LayerSource::Project);
    }

    #[test]
    fn an_invalid_chain_lists_nothing() {
        let bad = ConfigLayer::parse(LayerSource::User, "[nothing]\nhere = 1\n").unwrap();
        assert!(matches!(
            Config::effective(&[bad]),
            Err(ConfigError::UnknownKey { .. })
        ));
    }

    #[test]
    fn no_shipped_key_is_secret_and_a_secret_key_would_be_hidden() {
        let all = Config::effective(&[]).unwrap();
        assert!(all.iter().all(|s| !s.hidden));
        assert!(EffectiveSetting::is_secret_key("server.api_Token"));
        assert!(EffectiveSetting::is_secret_key("git.password"));
        assert!(EffectiveSetting::is_secret_key("git.ssh_key"));
        assert!(!EffectiveSetting::is_secret_key("scan.max_workers"));
        assert!(!EffectiveSetting::is_secret_key("similarity.min_tokens"));
        assert!(!EffectiveSetting::is_secret_key("keys.max_files"));
    }
}
