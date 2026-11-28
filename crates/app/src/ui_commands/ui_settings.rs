//! The commands that read and write the settings the UI saved (ADR 0103). The file and the layer
//! the configuration reads from it are in `crate::ui_settings`.

use std::sync::{Mutex, MutexGuard, PoisonError};

use codeskimr_application::access;
use codeskimr_application::config::{Config, MAX_INDEXER_MEMORY_MB};
use codeskimr_application::error::CodedError;
use codeskimr_application::registry::{Command, CommandDescriptor, CommandId, Request};
use codeskimr_cli::GlobalOptions;
use codeskimr_codes::ErrorCode;
use codeskimr_domain::Permission;
use serde::Deserialize;

use super::consts::{
    UI_SETTINGS_GET, UI_SETTINGS_GET_DESCRIPTION, UI_SETTINGS_GET_TITLE,
    UI_SETTINGS_SET_INDEXER_MEMORY, UI_SETTINGS_SET_INDEXER_MEMORY_DESCRIPTION,
    UI_SETTINGS_SET_INDEXER_MEMORY_TITLE, UI_SETTINGS_SET_VALUE, UI_SETTINGS_SET_VALUE_DESCRIPTION,
    UI_SETTINGS_SET_VALUE_TITLE,
};
use super::preferences::write_private;
use crate::config_files::layers_with;
use crate::locations::Locations;
use crate::ui_settings::{UiSettings, UiSettingsFile, UiValue, editable, known_indexers};

/// Replaces the file with `settings`.
///
/// # Errors
/// `APP-201` if the data folder cannot be written.
/// Holds the file of the saved settings for one change, so two calls at the same moment cannot
/// overwrite each other's entry.
static SAVE_LOCK: Mutex<()> = Mutex::new(());

/// Takes the lock of a change; a call that panicked while holding it does not block the next one.
fn lock_file() -> MutexGuard<'static, ()> {
    SAVE_LOCK.lock().unwrap_or_else(PoisonError::into_inner)
}

fn write(file: &UiSettingsFile, settings: &UiSettings) -> Result<(), CodedError> {
    let text = settings
        .to_file_text()
        .ok_or_else(|| CodedError::new(ErrorCode::DataDirUnavailable))?;
    write_private(file.path(), &text)
}

/// Returns the settings the UI saved.
pub struct GetUiSettings {
    pub(super) file: UiSettingsFile,
}

impl Command for GetUiSettings {
    type Input = ();
    type Output = UiSettings;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(UI_SETTINGS_GET),
        title_key: UI_SETTINGS_GET_TITLE,
        description_key: UI_SETTINGS_GET_DESCRIPTION,
        shortcut: None,
        read_only: true,
        permission: Permission::Read,
    };

    fn execute(&self, request: Request<'_, Self>) -> Result<UiSettings, CodedError> {
        access::require(request.principal, Permission::Read)?;
        Ok(self.file.read())
    }
}

/// The memory limit of one indexer: `megabytes` of `null` forgets the saved value.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndexerMemory {
    /// Name of the indexer, for example `scip-typescript`.
    pub indexer: String,
    /// Limit in megabytes; `0` follows `level2.max_memory_mb`.
    pub megabytes: Option<u32>,
}

/// Saves or forgets the memory limit of one indexer. The new limit counts from the next scan of the
/// running program (`crate::fresh_limits`).
pub struct SetIndexerMemory {
    pub(super) file: UiSettingsFile,
}

impl Command for SetIndexerMemory {
    type Input = IndexerMemory;
    type Output = UiSettings;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(UI_SETTINGS_SET_INDEXER_MEMORY),
        title_key: UI_SETTINGS_SET_INDEXER_MEMORY_TITLE,
        description_key: UI_SETTINGS_SET_INDEXER_MEMORY_DESCRIPTION,
        shortcut: None,
        read_only: false,
        permission: Permission::Manage,
    };

    /// # Errors
    /// - `API-101` if the indexer is unknown or the limit is above `MAX_INDEXER_MEMORY_MB`.
    /// - `APP-201` if the data folder cannot be written.
    fn execute(&self, request: Request<'_, Self>) -> Result<UiSettings, CodedError> {
        access::require(request.principal, Permission::Manage)?;
        let IndexerMemory { indexer, megabytes } = request.input;
        let acceptable = known_indexers().contains(&indexer)
            && megabytes.is_none_or(|value| value <= MAX_INDEXER_MEMORY_MB);
        if !acceptable {
            return Err(CodedError::new(ErrorCode::RequestInvalid));
        }
        let _guard = lock_file();
        let mut settings = self.file.read();
        match megabytes {
            Some(value) => settings.indexer_memory_mb.insert(indexer, value),
            None => settings.indexer_memory_mb.remove(&indexer),
        };
        write(&self.file, &settings)?;
        Ok(settings)
    }
}

/// The value of one setting: `value` of `null` forgets the saved value, so the setting is back to
/// what the files and flags say.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SettingValue {
    /// The dotted key, for example `scan.max_workers`.
    pub key: String,
    /// The new value, a number or a switch.
    pub value: Option<UiValue>,
}

/// Saves or forgets the value of one number or switch (ADR 0118). Only keys that
/// [`editable`] names are accepted, and the whole configuration with the new value must still be
/// valid, so a saved value can never keep the program from starting.
pub struct SetSettingValue {
    pub(super) file: UiSettingsFile,
    pub(super) locations: Locations,
    pub(super) global: GlobalOptions,
}

impl Command for SetSettingValue {
    type Input = SettingValue;
    type Output = UiSettings;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(UI_SETTINGS_SET_VALUE),
        title_key: UI_SETTINGS_SET_VALUE_TITLE,
        description_key: UI_SETTINGS_SET_VALUE_DESCRIPTION,
        shortcut: None,
        read_only: false,
        permission: Permission::Manage,
    };

    /// # Errors
    /// - `API-101` for a key the UI may not change or a value of another kind.
    /// - `CFG-101` if the configuration with the value is invalid (a value out of range, a default
    ///   above its maximum).
    /// - `APP-201` if the data folder cannot be written.
    fn execute(&self, request: Request<'_, Self>) -> Result<UiSettings, CodedError> {
        access::require(request.principal, Permission::Manage)?;
        let SettingValue { key, value } = request.input;
        let allowed = editable(&key).ok_or_else(|| CodedError::new(ErrorCode::RequestInvalid))?;
        let _guard = lock_file();
        let mut settings = self.file.read();
        match value {
            Some(given) => {
                let fitted = allowed
                    .fit(given)
                    .ok_or_else(|| CodedError::new(ErrorCode::RequestInvalid))?;
                settings.values.insert(key, fitted);
            }
            None => {
                settings.values.remove(&key);
            }
        }
        let layers = layers_with(&self.locations, &self.global, None, settings.to_layer())?;
        Config::resolve(&layers).map_err(CodedError::caused_by)?;
        write(&self.file, &settings)?;
        Ok(settings)
    }
}
