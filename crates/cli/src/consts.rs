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
    /// `{ $new }`: the proposed new name, or how many violations are new.
    pub const NEW: &str = "new";
    /// `{ $date }`: the day the state of the rules was held.
    pub const DATE: &str = "date";
    /// `{ $entries }`: how many violations the held state has.
    pub const ENTRIES: &str = "entries";
    /// `{ $held }`: how many violations the held state has, or whether one is held.
    pub const HELD: &str = "held";
    /// `{ $resolved }`: how many held violations are gone.
    pub const RESOLVED: &str = "resolved";
    /// `{ $places }`: how many places a violation has.
    pub const PLACES: &str = "places";
    /// `{ $rule }`: the id of a rule.
    pub const RULE: &str = "rule";
    /// `{ $chain }`: the way round a ring of modules.
    pub const CHAIN: &str = "chain";
    /// `{ $incoming }`: places in other modules that use a module.
    pub const INCOMING: &str = "incoming";
    /// `{ $outgoing }`: places in a module that use other modules.
    pub const OUTGOING: &str = "outgoing";
    /// `{ $whole }`: the part of a stability before the separator.
    pub const WHOLE: &str = "whole";
    /// `{ $hundredths }`: the two digits of a stability after the separator.
    pub const HUNDREDTHS: &str = "hundredths";
    /// `{ $stability }`: the stability of a module as text.
    pub const STABILITY: &str = "stability";
    /// `{ $layer }`: the layer that owns a module, as text with its brackets.
    pub const LAYER: &str = "layer";
    /// `{ $flags }`: what stands out about a module, as text.
    pub const FLAGS: &str = "flags";
    /// `{ $other }`: the module a risky module depends on.
    pub const OTHER: &str = "other";
    /// `{ $otherStability }`: the stability of that module as text.
    pub const OTHER_STABILITY: &str = "otherStability";
    /// `{ $modules }`: how many modules a ring has.
    pub const MODULES: &str = "modules";
    /// `{ $weakest }`: how many references the weakest edge of a ring has.
    pub const WEAKEST: &str = "weakest";
    /// `{ $edges }`: over how many edges a ring is resolved.
    pub const EDGES: &str = "edges";
    /// `{ $to }`: the symbol a violation must not reach; `{ $from }` is the one that refers.
    pub const TO: &str = "to";
    /// `{ $from }`: the symbol that refers.
    pub const FROM: &str = "from";
    /// `{ $verified }`: number of places that level 2 verified.
    pub const VERIFIED: &str = "verified";
    /// `{ $clash }`: whether two signatures are the same.
    pub const CLASH: &str = "clash";
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

