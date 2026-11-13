//! What the command line asks for, free of any parser type.

use std::path::PathBuf;

/// Options that every command accepts.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GlobalOptions {
    /// Folder for the database and the Git cache.
    pub data_dir: Option<PathBuf>,
    /// Folder with the language packs.
    pub languages_dir: Option<PathBuf>,
    /// User configuration file.
    pub config_file: Option<PathBuf>,
    /// Setting overrides as `(dotted key, raw TOML value)`, in the order given.
    pub overrides: Vec<(String, String)>,
}

/// Arguments of `codeskimr scan`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanArgs {
    /// A folder or a Git URL.
    pub source: String,
    /// Branch of a Git source.
    pub branch: Option<String>,
    /// Print the summary as JSON.
    pub json: bool,
    /// Keep running after the scan and re-scan whenever files change.
    pub watch: bool,
}

/// Arguments of `codeskimr mcp`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct McpArgs {
    /// Folder of the scanned project; the current folder if absent.
    pub project: Option<PathBuf>,
}

/// Arguments of `codeskimr serve`: none yet; the port comes from the setting `server.port`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ServeArgs;
/// Arguments of `codeskimr check`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CheckArgs {
    /// Files or folders to check, relative to the current folder; all of it if empty.
    pub paths: Vec<PathBuf>,
    /// Print the report as JSON.
    pub json: bool,
}

/// Arguments of `codeskimr open`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OpenArgs {
    /// Only create the launch file and print its path; do not start a browser.
    pub no_browser: bool,
}

/// The command to run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// `codeskimr scan <source>`.
    Scan(ScanArgs),
    /// `codeskimr mcp`.
    Mcp(McpArgs),
    /// `codeskimr serve`.
    Serve(ServeArgs),
    /// `codeskimr check`.
    Check(CheckArgs),
    /// `codeskimr open`.
    Open(OpenArgs),
}

/// A parsed command line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invocation {
    /// Options of every command.
    pub global: GlobalOptions,
    /// The command.
    pub action: Action,
}

/// The result of parsing: a request to run, or text to show (help, version) before exiting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Parsed {
    /// Run this request.
    Run(Invocation),
    /// Print this text to standard output and exit successfully.
    Display(String),
}
