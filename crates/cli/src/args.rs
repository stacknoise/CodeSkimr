//! What the command line asks for, free of any parser type.

use std::ffi::OsString;
use std::path::PathBuf;

use crate::consts::{
    ARG_CONFIG, ARG_DATA_DIR, ARG_LANGUAGES, ARG_SET, LONG_FLAG_PREFIX, OVERRIDE_SEPARATOR,
};

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

impl GlobalOptions {
    /// The options as separate command line arguments, in the form the parser reads them again, so
    /// that a program started with them sees the same folders, file and settings (ADR 0071,
    /// decision 5). Every value is its own argument; nothing is joined for a shell.
    #[must_use]
    pub fn arguments(&self) -> Vec<OsString> {
        let flag = |name: &str| OsString::from(format!("{LONG_FLAG_PREFIX}{name}"));
        let paths = [
            (ARG_DATA_DIR, &self.data_dir),
            (ARG_LANGUAGES, &self.languages_dir),
            (ARG_CONFIG, &self.config_file),
        ];
        let mut arguments = Vec::new();
        for (name, value) in paths {
            if let Some(path) = value {
                arguments.push(flag(name));
                arguments.push(path.clone().into_os_string());
            }
        }
        for (key, value) in &self.overrides {
            arguments.push(flag(ARG_SET));
            arguments.push(OsString::from(format!("{key}{OVERRIDE_SEPARATOR}{value}")));
        }
        arguments
    }
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
    /// Display name of the project; it also renames a project that is registered already.
    pub name: Option<String>,
}

/// Arguments of `codeskimr scope`: show the scope of a project, or change it first (ADR 0102).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScopeArgs {
    /// Id of the project, or the folder or Git URL it was scanned from.
    pub project: String,
    /// Languages to leave out.
    pub without_languages: Vec<String>,
    /// Languages to take back in.
    pub with_languages: Vec<String>,
    /// Folders to leave out.
    pub without_folders: Vec<String>,
    /// Folders to take back in.
    pub with_folders: Vec<String>,
    /// Take everything back in before the other changes apply.
    pub reset: bool,
    /// Print the answer as JSON.
    pub json: bool,
}

impl ScopeArgs {
    /// Whether the command changes the scope; without a change it only shows it.
    #[must_use]
    pub fn changes(&self) -> bool {
        self.reset
            || [
                &self.without_languages,
                &self.with_languages,
                &self.without_folders,
                &self.with_folders,
            ]
            .iter()
            .any(|list| !list.is_empty())
    }
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
/// Arguments of `codeskimr codes`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodesArgs {
    /// The id of the code to explain, such as `SRC-201`; all codes are listed if it is `None`.
    pub code: Option<String>,
}

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
    /// Print the address with the one-time code instead of writing a launch file (opt-in, ADR 0088).
    /// The parser demands `no_browser` with it.
    pub print_url: bool,
}

/// Arguments of the bare `codeskimr`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StartArgs {
    /// Stay in this console window instead of running in the system tray (ADR 0067).
    pub console: bool,
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

/// What `codeskimr level2` is asked to do. Both ways are local and interactive: nothing else, not
/// MCP, not the REST API, not a flag, can confirm level 2 (ADR 0033, decision 10).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Level2Action {
    /// Allows the indexers for the project after the user typed the confirmation word.
    Confirm {
        /// Id of the project, or the folder or Git URL it was scanned from.
        project: String,
        /// The window was opened by the server for the page in the browser (`--from-ui`,
        /// ADR 0071): it says so, and waits for Enter at the end. The consent is the same.
        from_ui: bool,
    },
    /// Forbids them again.
    Revoke {
        /// Id of the project, or the folder or Git URL it was scanned from.
        project: String,
    },
}

/// A symbol as the command line names it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolArg {
    /// Kind label, such as `class` or `method`.
    pub kind: String,
    /// Qualified name.
    pub name: String,
    /// Parameter types, to tell overloads apart.
    pub parameters: Vec<String>,
}

/// Where a page of a list starts and how long it may be.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PageArgs {
    /// Rows to skip.
    pub offset: Option<u32>,
    /// Rows to return.
    pub limit: Option<u32>,
}

