//! The settings of the UI and what the server needs to serve it.

use std::path::PathBuf;
use std::time::Duration;

use serde::Serialize;

/// The editor link the UI builds (blueprint 15.4, ADR 0028). The UI checks the scheme against the
/// list again, so that a template that was changed on the way is never opened.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EditorSettings {
    /// URL template with the placeholders `{path}` and `{line}`.
    pub url_template: String,
    /// The schemes a template may use.
    pub allowed_schemes: Vec<String>,
}

/// The settings the UI reads at start. None of them is secret.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UiSettings {
    /// The editor link.
    pub editor: EditorSettings,
    /// The theme used until the user chooses one.
    pub default_theme: String,
}

/// Everything the UI side of the server needs from the configuration.
#[derive(Debug, Clone)]
pub struct UiOptions {
    /// Folder with the built UI; without one the server only answers the API.
    pub dir: Option<PathBuf>,
    /// How long a launch code is valid.
    pub launch_lifetime: Duration,
    /// The settings the UI reads at start.
    pub settings: UiSettings,
}
