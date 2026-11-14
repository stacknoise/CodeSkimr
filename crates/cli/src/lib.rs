//! Command line interface (blueprint 10.2): the definition of the commands, the parsing of the
//! arguments and the texts the commands print.
//!
//! This crate knows no adapter. It turns `argv` into an [`Invocation`] and renders results; the
//! composition root (`codeskimr`) runs the request against the wired application.

mod args;
mod check;
mod consent;
mod consts;
mod credential;
mod error;
mod level2;
mod locale;
mod parser;
mod render;

pub use args::{
    Action, CheckArgs, CredentialAction, ExportArgs, GlobalOptions, Invocation, Level2Action,
    McpArgs, OpenArgs, Parsed, ProviderChoice, ScanArgs, ServeArgs,
};
pub use check::{check_json, check_text};
pub use consent::{
    confirm_prompt, confirm_warning, confirm_word, level2_confirmed, level2_declined,
    level2_revoked,
};
pub use error::CliError;
pub use level2::{Level2Line, Level2Reason};
pub use locale::language_from;
pub use parser::parse;
pub use render::{
    ScanSummary, credential_deleted, credential_stored, device_prompt, error_line, export_written,
    open_ready, scan_finished, scan_json, scan_started, secret_prompt, serve_started,
    watch_started, watch_updated,
};
