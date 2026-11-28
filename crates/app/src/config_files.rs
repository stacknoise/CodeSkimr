//! Reads the configuration files and the command line overrides into the typed [`Config`]
//! (blueprint section 14: user file, then project file, then command line flags).

use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use codeskimr_application::config::{Config, ConfigLayer, LayerSource, PROJECT_CONFIG_FILE};
use codeskimr_application::error::CodedError;
use codeskimr_cli::GlobalOptions;
use codeskimr_codes::ErrorCode;

use crate::locations::Locations;
use crate::ui_settings::UiSettingsFile;

/// Builds the configuration for a run.
///
/// `project_dir` is the folder of the scanned source; its `.codeskimr.toml` is read if it exists.
///
/// # Errors
/// - `CFG-102` if a named configuration file is missing or unreadable, or a file is not valid TOML.
/// - `CFG-103` / `CFG-101` for unknown keys and invalid values.
pub fn load(
    locations: &Locations,
    global: &GlobalOptions,
    project_dir: Option<&Path>,
) -> Result<Config, CodedError> {
    Config::resolve(&layers(locations, global, project_dir)?).map_err(CodedError::caused_by)
}

/// The layers of the override chain in their order: user file, the settings the UI saved, project
/// file, command line.
///
/// # Errors
/// - `CFG-102` if a named configuration file is missing or unreadable, or a file is not valid TOML,
///   or a flag value is not a TOML value.
pub fn layers(
    locations: &Locations,
    global: &GlobalOptions,
    project_dir: Option<&Path>,
) -> Result<Vec<ConfigLayer>, CodedError> {
    let saved = UiSettingsFile::in_dir(&locations.data_dir).layer();
    let with = layers_with(locations, global, project_dir, saved)?;
    if Config::resolve(&with).is_ok() {
        return Ok(with);
    }
    // Saved values that together break a rule of the configuration (a default above its maximum)
    // must not keep the program from starting: they are left out, as an invalid file is.
    layers_with(locations, global, project_dir, None)
}

/// The layers of [`layers`] with `saved` as the layer of the values the UI saved.
///
/// # Errors
/// The same as [`layers`].
pub fn layers_with(
    locations: &Locations,
    global: &GlobalOptions,
    project_dir: Option<&Path>,
    saved: Option<ConfigLayer>,
) -> Result<Vec<ConfigLayer>, CodedError> {
    let mut layers = Vec::new();
    if let Some(user) = &locations.user_config {
        layers.extend(read_layer(LayerSource::User, &user.path, user.required)?);
    }
    layers.extend(saved);
    if let Some(folder) = project_dir {
        let file = folder.join(PROJECT_CONFIG_FILE);
        layers.extend(read_layer(LayerSource::Project, &file, false)?);
    }
    let pairs = global
        .overrides
        .iter()
        .map(|(key, value)| (key.as_str(), value.as_str()));
    layers.push(ConfigLayer::from_overrides(pairs).map_err(CodedError::caused_by)?);
    Ok(layers)
}

/// The layer of one file; `None` if the file does not exist and `required` is false.
fn read_layer(
    source: LayerSource,
    path: &Path,
    required: bool,
) -> Result<Option<ConfigLayer>, CodedError> {
    match fs::read_to_string(path) {
        Ok(text) => ConfigLayer::parse(source, &text)
            .map(Some)
            .map_err(CodedError::caused_by),
        Err(error) if error.kind() == ErrorKind::NotFound && !required => Ok(None),
        Err(_) => Err(CodedError::new(ErrorCode::ConfigParseFailed)),
    }
}
