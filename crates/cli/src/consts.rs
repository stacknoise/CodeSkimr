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
/// Flag of `open` that prints the address with the one-time code (ADR 0088).
pub const ARG_PRINT_URL: &str = "print-url";

/// Flag of the bare start that keeps the program in the console window.
pub const ARG_CONSOLE: &str = "console";

/// Sub-command that exports a project.
pub const COMMAND_EXPORT: &str = "export";
/// Argument id of the project of `export`.
pub const ARG_EXPORT_PROJECT: &str = "export-project";
/// Placeholder shown for the project of `export`.
pub const VALUE_EXPORT_PROJECT: &str = "PROJECT";
/// Argument id and long flag of the HTML file of `export`.
pub const ARG_HTML: &str = "html";

/// Sub-command that allows or forbids level 2 for a project.
pub const COMMAND_LEVEL2: &str = "level2";
/// Sub-command of `level2` that allows the indexers.
pub const COMMAND_LEVEL2_CONFIRM: &str = "confirm";
/// Sub-command of `level2` that forbids them.
pub const COMMAND_LEVEL2_REVOKE: &str = "revoke";
/// Argument id of the project of `level2`.
pub const ARG_LEVEL2_PROJECT: &str = "level2-project";
/// Flag of `level2 confirm` that marks a window the server opened for the page in the browser
/// (ADR 0071, decision 5). It changes what the window shows, never what it asks.
pub const ARG_LEVEL2_FROM_UI: &str = "from-ui";

/// Sub-command that manages credentials for Git sources.
pub const COMMAND_CREDENTIAL: &str = "credential";
/// Sub-command of `credential` that stores a credential.
pub const COMMAND_CREDENTIAL_SET: &str = "set";
/// Sub-command of `credential` that removes a credential.
pub const COMMAND_CREDENTIAL_DELETE: &str = "delete";
/// Sub-command of `credential` that signs in with the OAuth device flow.
pub const COMMAND_CREDENTIAL_LOGIN: &str = "login";
/// Argument id of the credential name.
pub const ARG_ID: &str = "id";
/// Placeholder shown for the credential name.
pub const VALUE_ID: &str = "ID";
/// Argument id and long flag of the user name of a credential.
pub const ARG_USER: &str = "user";
/// Placeholder shown for the user name.
pub const VALUE_USER: &str = "USER";
/// Argument id and long flag of the platform.
pub const ARG_PROVIDER: &str = "provider";
/// Placeholder shown for the platform.
pub const VALUE_PROVIDER: &str = "PROVIDER";
/// Value of `--provider` for GitHub.
pub const PROVIDER_GITHUB: &str = "github";
/// Value of `--provider` for GitLab.
pub const PROVIDER_GITLAB: &str = "gitlab";
/// Argument id and long flag of the OAuth application id.
pub const ARG_CLIENT_ID: &str = "client-id";
/// Placeholder shown for the OAuth application id.
pub const VALUE_CLIENT_ID: &str = "CLIENT-ID";
/// Argument id and long flag of the address of a self-managed installation.
pub const ARG_HOST: &str = "host";
/// Placeholder shown for the address of a self-managed installation.
pub const VALUE_HOST: &str = "URL";
/// Argument id and long flag of the credential of a Git source.
pub const ARG_CREDENTIAL: &str = "credential";
/// Argument id and long flag of the display name of a project (`scan --name`).
pub const ARG_NAME: &str = "name";
/// Placeholder shown for the display name.
pub const VALUE_NAME: &str = "NAME";

/// Sub-command that lists the error codes or explains one.
pub const COMMAND_CODES: &str = "codes";
/// Argument id of the code that `codes` explains.
pub const ARG_CODE: &str = "code";
/// Placeholder shown for the code that `codes` explains.
pub const VALUE_CODE: &str = "CODE";
/// Select keys of the Fluent message that explains a code.
pub const SELECT_YES: &str = "yes";
/// Select key of a code that cannot succeed on a retry.
pub const SELECT_NO: &str = "no";

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
/// Start of a long flag on the command line.
pub const LONG_FLAG_PREFIX: &str = "--";

/// German, the second language of the texts.
pub const LANGUAGE_DE: &str = "de";
/// English, the default language of the texts.
pub const LANGUAGE_EN: &str = "en";