/// The fields of an annotation that the command line can set. A field that is `None` is not sent.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AnnotationFields {
    /// Title of a task.
    pub title: Option<String>,
    /// The text.
    pub text: Option<String>,
    /// Status of a task.
    pub status: Option<String>,
    /// Due date in seconds since the Unix epoch.
    pub due: Option<i64>,
    /// Assignee of a task.
    pub assignee: Option<String>,
}

/// Arguments of `codeskimr annotations list`. `project` is an id, or the folder or Git URL the
/// project was scanned from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListAnnotations {
    /// The project.
    pub project: String,
    /// Only this symbol.
    pub symbol: Option<SymbolArg>,
    /// Only this kind.
    pub kind: Option<String>,
    /// Only tasks in this status.
    pub status: Option<String>,
    /// The page.
    pub page: PageArgs,
    /// Print JSON.
    pub json: bool,
}

/// Arguments of `codeskimr annotations add`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddAnnotation {
    /// The project.
    pub project: String,
    /// The symbol.
    pub symbol: SymbolArg,
    /// `critical`, `note` or `task`.
    pub kind: String,
    /// What it says.
    pub fields: AnnotationFields,
    /// Print JSON.
    pub json: bool,
}

/// Which fields `codeskimr annotations update` removes.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Clears {
    /// Remove the text.
    pub text: bool,
    /// Remove the due date.
    pub due: bool,
    /// Remove the assignee.
    pub assignee: bool,
}

/// Arguments of `codeskimr annotations update`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateAnnotation {
    /// The project.
    pub project: String,
    /// Id of the annotation.
    pub id: u64,
    /// Assigns it to another symbol.
    pub symbol: Option<SymbolArg>,
    /// What to set.
    pub fields: AnnotationFields,
    /// What to remove.
    pub clears: Clears,
    /// Print JSON.
    pub json: bool,
}

/// Arguments of `codeskimr annotations remove` and `codeskimr alerts dismiss`: one id of a project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ById {
    /// The project.
    pub project: String,
    /// Id of the annotation or the alert.
    pub id: u64,
    /// Print JSON.
    pub json: bool,
}

/// What `codeskimr annotations` is asked to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnnotationsAction {
    /// Lists the annotations of a project.
    List(ListAnnotations),
    /// Adds an annotation.
    Add(AddAnnotation),
    /// Changes an annotation.
    Update(UpdateAnnotation),
    /// Deletes an annotation.
    Remove(ById),
}

/// Arguments of `codeskimr alerts list`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListAlerts {
    /// The project.
    pub project: String,
    /// `open` or `dismissed`.
    pub state: Option<String>,
    /// Only these kinds.
    pub kinds: Vec<String>,
    /// The page.
    pub page: PageArgs,
    /// Print JSON.
    pub json: bool,
}

/// Arguments of `codeskimr alerts dismiss-all`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DismissAlerts {
    /// The project.
    pub project: String,
    /// Only these kinds.
    pub kinds: Vec<String>,
    /// The explicit confirmation (`--yes`).
    pub confirm: bool,
    /// Print JSON.
    pub json: bool,
}

/// What `codeskimr alerts` is asked to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlertsAction {
    /// Lists the alerts of a project.
    List(ListAlerts),
    /// Dismisses one alert.
    Dismiss(ById),
    /// Dismisses every open alert of a filter.
    DismissAll(DismissAlerts),
}

/// What every query command takes: the project and the output form.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct QueryTarget {
    /// Id of the project, or the folder or Git URL it was scanned from; the project of the
    /// current folder if absent.
    pub project: Option<String>,
    /// Print JSON.
    pub json: bool,
}

/// Arguments of `codeskimr symbols`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolsQuery {
    /// Part of the name.
    pub query: String,
    /// Only this kind.
    pub kind: Option<String>,
    /// Number of hits.
    pub limit: Option<u32>,
    /// Project and output.
    pub target: QueryTarget,
}

/// Arguments of `codeskimr usages`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsagesQuery {
    /// Id or qualified name.
    pub symbol: String,
    /// `incoming` or `outgoing`.
    pub direction: Option<String>,
    /// Steps of the walk.
    pub depth: Option<u32>,
    /// References from tests count.
    pub include_tests: bool,
    /// Number of usages.
    pub limit: Option<u32>,
    /// Project and output.
    pub target: QueryTarget,
}

