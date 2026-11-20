//! The two commands behind the "Browse" button of the new-project form (ADR 0075): whether the
//! operating system's folder dialog exists on this machine, and showing it.
//!
//! The dialog opens on the machine of the app. It hands back only the chosen path, which the user
//! still has to register with `projects.add`; the path is typed text for that command, so it is
//! checked there like any other.

use std::sync::Arc;

use codeskimr_application::error::CodedError;
use codeskimr_application::registry::{Command, CommandDescriptor, CommandId, Request};
use codeskimr_domain::Permission;
use serde::{Deserialize, Serialize};

use super::consts::{
    FOLDER_DIALOG_DESCRIPTION, FOLDER_DIALOG_TITLE, PICK_FOLDER_DESCRIPTION, PICK_FOLDER_TITLE,
    PROJECTS_FOLDER_DIALOG, PROJECTS_PICK_FOLDER,
};
use crate::folder_dialog::FolderPicker;

/// Nothing to say; the commands take no input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct NoInput {}

/// What `projects.folder_dialog` reports back.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct DialogInfo {
    /// Whether the app can show a folder dialog on this machine.
    pub available: bool,
}

/// What `projects.pick_folder` reports back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Picked {
    /// The chosen folder; `None` if the user cancelled, no dialog exists here or one is open.
    pub path: Option<String>,
}

/// Tells whether a folder dialog exists on this machine.
pub struct FolderDialogInfo {
    pub(super) picker: Arc<dyn FolderPicker>,
}

impl Command for FolderDialogInfo {
    type Input = NoInput;
    type Output = DialogInfo;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(PROJECTS_FOLDER_DIALOG),
        title_key: FOLDER_DIALOG_TITLE,
        description_key: FOLDER_DIALOG_DESCRIPTION,
        shortcut: None,
        read_only: true,
        permission: Permission::Read,
    };

    /// # Errors
    /// `AUTH-401` without the right to read.
    fn execute(&self, _request: Request<'_, Self>) -> Result<DialogInfo, CodedError> {
        Ok(DialogInfo {
            available: self.picker.available(),
        })
    }
}

/// Shows the folder dialog and waits for the choice.
pub struct PickFolder {
    pub(super) picker: Arc<dyn FolderPicker>,
}

impl Command for PickFolder {
    type Input = NoInput;
    type Output = Picked;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(PROJECTS_PICK_FOLDER),
        title_key: PICK_FOLDER_TITLE,
        description_key: PICK_FOLDER_DESCRIPTION,
        shortcut: None,
        read_only: false,
        permission: Permission::RegisterProject,
    };

    /// # Errors
    /// `AUTH-401` without the right to register projects. A cancelled or impossible dialog is no
    /// error: the path is `None`.
    fn execute(&self, _request: Request<'_, Self>) -> Result<Picked, CodedError> {
        let path = self
            .picker
            .pick()
            .map(|folder| folder.to_string_lossy().into_owned());
        Ok(Picked { path })
    }
}
