//! The commands that read and write the settings the UI saved (ADR 0103). The file and the layer
//! the configuration reads from it are in `crate::ui_settings`.

use codeskimr_application::access;
use codeskimr_application::config::MAX_INDEXER_MEMORY_MB;
use codeskimr_application::error::CodedError;
use codeskimr_application::registry::{Command, CommandDescriptor, CommandId, Request};
use codeskimr_codes::ErrorCode;
use codeskimr_domain::Permission;
use serde::Deserialize;

use super::consts::{
    UI_SETTINGS_GET, UI_SETTINGS_GET_DESCRIPTION, UI_SETTINGS_GET_TITLE,
    UI_SETTINGS_SET_INDEXER_MEMORY, UI_SETTINGS_SET_INDEXER_MEMORY_DESCRIPTION,
    UI_SETTINGS_SET_INDEXER_MEMORY_TITLE,
};
use super::preferences::write_private;
use crate::ui_settings::{UiSettings, UiSettingsFile, known_indexers};

/// Replaces the file with `settings`.
///
/// # Errors
/// `APP-201` if the data folder cannot be written.
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
        let mut settings = self.file.read();
        match megabytes {
            Some(value) => settings.indexer_memory_mb.insert(indexer, value),
            None => settings.indexer_memory_mb.remove(&indexer),
        };
        write(&self.file, &settings)?;
        Ok(settings)
    }
}