/// Arguments of `codeskimr impact`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImpactQuery {
    /// Id or qualified name.
    pub symbol: String,
    /// Number of symbols.
    pub limit: Option<u32>,
    /// Project and output.
    pub target: QueryTarget,
}

/// Arguments of `codeskimr context`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextQuery {
    /// Id or qualified name.
    pub symbol: String,
    /// `short`, `medium` or `long`.
    pub budget: Option<String>,
    /// Project and output.
    pub target: QueryTarget,
}

/// Arguments of `codeskimr unused`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnusedQuery {
    /// `all`, `types` or `functions`.
    pub kind: Option<String>,
    /// Only symbols of this language pack.
    pub language: Option<String>,
    /// `likely`, `public` or `approximate`.
    pub assessment: Option<String>,
    /// Test code counts.
    pub include_tests: bool,
    /// Number of symbols.
    pub limit: Option<u32>,
    /// Project and output.
    pub target: QueryTarget,
}

/// Arguments of `codeskimr rules`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RulesQuery {
    /// Number of violations listed per rule.
    pub limit: Option<u32>,
    /// Project and output.
    pub target: QueryTarget,
}

/// Arguments of `codeskimr preview-rename`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenameQuery {
    /// Id or qualified name of the symbol.
    pub symbol: String,
    /// The name it would get.
    pub new_name: String,
    /// Number of places and text lines listed.
    pub limit: Option<u32>,
    /// Project and output.
    pub target: QueryTarget,
}

/// What `codeskimr symbols`, `usages`, `impact`, `context`, `unused`, `rules` and `preview-rename` are asked
/// to do. The
/// limits and defaults are the configured ones of the MCP tools, so the answers are the same.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryAction {
    /// Searches symbols by name.
    Symbols(SymbolsQuery),
    /// Lists where a symbol is used, or what it uses.
    Usages(UsagesQuery),
    /// Lists what a change of a symbol can reach.
    Impact(ImpactQuery),
    /// Gives signature, callers, dependencies, files and notes of a symbol.
    Context(ContextQuery),
    /// Lists the symbols for which no usage was found, with the reason.
    Unused(UnusedQuery),
    /// Judges the architecture rules of the project; fails only for new violations.
    Rules(RulesQuery),
    /// Shows what a new name for a symbol would hit; changes nothing.
    Rename(RenameQuery),
}

impl QueryAction {
    /// The project and the output form, whichever command it is.
    #[must_use]
    pub const fn target(&self) -> &QueryTarget {
        match self {
            Self::Symbols(SymbolsQuery { target, .. })
            | Self::Usages(UsagesQuery { target, .. })
            | Self::Impact(ImpactQuery { target, .. })
            | Self::Context(ContextQuery { target, .. })
            | Self::Unused(UnusedQuery { target, .. })
            | Self::Rules(RulesQuery { target, .. })
            | Self::Rename(RenameQuery { target, .. }) => target,
        }
    }
}

/// The command to run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// `codeskimr symbols`, `usages`, `impact`, `context` or `unused`.
    Query(Box<QueryAction>),
    /// `codeskimr annotations`.
    Annotations(Box<AnnotationsAction>),
    /// `codeskimr alerts`.
    Alerts(Box<AlertsAction>),
    /// `codeskimr credential`.
    Credential(CredentialAction),
    /// `codeskimr level2`.
    Level2(Level2Action),
    /// `codeskimr` without a command: start the server and open the browser, in the tray where
    /// the system has one.
    Start(StartArgs),
    /// `codeskimr scan <source>`.
    Scan(ScanArgs),
    /// `codeskimr mcp`.
    Mcp(McpArgs),
    /// `codeskimr serve`.
    Serve(ServeArgs),
    /// `codeskimr check`.
    Check(CheckArgs),
    /// `codeskimr codes`.
    Codes(CodesArgs),
    /// `codeskimr open`.
    Open(OpenArgs),
    /// `codeskimr export`.
    Export(ExportArgs),
    /// `codeskimr scope`.
    Scope(ScopeArgs),
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
