//! Command line interface (blueprint 10.2): the definition of the commands, the parsing of the
//! arguments and the texts the commands print.
//!
//! This crate knows no adapter. It turns `argv` into an [`Invocation`] and renders results; the
//! composition root (`codeskimr`) runs the request against the wired application.

mod args;
mod check;
mod codes;
mod consent;
mod consts;
mod credential;
mod error;
mod level2;
mod locale;
mod notes;
mod parser;
mod render;
mod scope;
mod warning;

pub use args::{
    Action, AddAnnotation, AlertsAction, AnnotationFields, AnnotationsAction, ById, CheckArgs,
    Clears, CodesArgs, CredentialAction, DismissAlerts, ExportArgs, GlobalOptions, Invocation,
    Level2Action, ListAlerts, ListAnnotations, McpArgs, OpenArgs, PageArgs, Parsed, ProviderChoice,
    ScanArgs, ScopeArgs, ServeArgs, StartArgs, SymbolArg, UpdateAnnotation,
};
pub use check::{check_json, check_text};
pub use codes::codes_text;
pub use consent::{
    confirm_prompt, confirm_target, confirm_warning, from_ui_notice, from_ui_wait,
    level2_confirmed, level2_declined, level2_revoked,
};
pub use error::CliError;
pub use level2::{Level2Line, Level2Reason};
pub use locale::language_from;
pub use parser::parse;
pub use render::{
    ScanSummary, StageLine, TrayLabels, credential_deleted, credential_stored, device_prompt,
    error_line, export_written, open_ready, open_url_warning, scan_finished, scan_json,
    scan_started, scan_tools, secret_prompt, serve_started, start_keep_open, start_tray,
    tray_labels, watch_started, watch_updated,
};
pub use warning::{LeftOutLine, LeftOutView, WarningLine};
