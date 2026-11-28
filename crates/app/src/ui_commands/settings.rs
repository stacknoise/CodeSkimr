//! The command behind the settings overview: every setting in effect with its value and the layer
//! that set it. It only reads; it changes nothing and never gives out a secret value.

use std::collections::BTreeMap;

use codeskimr_application::access;
use codeskimr_application::config::{Config, ConfigLayer, EffectiveSetting, LayerSource};
use codeskimr_application::error::CodedError;
use codeskimr_application::registry::{Command, CommandDescriptor, CommandId, Request};
use codeskimr_cli::GlobalOptions;
use codeskimr_domain::Permission;
use serde::Serialize;

use super::consts::{
    EDIT_CLI, EDIT_LOCKED, EDIT_SCAN, EDIT_START, ORIGIN_CLI, ORIGIN_DEFAULT, ORIGIN_PROJECT,
    ORIGIN_UI, ORIGIN_USER, OVERRIDE_JOIN, SETTINGS_EFFECTIVE, SETTINGS_EFFECTIVE_DESCRIPTION,
    SETTINGS_EFFECTIVE_TITLE,
};
use crate::config_files;
use crate::locations::Locations;
use crate::ui_settings::{Applies, editable};

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
    /// The value of the shipped defaults, as TOML text; empty for a secret value.
    pub default: String,
    /// Whether the UI may change it: `scan` (counts from the next scan), `start` (counts from the
    /// next start), `cli` (a flag of the console sets it and wins) or `locked` (only the
    /// configuration file may change it).
    pub edit: &'static str,
    /// The value saved since the program started, which counts from the next start; `None` if
    /// there is none.
    pub pending: Option<String>,
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
    /// The settings as the program started with them.
    started: Vec<EffectiveSetting>,
    defaults: BTreeMap<String, String>,
    locations: Locations,
    global: GlobalOptions,
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
        let started = Config::effective(layers).map_err(CodedError::caused_by)?;
        let defaults = Config::effective(&[])
            .map_err(CodedError::caused_by)?
            .into_iter()
            .map(|setting| (setting.key, setting.value))
            .collect();
        let mut view = Self {
            out: SettingsOut {
                user_file: locations.user_config.as_ref().map(|user| UserFileOut {
                    path: user.path.display().to_string(),
                    found: user.path.is_file(),
                }),
                overrides: global.overrides.iter().map(override_text).collect(),
                settings: Vec::new(),
            },
            started,
            defaults,
            locations: locations.clone(),
            global: global.clone(),
        };
        view.out.settings = view.rows(&view.started);
        Ok(view)
    }

    /// The rows for the settings as the files say now: what a scan reads again is shown as it is
    /// now, what only the start reads is shown as the program started with it and the newer value
    /// is named as pending.
    fn rows(&self, fresh: &[EffectiveSetting]) -> Vec<SettingOut> {
        let started: BTreeMap<&str, &EffectiveSetting> = self
            .started
            .iter()
            .map(|setting| (setting.key.as_str(), setting))
            .collect();
        fresh
            .iter()
            .map(|setting| {
                let mut out = self.row(setting);
                let applies = editable(&setting.key).map(|allowed| allowed.applies);
                if let (Some(Applies::Start), Some(was)) =
                    (applies, started.get(setting.key.as_str()))
                {
                    out = self.row(was);
                    out.pending = (!setting.hidden && setting.value != was.value)
                        .then(|| setting.value.clone());
                }
                out
            })
            .collect()
    }

    fn row(&self, setting: &EffectiveSetting) -> SettingOut {
        let mut out = setting_out(setting);
        out.default = if setting.hidden {
            String::new()
        } else {
            self.defaults.get(&setting.key).cloned().unwrap_or_default()
        };
        out.edit = edit_state(setting);
        out
    }
}

/// What the UI may do with a setting.
fn edit_state(setting: &EffectiveSetting) -> &'static str {
    if setting.hidden {
        return EDIT_LOCKED;
    }
    if setting.origin == LayerSource::Cli {
        return EDIT_CLI;
    }
    match editable(&setting.key).map(|allowed| allowed.applies) {
        Some(Applies::Scan) => EDIT_SCAN,
        Some(Applies::Start) => EDIT_START,
        None => EDIT_LOCKED,
    }
}

fn setting_out(setting: &EffectiveSetting) -> SettingOut {
    SettingOut {
        key: setting.key.clone(),
        value: setting.value.clone(),
        origin: origin_name(setting.origin),
        hidden: setting.hidden,
        default: String::new(),
        edit: EDIT_LOCKED,
        pending: None,
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
        let fresh = config_files::layers(&self.locations, &self.global, None)
            .ok()
            .and_then(|layers| Config::effective(&layers).ok());
        let mut out = self.out.clone();
        if let Some(fresh) = fresh {
            out.settings = self.rows(&fresh);
        }
        Ok(out)
    }
}