/// Number of decimals of the seconds in the scan summary.
pub const SECONDS_DECIMALS: i32 = 2;
/// Factor that moves `SECONDS_DECIMALS` decimals in front of the point.
pub const SECONDS_BASE: f64 = 10.0;

/// Name of the command that handles annotations.
pub const COMMAND_ANNOTATIONS: &str = "annotations";
/// Name of the command that handles alerts.
pub const COMMAND_ALERTS: &str = "alerts";
/// Sub-command of `annotations` and `alerts`: list.
pub const COMMAND_LIST: &str = "list";
/// Sub-command of `annotations`: add one.
pub const COMMAND_ADD: &str = "add";
/// Sub-command of `annotations`: change one.
pub const COMMAND_UPDATE: &str = "update";
/// Sub-command of `annotations`: delete one.
pub const COMMAND_REMOVE: &str = "remove";
/// Sub-command of `alerts`: dismiss one.
pub const COMMAND_DISMISS: &str = "dismiss";
/// Sub-command of `alerts`: dismiss all of a filter.
pub const COMMAND_DISMISS_ALL: &str = "dismiss-all";
/// Argument: the project, a number or the folder or Git URL it was scanned from.
pub const ARG_NOTE_PROJECT: &str = "note-project";
/// Value name of the project argument.
pub const VALUE_NOTE_PROJECT: &str = "PROJECT";
/// Argument: the id of an annotation.
pub const ARG_ANNOTATION: &str = "annotation";
/// Argument: the id of an alert.
pub const ARG_ALERT: &str = "alert";
/// Value name of an id.
pub const VALUE_NUMBER: &str = "ID";
/// Option: the qualified name of a symbol.
pub const ARG_SYMBOL: &str = "symbol";
/// Value name of the symbol.
pub const VALUE_SYMBOL: &str = "NAME";
/// Option: the kind of a symbol.
pub const ARG_SYMBOL_KIND: &str = "symbol-kind";
/// Value name of the kind of a symbol.
pub const VALUE_SYMBOL_KIND: &str = "KIND";
/// Option: a parameter type of a symbol.
pub const ARG_PARAM: &str = "param";
/// Value name of a parameter type.
pub const VALUE_PARAM: &str = "TYPE";
/// Option: the kind of an annotation.
pub const ARG_KIND: &str = "kind";
/// Value name of the kind of an annotation.
pub const VALUE_KIND: &str = "KIND";
/// Option: the status of a task.
pub const ARG_STATUS: &str = "status";
/// Value name of a status.
pub const VALUE_STATUS: &str = "STATUS";
/// Option: the title of a task.
pub const ARG_TITLE: &str = "title";
/// Value name of a title.
pub const VALUE_TITLE: &str = "TITLE";
/// Option: the text of an annotation.
pub const ARG_TEXT: &str = "text";
/// Value name of a text.
pub const VALUE_TEXT: &str = "TEXT";
/// Option: the due date of a task.
pub const ARG_DUE: &str = "due";
/// Value name of the due date.
pub const VALUE_DUE: &str = "SECONDS";
/// Option: the assignee of a task.
pub const ARG_ASSIGNEE: &str = "assignee";
/// Value name of the assignee.
pub const VALUE_ASSIGNEE: &str = "NAME";
/// Flag: remove the text.
pub const ARG_CLEAR_TEXT: &str = "clear-text";
/// Flag: remove the due date.
pub const ARG_CLEAR_DUE: &str = "clear-due";
/// Flag: remove the assignee.
pub const ARG_CLEAR_ASSIGNEE: &str = "clear-assignee";
/// Option: rows to skip.
pub const ARG_OFFSET: &str = "offset";
/// Option: rows to return.
pub const ARG_ROWS: &str = "limit";
/// Value name of a row count.
pub const VALUE_ROWS: &str = "N";
/// Option: the state of an alert.
pub const ARG_STATE: &str = "state";
/// Value name of a state.
pub const VALUE_STATE: &str = "STATE";
/// Flag: the confirmation of `dismiss-all`.
pub const ARG_YES: &str = "yes";
/// The kinds of annotations the command line accepts.
pub const ANNOTATION_KINDS: [&str; 3] = ["critical", "note", "task"];
/// The statuses of a task the command line accepts.
pub const TASK_STATUSES: [&str; 3] = ["open", "in_progress", "done"];
/// The states of an alert the command line accepts.
pub const ALERT_STATES: [&str; 2] = ["open", "dismissed"];
/// The kinds of alerts the command line accepts.
pub const ALERT_KINDS: [&str; 3] = ["changed", "removed", "orphaned"];

