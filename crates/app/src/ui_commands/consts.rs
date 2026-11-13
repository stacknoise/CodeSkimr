//! Fixed values of the commands the browser UI uses.

use codeskimr_codes::i18n;

/// Id of the command that lists the projects.
pub const PROJECTS_LIST: &str = "projects.list";
/// Id of the command that registers a project and scans it.
pub const PROJECTS_ADD: &str = "projects.add";
/// Id of the command that scans a registered project again.
pub const PROJECTS_RESCAN: &str = "projects.rescan";

/// Shortcut of the new-project form.
pub const SHORTCUT_NEW: &str = "N";
/// Shortcut of the re-scan.
pub const SHORTCUT_RESCAN: &str = "R";

/// Title key of `projects.list`.
pub const LIST_TITLE: &str = i18n::COMMAND_PROJECTS_LIST_TITLE;
/// Description key of `projects.list`.
pub const LIST_DESCRIPTION: &str = i18n::COMMAND_PROJECTS_LIST_DESCRIPTION;
/// Title key of `projects.add`.
pub const ADD_TITLE: &str = i18n::COMMAND_PROJECTS_ADD_TITLE;
/// Description key of `projects.add`.
pub const ADD_DESCRIPTION: &str = i18n::COMMAND_PROJECTS_ADD_DESCRIPTION;
/// Title key of `projects.rescan`.
pub const RESCAN_TITLE: &str = i18n::COMMAND_PROJECTS_RESCAN_TITLE;
/// Description key of `projects.rescan`.
pub const RESCAN_DESCRIPTION: &str = i18n::COMMAND_PROJECTS_RESCAN_DESCRIPTION;

/// How a project's source is named on the wire.
pub const KIND_FOLDER: &str = "folder";
/// How a Git source is named on the wire.
pub const KIND_GIT: &str = "git";
