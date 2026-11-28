//! Writes the layers into the text of a project file (ADR 0065): everything but the `[[layers]]`
//! tables stays as it is, the tables are written anew at the end.

use std::collections::HashSet;

use serde::Serialize;

use super::consts::{
    BLOCK_SEPARATOR, LAYER_TABLE_HEADER, LINE_END, TABLE_HEADER_START, WINDOWS_LINE_END,
};
use super::error::{ConfigError, LayerSource, Violation};
use super::model::LayerRule;
use super::{Config, ConfigLayer};

/// The shape of the written tables.
#[derive(Serialize)]
struct Written<'a> {
    layers: &'a [LayerRule],
}

/// `text` without its tables that start with `header`: a table runs to the next table header.
pub(super) fn without_tables(text: &str, header: &str) -> String {
    without_tables_where(text, |head| head.starts_with(header))
}

/// Like [`without_tables`], for the tables whose header line (without its indent) `starts`
/// accepts.
pub(super) fn without_tables_where(text: &str, starts: impl Fn(&str) -> bool) -> String {
    let mut inside = false;
    let mut kept = Vec::new();
    for line in text.lines() {
        let head = line.trim_start();
        if starts(head) {
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

/// The layers of `text` as rules; none if the file has none.
///
/// # Errors
/// - `CFG-102` if `text` is not TOML, `CFG-101` if its layers cannot be read as rules.
fn existing_rules(text: &str) -> Result<Vec<LayerRule>, ConfigError> {
    let layer = ConfigLayer::parse(LayerSource::Project, text)?;
    match layer.table().get(super::consts::keys::LAYERS) {
        None => Ok(Vec::new()),
        Some(value) => value
            .clone()
            .try_into()
            .map_err(|_| ConfigError::OutOfRange {
                key: super::consts::keys::LAYERS,
                violation: Violation::MustBeLayerRule,
            }),
    }
}

/// Gives each wanted layer the `may_use` it has in `text`, as far as the layers it names stay.
fn with_known_limits(
    text: &str,
    mut wanted: Vec<LayerRule>,
) -> Result<Vec<LayerRule>, ConfigError> {
    let before = existing_rules(text)?;
    let names: HashSet<String> = wanted.iter().map(|rule| rule.name.clone()).collect();
    for rule in &mut wanted {
        rule.may_use = before
            .iter()
            .find(|old| old.name == rule.name)
            .and_then(|old| old.may_use.clone())
            .map(|used| used.into_iter().filter(|n| names.contains(n)).collect());
    }
    Ok(wanted)
}

/// The text of a project file with `wanted` as its layers, in that order. An entry's `may_use`
/// is taken over from the layer of the same name in `text`.
///
/// # Errors
/// - `CFG-102` if `text` is not valid TOML (the file is then left alone).
/// - `CFG-101` if the layers are not valid: a name or namespace is not valid, a name is used
///   twice, or a layer has no namespace.
pub fn with_layers(text: &str, wanted: Vec<LayerRule>) -> Result<String, ConfigError> {
    write_layers(text, &with_known_limits(text, wanted)?)
}

/// What one layer may use as the editor of the rules sets it: `None` is no limit.
pub type LayerLimit = (String, Option<Vec<String>>);

/// The text of a project file whose layers keep their names, namespaces and order but get the
/// `may_use` of `limits` (a layer without an entry keeps its own). A limit that names a layer
/// the file does not have is ignored. Without limits the text is returned as it is: the
/// `[[layers]]` tables are not touched, comments inside them stay.
///
/// # Errors
/// - `CFG-102` if `text` is not valid TOML (the file is then left alone).
/// - `CFG-101` if the layers cannot be read, or the result is not valid: a limit lets a layer
///   use a layer that does not exist.
pub fn with_layer_limits(text: &str, limits: &[LayerLimit]) -> Result<String, ConfigError> {
    if limits.is_empty() {
        ConfigLayer::parse(LayerSource::Project, text)?;
        return Ok(text.to_owned());
    }
    let mut layers = existing_rules(text)?;
    for layer in &mut layers {
        if let Some((_, may_use)) = limits.iter().find(|(name, _)| name == &layer.name) {
            layer.may_use.clone_from(may_use);
        }
    }
    write_layers(text, &layers)
}

fn write_layers(text: &str, rules: &[LayerRule]) -> Result<String, ConfigError> {
    let block = if rules.is_empty() {
        String::new()
    } else {
        toml::to_string(&Written { layers: rules }).map_err(|_| ConfigError::OutOfRange {
            key: super::consts::keys::LAYERS,
            violation: Violation::MustBeLayerRule,
        })?
    };
    rewrite(text, LAYER_TABLE_HEADER, &block)
}

/// The text of a project file without its tables that start with `header`, and with `block` (the
/// tables written anew, empty if there are none) at the end. The result must be a valid project
/// file; the line ends of `text` are kept.
///
/// # Errors
/// - `CFG-102` if `text` is not valid TOML (the file is then left alone).
/// - `CFG-101` if the result is not a valid configuration.
pub(super) fn rewrite(text: &str, header: &str, block: &str) -> Result<String, ConfigError> {
    rewrite_with(text, &without_tables(text, header), block)
}

/// Like `rewrite`, for a caller that has taken the old tables out of `text` itself: `kept` is
/// what stays, `text` is the file as it was (it is checked and gives the line ends).
pub(super) fn rewrite_with(text: &str, kept: &str, block: &str) -> Result<String, ConfigError> {
    ConfigLayer::parse(LayerSource::Project, text)?;
    let mut result = kept.trim_end().to_owned();
    if !block.is_empty() {
        if !result.is_empty() {
            result.push_str(BLOCK_SEPARATOR);
        }
        result.push_str(block);
    }
    Config::resolve(&[ConfigLayer::parse(LayerSource::Project, &result)?])?;
    if !result.is_empty() {
        result.push(LINE_END);
    }
    Ok(if text.contains(WINDOWS_LINE_END) {
        result.replace(LINE_END, WINDOWS_LINE_END)
    } else {
        result
    })
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
    fn a_file_with_windows_line_ends_keeps_them() {
        let text = "[keys]\r\nmax_files = 100\r\n";
        let written = with_layers(text, vec![rule("core", &["app::core"])]).unwrap();
        assert!(written.starts_with("[keys]\r\nmax_files = 100\r\n\r\n[[layers]]\r\n"));
        assert_eq!(
            written.matches('\n').count(),
            written.matches("\r\n").count()
        );
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

    #[test]
    fn a_limit_for_an_unknown_layer_is_ignored_and_a_broken_layer_table_is_refused() {
        let text = "[[layers]]\nname = \"a\"\nnamespaces = [\"x\"]\n";
        let limits = vec![("gone".to_owned(), Some(Vec::new()))];
        let written = with_layer_limits(text, &limits).unwrap();
        assert_eq!(written.trim_end(), text.trim_end());
        assert!(with_layer_limits("[[layers]]\nname = 3\n", &limits).is_err());
    }
}