/// Names of the Fluent arguments of the messages.
pub mod arg {
    /// `{ $place }`: the folder or Git address of a project.
    pub const PLACE: &str = "place";
    /// `{ $branch }`: the Git branch of a project.
    pub const BRANCH: &str = "branch";
    /// `{ $project }`: the project a message is about.
    pub const PROJECT: &str = "project";
    /// `{ $language }`: name of a language.
    pub const LANGUAGE: &str = "language";
    /// `{ $count }`: how many times something happened.
    pub const COUNT: &str = "count";
    /// `{ $from_layer }`: the layer of a function that breaks the layer rule.
    pub const FROM_LAYER: &str = "from_layer";
    /// `{ $to_layer }`: the layer that function must not use.
    pub const TO_LAYER: &str = "to_layer";
    /// `{ $stale }`: files that changed since the scan.
    pub const STALE: &str = "stale";
    /// `{ $reason }`: why level 2 was skipped, or why a symbol counts as unused.
    pub const REASON: &str = "reason";
    /// `{ $assessment }`: how much "no usage found" says about a symbol.
    pub const ASSESSMENT: &str = "assessment";
    /// `{ $group }`: the id prefix of the group of an error code.
    pub const GROUP: &str = "group";
    /// `{ $description }`: the English developer description of an error code.
    pub const DESCRIPTION: &str = "description";
    /// `{ $severity }`: `error`, `warning` or `info`.
    pub const SEVERITY: &str = "severity";
    /// `{ $retryable }`: `yes` or `no`.
    pub const RETRYABLE: &str = "retryable";
    /// `{ $since }`: the first release that has the code.
    pub const SINCE: &str = "since";
    /// `{ $code }`: the error code id.
    pub const CODE: &str = "code";
    /// `{ $message }`: the user-facing text of an error.
    pub const MESSAGE: &str = "message";
    /// `{ $path }`: the source being scanned.
    pub const PATH: &str = "path";
    /// `{ $files }`: number of scanned files.
    pub const FILES: &str = "files";
    /// `{ $stage }`: the label of a stage of a scan.
    pub const STAGE: &str = "stage";
    /// `{ $stages }`: the times of the stages of a scan, one text per stage.
    pub const STAGES: &str = "stages";
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
    /// `{ $log }`: path of the log file.
    pub const LOG: &str = "log";
    /// `{ $url }`: address of the local server.
    pub const URL: &str = "url";
    /// `{ $line }`: one-based line of a finding.
    pub const LINE: &str = "line";
    /// `{ $name }`: qualified name of a function.
    pub const NAME: &str = "name";
    /// `{ $lines }`: code lines of a function.
    pub const LINES: &str = "lines";
    /// A count that is above its maximum.
    pub const VALUE: &str = "value";
    /// `{ $kind }`: the kind of a symbol, of a reference or of an annotation.
    pub const KIND: &str = "kind";
    /// `{ $column }`: one-based column of a usage.
    pub const COLUMN: &str = "column";
    /// `{ $confidence }`: `certain` or `probable`.
    pub const CONFIDENCE: &str = "confidence";
    /// `{ $depth }`: the step of a walk a symbol was reached in.
    pub const DEPTH: &str = "depth";
    /// `{ $shown }`: how many entries of a list are shown.
    pub const SHOWN: &str = "shown";
    /// `{ $total }`: how many entries a list has in all.
    pub const TOTAL: &str = "total";
    /// `{ $text }`: a text that was taken from the project as it is.
    pub const TEXT: &str = "text";
    /// `{ $limit }`: the configured maximum.
    pub const LIMIT: &str = "limit";
    /// `{ $percent }`: similarity in percent.
    pub const PERCENT: &str = "percent";
    /// `{ $original }`: qualified name of the function that was copied.
    pub const ORIGINAL: &str = "original";
    /// `{ $findings }`: number of findings.
    pub const FINDINGS: &str = "findings";
    /// `{ $id }`: name of a credential.
    pub const ID: &str = "id";
    /// `{ $uri }`: page where a sign-in code is entered.
    pub const URI: &str = "uri";
    /// `{ $user_code }`: the code that the user enters on that page.
    pub const USER_CODE: &str = "user_code";
    /// `{ $codes }`: error codes, separated by `CODE_LIST_SEPARATOR`.
    pub const CODES: &str = "codes";
    /// `{ $kilobytes }`: size of an export file.
    pub const KILOBYTES: &str = "kilobytes";
    /// `{ $tool }`: name of a tool the project needs.
    pub const TOOL: &str = "tool";
    /// `{ $languages }`: the languages that need the tool, separated by `LIST_SEPARATOR`.
    pub const LANGUAGES: &str = "languages";
    /// `{ $found }`: the version of the tool that is installed.
    pub const FOUND: &str = "found";
    /// `{ $required }`: the version the project requires.
    pub const REQUIRED: &str = "required";
    /// `{ $purpose }`: what the tool is needed for.
    pub const PURPOSE: &str = "purpose";
    /// `{ $install }`: how to install the tool.
    pub const INSTALL: &str = "install";
    /// `{ $seconds }` in the launch message is the same name as the scan duration.
    pub const LAUNCH_SECONDS: &str = SECONDS;
}

