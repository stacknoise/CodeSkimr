//! The command behind the settings overview: every setting in effect with its value and the layer
//! that set it. It only reads; it changes nothing and never gives out a secret value.

use codeskimr_application::access;
use codeskimr_application::config::{Config, ConfigLayer, EffectiveSetting, LayerSource};
use codeskimr_application::error::CodedError;
use codeskimr_application::registry::{Command, CommandDescriptor, CommandId, Request};
use codeskimr_cli::GlobalOptions;
use codeskimr_domain::Permission;
use serde::Serialize;

use super::consts::{
    ORIGIN_CLI, ORIGIN_DEFAULT, ORIGIN_PROJECT, ORIGIN_UI, ORIGIN_USER, OVERRIDE_JOIN,
    SETTINGS_EFFECTIVE, SETTINGS_EFFECTIVE_DESCRIPTION, SETTINGS_EFFECTIVE_TITLE,
};
use crate::locations::Locations;

/// One setting for the screen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SettingOut {
    /// The dotted key path.
    pub key: String,
    /// The value as TOML text; empty for a secret value.
    pub value: String,
    /// `default`, `user`, `project` or `cli`.
    pub origin: &'static str,
    /// The value is secret and was left out.
    pub hidden: bool,
}

/// The user configuration file of this run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UserFileOut {
    /// Where the program looks for it.
    pub path: String,
    /// Whether the file exists and was read.
    pub found: bool,
}

/// The settings in effect and where they came from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SettingsOut {
    /// The user configuration file; `None` if no location is known.
    pub user_file: Option<UserFileOut>,
    /// The flags `--set key=value` of the start, secret values left out.
    pub overrides: Vec<String>,
    /// Every setting, in the order of the key paths.
    pub settings: Vec<SettingOut>,
}

/// Shows the settings that hold for this run of the program. They are read when the program
/// starts, so the screen never shows a value the program does not use.
pub struct EffectiveSettings {
    out: SettingsOut,
}

impl EffectiveSettings {
    /// Builds the view from the `layers` of the run.
    ///
    /// # Errors
    /// The coded failure of an invalid configuration (`CFG-101`, `CFG-103`).
    pub fn new(
        layers: &[ConfigLayer],
        locations: &Locations,
        global: &GlobalOptions,
    ) -> Result<Self, CodedError> {
        let settings = Config::effective(layers)
            .map_err(CodedError::caused_by)?
            .iter()
            .map(setting_out)
            .collect();
        Ok(Self {
            out: SettingsOut {
                user_file: locations.user_config.as_ref().map(|user| UserFileOut {
                    path: user.path.display().to_string(),
                    found: user.path.is_file(),
                }),
                overrides: global.overrides.iter().map(override_text).collect(),
                settings,
            },
        })
    }
}

fn setting_out(setting: &EffectiveSetting) -> SettingOut {
    SettingOut {
        key: setting.key.clone(),
        value: setting.value.clone(),
        origin: origin_name(setting.origin),
        hidden: setting.hidden,
    }
}

const fn origin_name(source: LayerSource) -> &'static str {
    match source {
        LayerSource::Default => ORIGIN_DEFAULT,
        LayerSource::User => ORIGIN_USER,
        LayerSource::Ui => ORIGIN_UI,
        LayerSource::Project => ORIGIN_PROJECT,
        LayerSource::Cli => ORIGIN_CLI,
    }
}

/// `key=value` of a flag; the value of a secret key stays out.
fn override_text((key, value): &(String, String)) -> String {
    if EffectiveSetting::is_secret_key(key) {
        return key.clone();
    }
    format!("{key}{OVERRIDE_JOIN}{value}")
}

impl Command for EffectiveSettings {
    type Input = ();
    type Output = SettingsOut;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(SETTINGS_EFFECTIVE),
        title_key: SETTINGS_EFFECTIVE_TITLE,
        description_key: SETTINGS_EFFECTIVE_DESCRIPTION,
        shortcut: None,
        read_only: true,
        permission: Permission::Read,
    };

    fn execute(&self, request: Request<'_, Self>) -> Result<SettingsOut, CodedError> {
        access::require(request.principal, Permission::Read)?;
        Ok(self.out.clone())
    }
}
