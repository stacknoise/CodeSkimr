//! Command line interface (blueprint 10.2): the definition of the commands, the parsing of the
//! arguments and the texts the commands print.
//!
//! This crate knows no adapter. It turns `argv` into an [`Invocation`] and renders results; the
//! composition root (`codeskimr`) runs the request against the wired application.

mod args;
mod consts;
mod error;
mod locale;
mod parser;
mod render;

pub use args::{Action, GlobalOptions, Invocation, McpArgs, Parsed, ScanArgs, ServeArgs};
pub use error::CliError;
pub use locale::language_from;
pub use parser::parse;
pub use render::{
    ScanSummary, error_line, scan_finished, scan_json, scan_started, serve_started, watch_started,
    watch_updated,
};