/// Sub-command that shows or changes what a project covers (ADR 0102).
pub const COMMAND_SCOPE: &str = "scope";
/// Argument id of the project of `scope`.
pub const ARG_SCOPE_PROJECT: &str = "scope-project";
/// Flag of `scope` that leaves a language out.
pub const ARG_WITHOUT_LANGUAGE: &str = "without-language";
/// Flag of `scope` that takes a language back in.
pub const ARG_WITH_LANGUAGE: &str = "with-language";
/// Flag of `scope` that leaves a folder out.
pub const ARG_WITHOUT_FOLDER: &str = "without-folder";
/// Flag of `scope` that takes a folder back in.
pub const ARG_WITH_FOLDER: &str = "with-folder";
/// Flag of `scope` that takes everything back in.
pub const ARG_RESET: &str = "reset";
/// Placeholder shown for a folder of `scope`.
pub const VALUE_FOLDER: &str = "FOLDER";
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
/// Sub-command that judges the architecture rules of the project.
pub const COMMAND_RULES: &str = "rules";
/// Sub-command that shows the modules of the project with cycles, coupling and stability.
pub const COMMAND_MODULES: &str = "modules";
/// Sub-command that shows what a new name for a symbol would hit.
pub const COMMAND_PREVIEW_RENAME: &str = "preview-rename";
/// Argument id of the new name of `preview-rename`.
pub const ARG_QUERY_NEW_NAME: &str = "query-new-name";
/// Placeholder shown for the new name.
pub const VALUE_QUERY_NEW_NAME: &str = "NEW_NAME";
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
/// Option of `unused`: only symbols with this reason.
pub const ARG_REASON: &str = "reason";
/// Placeholder shown for the reason.
pub const VALUE_REASON: &str = "REASON";
/// The reasons the command line accepts.
pub const REASONS: [&str; 4] = ["private", "internal", "public", "approximate"];
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
/// Option of `modules`: the depending module of the dependency whose places are listed.
pub const ARG_MODULE_FROM: &str = "from";
/// Option of `modules`: the module that is depended on.
pub const ARG_MODULE_TO: &str = "to";
/// Value name of the module options of `modules`.
pub const VALUE_MODULE: &str = "MODULE";
/// Flag of `usages`: references from tests count.
pub const ARG_INCLUDE_TESTS: &str = "include-tests";
/// Long flag of `rules` that holds the state.
pub const ARG_HOLD: &str = "hold";
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
    /// The line of code a place of `preview-rename` stands on.
    pub const SITE_CODE: &str = "code";
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
    /// The places of `preview-rename`.
    pub const SITES: &str = "sites";
    /// The name conflicts of `preview-rename`.
    pub const CONFLICTS: &str = "conflicts";
    /// The text lines of `preview-rename`.
    pub const TEXT_HITS: &str = "text_hits";
    /// The proposed name of `preview-rename`.
    pub const NEW_NAME: &str = "new_name";
    /// Whether the text of the project folder was searched.
    pub const SEARCHED: &str = "searched";
    /// Whether the symbol is public and used by code the answer cannot see.
    pub const PUBLIC_INTERFACE: &str = "public_interface";
    /// Places that level 2 verified.
    pub const VERIFIED: &str = "verified";
    /// Files that hold places.
    pub const FILES: &str = "files";
    /// Whether a name conflict has the same signature.
    pub const SAME_SIGNATURE: &str = "same_signature";
    /// The rules of `rules`.
    pub const RULES: &str = "rules";
    /// The day the state was held.
    pub const HELD_ON: &str = "held_on";
    /// How many violations the held state has.
    pub const ENTRIES: &str = "entries";
    /// The violations of a rule.
    pub const VIOLATIONS: &str = "violations";
    /// The places of a violation.
    pub const PLACES: &str = "places";
    /// The ring of modules behind a cycle violation.
    pub const RING: &str = "ring";
    /// Every module of a ring.
    pub const MODULES: &str = "modules";
    /// A way round a ring.
    pub const CHAIN: &str = "chain";
    /// How many references the weakest edge of a ring has.
    pub const WEAKEST_COUNT: &str = "weakest_count";
    /// Over how many edges a ring is resolved.
    pub const RESOLUTION_EDGES: &str = "resolution_edges";
    /// Whether a violation is an old burden, or how many there are.
    pub const OLD: &str = "old";
    /// How many violations are new.
    pub const NEW: &str = "new";
    /// How many held violations are gone.
    pub const RESOLVED: &str = "resolved";
    /// The symbol a violation must not reach.
    pub const TO: &str = "to";
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
    /// Field of the answer about modules: places in other modules that use one.
    pub const INCOMING: &str = "incoming";
    /// Field of the answer about modules: places in a module that use other modules.
    pub const OUTGOING: &str = "outgoing";
    /// Field of the answer about modules: stability in thousandths, `null` without a dependency.
    pub const STABILITY: &str = "stability";
    /// Field of the answer about modules: whether a module is part of a cycle.
    pub const IN_CYCLE: &str = "in_cycle";
    /// Field of the answer about modules: the rings.
    pub const CYCLES: &str = "cycles";
    /// Field of the answer about modules: the modules that stand out.
    pub const RISKS: &str = "risks";
    /// Field of a ring: the dependencies to remove to end it.
    pub const RESOLUTION: &str = "resolution";
    /// Field of a violation: the dependency.
    pub const EDGE: &str = "edge";
    /// Field of a risk: the risky module.
    pub const MODULE: &str = "module";
    /// Field of a risk: the less stable module it depends on.
    pub const DEPENDS_ON: &str = "depends_on";
    /// Field of a risk: the stability of that module.
    pub const DEPENDS_ON_STABILITY: &str = "depends_on_stability";
    /// Field of a violation: the layer of the depending module.
    pub const FROM_LAYER: &str = "from_layer";
    /// Field of a violation: the layer of the module it depends on.
    pub const TO_LAYER: &str = "to_layer";
    /// Field of a module: its name.
    pub const NAME: &str = "name";
}

