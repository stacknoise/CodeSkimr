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
    let mut layers = Vec::new();
    if let Some(user) = &locations.user_config {
        layers.extend(read_layer(LayerSource::User, &user.path, user.required)?);
    }
    if let Some(folder) = project_dir {
        let file = folder.join(PROJECT_CONFIG_FILE);
        layers.extend(read_layer(LayerSource::Project, &file, false)?);
    }
    let pairs = global
        .overrides
        .iter()
        .map(|(key, value)| (key.as_str(), value.as_str()));
    layers.push(ConfigLayer::from_overrides(pairs).map_err(CodedError::caused_by)?);
    Config::resolve(&layers).map_err(CodedError::caused_by)
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