/// Separates the lines of a multi-line message.
pub const LINE_BREAK: &str = "\n";

/// Separates the languages in the line of a tool.
pub const LIST_SEPARATOR: &str = ", ";

/// Stands in for a version that is not known.
pub const NO_VERSION: &str = "?";

/// Separates the codes in a list of codes.
pub const CODE_LIST_SEPARATOR: &str = ", ";

/// Sub-command that searches symbols by name.
pub const COMMAND_SYMBOLS: &str = "symbols";
/// Sub-command that lists the usages of a symbol.
pub const COMMAND_USAGES: &str = "usages";
/// Sub-command that lists what a change of a symbol can reach.
pub const COMMAND_IMPACT: &str = "impact";
/// Sub-command that gives the context of a symbol in one answer.
pub const COMMAND_CONTEXT: &str = "context";
/// Sub-command that lists the symbols without a usage.
pub const COMMAND_UNUSED: &str = "unused";
/// Option of `unused`: only symbols of this language.
pub const ARG_LANGUAGE: &str = "language";
/// Placeholder shown for the language.
pub const VALUE_LANGUAGE: &str = "LANGUAGE";
/// Option of `unused`: only symbols with this assessment.
pub const ARG_ASSESSMENT: &str = "assessment";
/// Placeholder shown for the assessment.
pub const VALUE_ASSESSMENT: &str = "ASSESSMENT";
/// The assessments the command line accepts.
pub const ASSESSMENTS: [&str; 3] = ["likely", "public", "approximate"];
/// The groups of kinds `unused` accepts.
pub const UNUSED_KINDS: [&str; 3] = ["all", "types", "functions"];
/// Argument id of the search text of `symbols`.
pub const ARG_QUERY_TEXT: &str = "query-text";
/// Placeholder shown for the search text.
pub const VALUE_QUERY_TEXT: &str = "QUERY";
/// Argument id of the symbol of `usages`, `impact` and `context`.
pub const ARG_QUERY_SYMBOL: &str = "query-symbol";
/// Placeholder shown for that symbol.
pub const VALUE_QUERY_SYMBOL: &str = "SYMBOL";
/// Argument id of the project of the query commands; the long flag is [`ARG_PROJECT`].
pub const ARG_QUERY_PROJECT: &str = "query-project";
/// Placeholder shown for it.
pub const VALUE_QUERY_PROJECT: &str = "ID|FOLDER";
/// Option of `usages`: which way the walk goes.
pub const ARG_DIRECTION: &str = "direction";
/// Placeholder shown for the direction.
pub const VALUE_DIRECTION: &str = "DIRECTION";
/// The directions the command line accepts.
pub const DIRECTIONS: [&str; 2] = ["incoming", "outgoing"];
/// Option of `usages`: how many steps the walk goes.
pub const ARG_DEPTH: &str = "depth";
/// Flag of `usages`: references from tests count.
pub const ARG_INCLUDE_TESTS: &str = "include-tests";
/// Option of `context`: how much room the answer takes.
pub const ARG_BUDGET: &str = "budget";
/// Placeholder shown for the budget.
pub const VALUE_BUDGET: &str = "BUDGET";
/// The budgets the command line accepts.
pub const BUDGETS: [&str; 3] = ["short", "medium", "long"];
/// Version of the JSON of the query commands (`schema`); it grows when a field changes meaning or
/// goes away.
pub const QUERY_SCHEMA_VERSION: u32 = 1;

