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
    /// Name of the stored credential that logs in to a Git source.
    pub credential: Option<String>,
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

/// Arguments of `codeskimr export`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportArgs {
    /// Id of the project, or the folder or Git URL it was scanned from.
    pub project: String,
    /// The HTML file to write.
    pub html: PathBuf,
}

/// The platforms that `credential login` can sign in to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderChoice {
    /// GitHub.
    GitHub,
    /// GitLab.
    GitLab,
}

/// What `codeskimr credential` is asked to do. The secret is never an argument: it is read from
/// the terminal without echo or from standard input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CredentialAction {
    /// Stores or replaces the credential `id` for the user name `user`.
    Set {
        /// Name of the credential.
        id: String,
        /// User name that goes with the secret.
        user: String,
    },
    /// Removes the credential `id`.
    Delete {
        /// Name of the credential.
        id: String,
    },
    /// Signs in with the OAuth device flow and stores the token as the credential `id`.
    Login {
        /// Name of the credential.
        id: String,
        /// The platform.
        provider: ProviderChoice,
        /// Id of the OAuth application that is registered at the platform.
        client_id: String,
        /// Address of a self-managed installation, for example `https://git.example.com`.
        host: Option<String>,
    },
}

/// The command to run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// `codeskimr credential`.
    Credential(CredentialAction),
    /// `codeskimr` without a command: start the server and open the browser.
    Start,
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
    /// `codeskimr export`.
    Export(ExportArgs),
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