/// The first line of `change --markdown`: a script that updates one comment on a pull request
/// finds its own comment by it.
pub const MARKDOWN_MARKER: &str = "<!-- codeskimr-change -->";
/// The first line of `compare --metrics --markdown`, so that a bot can find its own comment.
pub const METRICS_MARKDOWN_MARKER: &str = "<!-- codeskimr-metrics -->";
/// Sub-command that lists the retained scans of a project.
pub const COMMAND_SCANS: &str = "scans";
/// Sub-command that compares two scans of a project.
pub const COMMAND_COMPARE: &str = "compare";
/// Sub-command that shows what the difference between two scans hits.
pub const COMMAND_CHANGE: &str = "change";
/// Option of `compare` and `change`: the older scan.
pub const ARG_BASE: &str = "base";
/// Option of `compare` and `change`: the newer scan.
pub const ARG_SCAN_TARGET: &str = "target";
/// Placeholder shown for the id of a scan.
pub const VALUE_SNAPSHOT: &str = "SNAPSHOT";
/// Flag of `change`: the answer as Markdown.
pub const ARG_MARKDOWN: &str = "markdown";
/// Flag of `compare`: the metric changes instead of the changed symbols (ADR 0114).
pub const ARG_METRICS: &str = "metrics";
/// Flag of `compare --metrics`: only what got worse.
pub const ARG_WORSE: &str = "worse";
/// Flag of `compare --metrics`: only what got better.
pub const ARG_BETTER: &str = "better";
/// Option of `compare --metrics`: only objects in which one of these metrics moved.
pub const ARG_METRIC: &str = "metric";
/// Option of `compare --metrics`: the level to list.
pub const ARG_LEVEL: &str = "level";
/// Placeholder shown for the list of metrics.
pub const VALUE_METRICS: &str = "METRICS";
/// Placeholder shown for the level.
pub const VALUE_LEVEL: &str = "LEVEL";

/// The names the text of `compare --metrics` reads from the JSON of the answer and passes to the
/// texts (ADR 0114).
pub mod metrics {
    /// The field of the answer that lists what got worse (a key of the JSON, not a verdict).
    pub const WORSE: &str = "worse";
    /// The field of the answer that lists what got better.
    pub const BETTER: &str = "better";
    /// The level of the answer.
    pub const LEVEL: &str = "level";
    /// How many objects the newer scan has on the level.
    pub const LOOKED_AT: &str = "looked_at";
    /// The moves of one object.
    pub const MOVES: &str = "moves";
    /// The metric of a move.
    pub const METRIC: &str = "metric";
    /// The value before a move.
    pub const FROM: &str = "from";
    /// The value after a move.
    pub const TO: &str = "to";
    /// Whether a move crossed the bound of the rating, and which way.
    pub const CROSSING: &str = "crossing";
    /// `{ $metric }` and the other arguments of the texts of a move.
    pub const ARG_METRIC: &str = "metric";
    /// `{ $from }`.
    pub const ARG_FROM: &str = "from";
    /// `{ $to }`.
    pub const ARG_TO: &str = "to";
    /// `{ $move }`.
    pub const ARG_MOVE: &str = "move";
    /// `{ $crossing }`.
    pub const ARG_CROSSING: &str = "crossing";
    /// `{ $moves }`.
    pub const ARG_MOVES: &str = "moves";
    /// `{ $level }`.
    pub const ARG_LEVEL: &str = "level";
    /// `{ $worse }`.
    pub const ARG_WORSE: &str = "worse";
    /// `{ $better }`.
    pub const ARG_BETTER: &str = "better";
    /// `{ $looked }`.
    pub const ARG_LOOKED: &str = "looked";
    /// Separates the moves of one object.
    pub const MOVE_JOIN: &str = ", ";
    /// Values are shown with this many hundredths of precision: the value is multiplied by it,
    /// rounded and divided again.
    pub const ROUNDING: f64 = 100.0;
}