/// The names of the fields of the answers of the query commands, which are those of the MCP tools.
pub mod key {
    /// Version of the format; added to every answer.
    pub const SCHEMA: &str = "schema";
    /// Whether files changed since the scan; added to every answer.
    pub const STALE: &str = "stale";
    /// The registered code of a failure.
    pub const CODE: &str = "code";
    /// The failure of a command.
    pub const ERROR: &str = "error";
    /// An answer that is not an object, as a field.
    pub const RESULT: &str = "result";
    /// The hits of `symbols`.
    pub const HITS: &str = "hits";
    /// The usages of `usages`.
    pub const USAGES: &str = "usages";
    /// The symbols of `impact`.
    pub const SYMBOLS: &str = "symbols";
    /// The symbol of `context`.
    pub const SYMBOL: &str = "symbol";
    /// The symbols without a usage of `unused`.
    pub const UNUSED: &str = "unused";
    /// The symbols the exception list takes out, in `unused`.
    pub const LEFT_OUT: &str = "left_out";
    /// The counts of `unused`.
    pub const COUNTS: &str = "counts";
    /// The entry of the exception list that takes a symbol out.
    pub const RULE: &str = "rule";
    /// Why a symbol counts as unused.
    pub const REASON: &str = "reason";
    /// How much "no usage found" says about a symbol.
    pub const ASSESSMENT: &str = "assessment";
    /// The callers of `context`.
    pub const CALLERS: &str = "callers";
    /// The dependencies of `context`.
    pub const DEPENDENCIES: &str = "dependencies";
    /// The files of `context`.
    pub const AFFECTED_FILES: &str = "affected_files";
    /// The annotations of `context`.
    pub const ANNOTATIONS: &str = "annotations";
    /// The entries of a list of `context`.
    pub const ITEMS: &str = "items";
    /// The number of entries a list has in all.
    pub const TOTAL: &str = "total";
    /// The notice that a list was cut.
    pub const NOTICE: &str = "notice";
    /// Whether an answer rests on unchecked references.
    pub const PRELIMINARY: &str = "preliminary";
    /// The id of a symbol.
    pub const ID: &str = "id";
    /// The kind of a symbol, a reference or an annotation.
    pub const KIND: &str = "kind";
    /// The qualified name of a symbol.
    pub const QUALIFIED_NAME: &str = "qualified_name";
    /// The file of a symbol or a usage.
    pub const PATH: &str = "path";
    /// The one-based line of a usage.
    pub const LINE: &str = "line";
    /// The first line of a symbol.
    pub const FIRST_LINE: &str = "first_line";
    /// The one-based column of a usage.
    pub const COLUMN: &str = "column";
    /// `certain` or `probable`.
    pub const CONFIDENCE: &str = "confidence";
    /// The step of a walk.
    pub const DEPTH: &str = "depth";
    /// The symbol that contains a usage.
    pub const FROM: &str = "from";
    /// The number of references of a dependency.
    pub const COUNT: &str = "count";
    /// The signature of a symbol.
    pub const SIGNATURE: &str = "signature";
    /// The layer of a symbol.
    pub const LAYER: &str = "layer";
    /// The documentation of a symbol.
    pub const DOC: &str = "doc";
    /// The title of an annotation.
    pub const TITLE: &str = "title";
    /// The text of an annotation.
    pub const TEXT: &str = "text";
}

/// What a missing `layer` or `signature` is in the JSON.
pub const JSON_NULL_TEXT: &str = "null";
/// What a control character in a text of the project becomes.
pub const CONTROL_REPLACEMENT: char = ' ';
/// Puts the entries of a list below its heading.
pub const LIST_INDENT: &str = "  ";
/// Separates the title of an annotation from its text.
pub const TITLE_SEPARATOR: &str = " ";
