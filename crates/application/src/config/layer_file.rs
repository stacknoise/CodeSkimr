//! Writes the layers into the text of a project file (ADR 0065): everything but the `[[layers]]`
//! tables stays as it is, the tables are written anew at the end.

use std::collections::HashSet;

use serde::Serialize;

use super::consts::{BLOCK_SEPARATOR, LAYER_TABLE_HEADER, LINE_END, TABLE_HEADER_START};
use super::error::{ConfigError, LayerSource, Violation};
use super::model::LayerRule;
use super::{Config, ConfigLayer};

/// The shape of the written tables.
#[derive(Serialize)]
struct Written<'a> {
    layers: &'a [LayerRule],
}

/// `text` without its `[[layers]]` tables: a table runs to the next table header.
fn without_layer_tables(text: &str) -> String {
    let mut inside = false;
    let mut kept = Vec::new();
    for line in text.lines() {
        let head = line.trim_start();
        if head.starts_with(LAYER_TABLE_HEADER) {
            inside = true;
        } else if head.starts_with(TABLE_HEADER_START) {
            inside = false;
        }
        if !inside {
            kept.push(line);
        }
    }
    kept.join(&LINE_END.to_string())
}

/// The layers of `text` as rules; none if the file has none or cannot be read as such.
fn existing_rules(text: &str) -> Vec<LayerRule> {
    text.parse::<toml::Table>()
        .ok()
        .and_then(|mut table| table.remove(super::consts::keys::LAYERS))
        .and_then(|value| value.try_into().ok())
        .unwrap_or_default()
}

/// Gives each wanted layer the `may_use` it has in `text`, as far as the layers it names stay.
fn with_known_limits(text: &str, mut wanted: Vec<LayerRule>) -> Vec<LayerRule> {
    let before = existing_rules(text);
    let names: HashSet<String> = wanted.iter().map(|rule| rule.name.clone()).collect();
    for rule in &mut wanted {
        rule.may_use = before
            .iter()
            .find(|old| old.name == rule.name)
            .and_then(|old| old.may_use.clone())
            .map(|used| used.into_iter().filter(|n| names.contains(n)).collect());
    }
    wanted
}

/// The text of a project file with `wanted` as its layers, in that order. An entry's `may_use`
/// is taken over from the layer of the same name in `text`.
///
/// # Errors
/// - `CFG-102` if `text` is not valid TOML (the file is then left alone).
/// - `CFG-101` if the layers are not valid: a name or namespace is not valid, a name is used
///   twice, or a layer has no namespace.
pub fn with_layers(text: &str, wanted: Vec<LayerRule>) -> Result<String, ConfigError> {
    ConfigLayer::parse(LayerSource::Project, text)?;
    let rules = with_known_limits(text, wanted);
    let mut result = without_layer_tables(text).trim_end().to_owned();
    if !rules.is_empty() {
        let block =
            toml::to_string(&Written { layers: &rules }).map_err(|_| ConfigError::OutOfRange {
                key: super::consts::keys::LAYERS,
                violation: Violation::MustBeLayerRule,
            })?;
        if !result.is_empty() {
            result.push_str(BLOCK_SEPARATOR);
        }
        result.push_str(&block);
    }
    Config::resolve(&[ConfigLayer::parse(LayerSource::Project, &result)?])?;
    if !result.is_empty() {
        result.push(LINE_END);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(name: &str, namespaces: &[&str]) -> LayerRule {
        LayerRule {
            name: name.to_owned(),
            namespaces: namespaces.iter().map(|n| (*n).to_owned()).collect(),
            may_use: None,
        }
    }

    #[test]
    fn a_file_without_layers_gets_them_at_the_end() {
        let text = "[keys]\nmax_files = 100\n";
        let written = with_layers(text, vec![rule("core", &["app::core"])]).unwrap();
        assert!(written.starts_with("[keys]\nmax_files = 100\n\n[[layers]]\n"));
        let again = Config::resolve(&[ConfigLayer::parse(LayerSource::Project, &written).unwrap()])
            .unwrap();
        assert_eq!(again.layers, vec![rule("core", &["app::core"])]);
    }

    #[test]
    fn old_layers_are_replaced_and_everything_else_stays() {
        let text = "# mine\n[[layers]]\nname = \"a\"\nnamespaces = [\n  \"x\",\n  \"y\",\n]\n\n[keys]\nmax_files = 7\n";
        let written = with_layers(text, vec![rule("b", &["z"])]).unwrap();
        assert!(written.starts_with("# mine\n[keys]\nmax_files = 7\n\n[[layers]]\nname = \"b\""));
        assert!(!written.contains("\"x\""));
    }

    #[test]
    fn no_layers_removes_the_tables() {
        let text = "[[layers]]\nname = \"a\"\nnamespaces = [\"x\"]\n";
        assert_eq!(with_layers(text, Vec::new()).unwrap(), "");
    }

    #[test]
    fn the_allowed_directions_of_a_kept_layer_stay_as_far_as_they_still_make_sense() {
        let text = "[[layers]]\nname = \"a\"\nnamespaces = [\"x\"]\nmay_use = [\"b\", \"gone\"]\n\n[[layers]]\nname = \"b\"\nnamespaces = [\"y\"]\n";
        let wanted = vec![rule("b", &["y"]), rule("a", &["x"])];
        let written = with_layers(text, wanted).unwrap();
        let config =
            Config::resolve(&[ConfigLayer::parse(LayerSource::Project, &written).unwrap()])
                .unwrap();
        assert_eq!(config.layers[1].may_use, Some(vec!["b".to_owned()]));
        assert_eq!(config.layers[0].may_use, None);
    }

    #[test]
    fn invalid_layers_and_broken_files_are_refused() {
        assert!(with_layers("", vec![rule("a", &[])]).is_err());
        assert!(with_layers("", vec![rule("a", &["x"]), rule("a", &["y"])]).is_err());
        assert!(with_layers("[scan", vec![rule("a", &["x"])]).is_err());
    }
}