/// Separates the names after `--metric`, so that `--metric cc,mi` names two metrics.
pub const METRIC_LIST_DELIMITER: char = ',';
/// What a missing `layer` or `signature` is in the JSON.
pub const JSON_NULL_TEXT: &str = "null";
/// What a control character in a text of the project becomes.
pub const CONTROL_REPLACEMENT: char = ' ';
/// Puts the entries of a list below its heading.
pub const LIST_INDENT: &str = "  ";
/// The indent of a list whose lines start at the left edge, as in Markdown.
pub const NO_INDENT: &str = "";
/// Between the flags of one module in the text of `modules`.
pub const FLAG_JOIN: &str = ", ";
/// Before the flags of a module in the text of `modules`.
pub const FLAG_PREFIX: &str = "  ";
/// Before the layer of a module in the text of `modules`.
pub const LAYER_PREFIX: &str = " ";
/// Thousandths in one hundredth, the step the text of `modules` shows.
pub const STABILITY_STEP: u64 = 10;
/// Half a step, added before the division so that the hundredths are rounded like in the UI.
pub const STABILITY_HALF_STEP: u64 = 5;
/// Hundredths in one whole.
pub const STABILITY_HUNDRED: u64 = 100;
/// Digits after the separator in the text of a stability.
pub const STABILITY_DIGITS: usize = 2;
/// Joins the modules of the way round a ring in a line of the terminal.
pub const CHAIN_JOIN: &str = " -> ";
/// Separates the title of an annotation from its text.
pub const TITLE_SEPARATOR: &str = " ";

/// The `kind` of a place of `preview-rename` that is a declaration, not a usage.
pub const KIND_DECLARATION: &str = "declaration";

/// Names, fields and numbers of the texts of `scans`, `compare` and `change` (ADR 0097).
pub mod change {
    /// The names of the arguments of the texts of this module.
    pub mod name {
        pub const COUNT: &str = "count";
        pub const KEEP: &str = "keep";
        pub const AVERAGE: &str = "average";
        pub const SNAPSHOT: &str = "snapshot";
        pub const CREATED: &str = "created";
        pub const MODE: &str = "mode";
        pub const FILES: &str = "files";
        pub const CHANGED: &str = "changed";
        pub const DURATION: &str = "duration";
        pub const COMMIT: &str = "commit";
        pub const MS: &str = "ms";
        pub const STEP: &str = "step";
        pub const STEPS: &str = "steps";
        pub const BASE: &str = "base";
        pub const BASE_TIME: &str = "base_time";
        pub const TARGET: &str = "target";
        pub const TARGET_TIME: &str = "target_time";
        pub const ADDED: &str = "added";
        pub const REMOVED: &str = "removed";
        pub const MOVED: &str = "moved";
        pub const KIND: &str = "kind";
        pub const NAME: &str = "name";
        pub const PATH: &str = "path";
        pub const REASONS: &str = "reasons";
        pub const FROM: &str = "from";
        pub const TO: &str = "to";
        pub const OUTCOME: &str = "outcome";
        pub const PUBLIC: &str = "public";
        pub const CERTAIN: &str = "certain";
        pub const PROBABLE: &str = "probable";
        pub const OPEN: &str = "open";
        pub const CAUSE: &str = "cause";
        pub const LINE: &str = "line";
        pub const STATUS: &str = "status";
        pub const SYMBOL: &str = "symbol";
        pub const HOLDER: &str = "holder";
        pub const NOTE: &str = "note";
    }
    /// The fields of the answer that only this module reads.
    pub mod field_name {
        pub const SCANS: &str = "scans";
        pub const KEEP: &str = "keep";
        pub const AVERAGE: &str = "average_ms";
        pub const SNAPSHOT: &str = "snapshot";
        pub const CREATED: &str = "created";
        pub const MODE: &str = "mode";
        pub const FILES: &str = "files";
        pub const FILES_CHANGED: &str = "files_changed";
        pub const DURATION: &str = "duration_ms";
        pub const STEP: &str = "step";
        pub const STEPS: &str = "steps";
        pub const COMMIT: &str = "commit";
        pub const BASE: &str = "base";
        pub const TARGET: &str = "target";
        pub const ADDED: &str = "added";
        pub const REMOVED: &str = "removed";
        pub const CHANGED: &str = "changed";
        pub const MOVED: &str = "moved";
        pub const REASONS: &str = "reasons";
        pub const NARROWED: &str = "narrowed";
        pub const CRITICAL: &str = "critical";
        pub const OUTCOME: &str = "outcome";
        pub const LAYERS: &str = "layers";
        pub const NEW_VIOLATIONS: &str = "new_violations";
        pub const NEW_CYCLES: &str = "new_cycles";
        pub const TO: &str = "to";
        pub const SUMMARY: &str = "summary";
        pub const CHANGED_SYMBOLS: &str = "changed_symbols";
        pub const PUBLIC_SYMBOLS: &str = "public_symbols";
        pub const USAGES_CERTAIN: &str = "usages_certain";
        pub const USAGES_PROBABLE: &str = "usages_probable";
        pub const USAGES_NOT_TOUCHED: &str = "usages_not_touched";
        pub const IMPACT: &str = "impact";
        pub const CAUSE: &str = "cause";
        pub const IS_CUT: &str = "is_cut";
        pub const STATUS: &str = "status";
        pub const SYMBOL: &str = "symbol";
        pub const BASE_SYMBOL: &str = "base";
        pub const TARGET_SYMBOL: &str = "target";
    }

