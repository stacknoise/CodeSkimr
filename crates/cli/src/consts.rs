//! Fixed values of the command line: command and flag names, value placeholders, languages.

/// Name of the program in usage lines.
pub const BINARY_NAME: &str = "codeskimr";
/// Version shown by `--version`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Sub-command that scans a source.
pub const COMMAND_SCAN: &str = "scan";
/// Sub-command that serves MCP over standard input and output.
pub const COMMAND_MCP: &str = "mcp";
/// Sub-command that starts the local server.
pub const COMMAND_SERVE: &str = "serve";
/// Sub-command that opens the UI in the browser.
pub const COMMAND_OPEN: &str = "open";
/// Flag of `open` that keeps the browser from starting.
pub const ARG_NO_BROWSER: &str = "no-browser";

/// Sub-command that checks the code against the code rules.
pub const COMMAND_CHECK: &str = "check";
/// Argument id of the paths of `check`.
pub const ARG_PATHS: &str = "paths";
/// Placeholder shown for a file or folder to check.
pub const VALUE_PATH: &str = "PATH";

/// Argument id and long flag of the source to scan.
pub const ARG_SOURCE: &str = "source";
/// Argument id and long flag of the Git branch.
pub const ARG_BRANCH: &str = "branch";
/// Argument id and long flag of the JSON output switch.
pub const ARG_JSON: &str = "json";
/// Argument id and long flag of the watch mode switch.
pub const ARG_WATCH: &str = "watch";
/// Argument id and long flag of the project folder of `mcp`.
pub const ARG_PROJECT: &str = "project";
/// Argument id and long flag of the data folder.
pub const ARG_DATA_DIR: &str = "data-dir";
/// Argument id and long flag of the language pack folder.
pub const ARG_LANGUAGES: &str = "languages";
/// Argument id and long flag of the user configuration file.
pub const ARG_CONFIG: &str = "config";
/// Argument id and long flag of a setting override.
pub const ARG_SET: &str = "set";

/// Placeholder shown for a folder value.
pub const VALUE_DIR: &str = "DIR";
/// Placeholder shown for a file value.
pub const VALUE_FILE: &str = "FILE";
/// Placeholder shown for a Git branch.
pub const VALUE_BRANCH: &str = "BRANCH";
/// Placeholder shown for a setting override.
pub const VALUE_OVERRIDE: &str = "KEY=VALUE";
/// Placeholder shown for the source to scan.
pub const VALUE_SOURCE: &str = "PATH|GIT-URL";

/// Separates key and value in `--set key=value`.
pub const OVERRIDE_SEPARATOR: char = '=';

/// German, the second language of the texts.
pub const LANGUAGE_DE: &str = "de";
/// English, the default language of the texts.
pub const LANGUAGE_EN: &str = "en";

/// Number of decimals of the seconds in the scan summary.
pub const SECONDS_DECIMALS: i32 = 2;
/// Factor that moves `SECONDS_DECIMALS` decimals in front of the point.
pub const SECONDS_BASE: f64 = 10.0;

/// Names of the Fluent arguments of the messages.
pub mod arg {
    /// `{ $code }`: the error code id.
    pub const CODE: &str = "code";
    /// `{ $message }`: the user-facing text of an error.
    pub const MESSAGE: &str = "message";
    /// `{ $path }`: the source being scanned.
    pub const PATH: &str = "path";
    /// `{ $files }`: number of scanned files.
    pub const FILES: &str = "files";
    /// `{ $symbols }`: number of symbols.
    pub const SYMBOLS: &str = "symbols";
    /// `{ $references }`: number of references.
    pub const REFERENCES: &str = "references";
    /// `{ $snapshot }`: id of the new snapshot.
    pub const SNAPSHOT: &str = "snapshot";
    /// `{ $seconds }`: duration of the scan.
    pub const SECONDS: &str = "seconds";
    /// `{ $changed }`: number of files that changed since the last scan.
    pub const CHANGED: &str = "changed";
    /// `{ $millis }`: duration of a re-scan in milliseconds.
    pub const MILLIS: &str = "millis";
    /// `{ $url }`: address of the local server.
    pub const URL: &str = "url";
    /// `{ $line }`: one-based line of a finding.
    pub const LINE: &str = "line";
    /// `{ $name }`: qualified name of a function.
    pub const NAME: &str = "name";
    /// `{ $lines }`: code lines of a function.
    pub const LINES: &str = "lines";
    /// `{ $limit }`: the configured maximum.
    pub const LIMIT: &str = "limit";
    /// `{ $percent }`: similarity in percent.
    pub const PERCENT: &str = "percent";
    /// `{ $original }`: qualified name of the function that was copied.
    pub const ORIGINAL: &str = "original";
    /// `{ $findings }`: number of findings.
    pub const FINDINGS: &str = "findings";
    /// `{ $codes }`: error codes, separated by `CODE_LIST_SEPARATOR`.
    pub const CODES: &str = "codes";
    /// `{ $seconds }` in the launch message is the same name as the scan duration.
    pub const LAUNCH_SECONDS: &str = SECONDS;
}

/// Separates the lines of a multi-line message.
pub const LINE_BREAK: &str = "\n";

/// Separates the codes in a list of codes.
pub const CODE_LIST_SEPARATOR: &str = ", ";