    pub const SECONDS_PER_DAY: i64 = 86_400;
    pub const SECONDS_PER_HOUR: i64 = 3_600;
    pub const SECONDS_PER_MINUTE: i64 = 60;
    pub const REASON_SEPARATOR: &str = ", ";
    /// The `cause` of a symbol that is gone, as the answer says it.
    pub const REMOVED_CAUSE: &str = "removed";
    /// The `status` of a usage whose holder is untouched, as the answer says it.
    pub const NOT_TOUCHED_STATUS: &str = "not_touched";
    pub const BACKTICK: char = '`';
    pub const APOSTROPHE: &str = "'";
    pub const ERA_DAYS: i64 = 146_097;
    pub const EPOCH_SHIFT: i64 = 719_468;
    pub const YEAR_DAYS: i64 = 365;
    pub const FOUR_YEARS_DAYS: i64 = 1_460;
    pub const CENTURY_DAYS: i64 = 36_524;
    pub const MONTH_CYCLE_MONTHS: i64 = 5;
    pub const MONTH_CYCLE_DAYS: i64 = 153;
    pub const MONTH_OFFSET: i64 = 2;
    pub const MARCH_BASED_NEW_YEAR: i64 = 10;
    pub const MARCH: i64 = 3;
    pub const MONTHS_SHIFT_BACK: i64 = 9;
    pub const YEARS_PER_ERA: i64 = 400;
    pub const LEAP_CYCLE: i64 = 4;
    pub const CENTURY: i64 = 100;
    /// Number of list sections of `compare`.
    pub const LIST_SECTIONS: usize = 4;
    /// Number of arguments of the summary text.
    pub const SUMMARY_ARGUMENTS: usize = 7;
    /// Separates year, month and day.
    pub const DATE_SEPARATOR: &str = "-";
    /// Separates the date from the time.
    pub const DATE_TIME_SEPARATOR: &str = " ";
    /// Separates hour and minute.
    pub const TIME_SEPARATOR: &str = ":";
    /// Digits of a year.
    pub const YEAR_WIDTH: usize = 4;
    /// Digits of month, day, hour and minute.
    pub const PAIR_WIDTH: usize = 2;
    /// Fills a number up to its width.
    pub const ZERO_DIGIT: &str = "0";
}
