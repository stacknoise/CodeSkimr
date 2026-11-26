//! Fixed texts of the application layer.

/// Label of `StageKind::Fetch`.
pub const STAGE_FETCH: &str = "fetch";
/// Label of `StageKind::Discover`.
pub const STAGE_DISCOVER: &str = "discover";
/// Label of `StageKind::Analyze`.
pub const STAGE_ANALYZE: &str = "analyze";
/// Label of `StageKind::Resolve`.
pub const STAGE_RESOLVE: &str = "resolve";
/// Label of `StageKind::Measure`.
pub const STAGE_MEASURE: &str = "measure";
/// Label of `StageKind::Persist`.
pub const STAGE_PERSIST: &str = "persist";
/// The stage labels in execution order, the same order as `StageKind::ALL`.
pub const STAGE_LABELS: [&str; 6] = [
    STAGE_FETCH,
    STAGE_DISCOVER,
    STAGE_ANALYZE,
    STAGE_RESOLVE,
    STAGE_MEASURE,
    STAGE_PERSIST,
];

/// Separator of the parts of a project-relative path.
pub const PATH_SEPARATOR: char = '/';
/// The separator Windows users type; it is turned into [`PATH_SEPARATOR`].
pub const WINDOWS_SEPARATOR: char = '\\';
/// Name of the current folder; it selects the whole project.
pub const CURRENT_DIR: &str = ".";
/// Prefix that names the current folder and is dropped from a requested path.
pub const CURRENT_DIR_PREFIX: &str = "./";
/// First id handed out for the throw-away ids of a check; nothing is stored.
pub const CHECK_FIRST_ID: u64 = 1;
/// How many stored functions a check asks for per function it compares against the project; a
/// few more than one, so that hits in the checked files themselves (their stored, older versions)
/// and in tests do not hide a real original.
pub const STORED_DUPLICATE_HITS: u32 = 20;
/// Snapshot id used for the throw-away analysis of a check; nothing is stored.
pub const CHECK_SNAPSHOT_ID: u64 = 0;
/// The words in a comment directly above a pure mapping function that exempt it from `CHK-102`
/// (ADR 0070). A reason must follow.
pub const CHECK_ALLOW_MARKER: &str = "codeskimr: allow CHK-102";
/// How many lines above its declaration a function is searched for the marker, attributes and
/// doc comments included.
pub const CHECK_MARKER_LOOKBACK: u32 = 12;
/// The marker only counts with a reason of at least this many letters and digits after it.
pub const CHECK_MARKER_MIN_REASON: usize = 8;
/// Largest file that is read for the marker, in bytes.
pub const CHECK_MARKER_MAX_FILE_BYTES: u64 = 4_194_304;

/// At least one analysis worker, whatever the machine reports.
pub const MIN_WORKERS: usize = 1;
/// Index of the first file handed to the analysis workers.
pub const FIRST_INDEX: usize = 0;
/// Step between two file indexes handed to the analysis workers.
pub const INDEX_STEP: usize = 1;
/// Files analysed so far when the analysis starts.
pub const NONE_DONE: u32 = 0;
/// Files one worker adds to the progress after each file.
pub const DONE_STEP: u32 = 1;
/// Ids reserved for the file and symbols of one analysed file. A file that needs more fails the
/// scan instead of taking ids of its neighbour.
pub const FILE_ID_BLOCK: u64 = 1 << 24;

/// Bytes in one mebibyte, for the memory limit of an indexer.
pub const BYTES_PER_MIB: u64 = 1024 * 1024;
/// Separates the numbers of a tool version.
pub const VERSION_SEPARATOR: char = '.';
/// Characters that surround a version in the output of a tool, such as `v20.11.0` or `"17.0.9"`.
pub const VERSION_DECORATION: &[char] = &['"', '\'', 'v', 'V', '(', ')', ',', ';'];
/// Placeholder of an indexer argument for the project root.
pub const PLACEHOLDER_PROJECT: &str = "{project}";
/// Placeholder of an indexer argument for the index file the indexer writes.
pub const PLACEHOLDER_OUTPUT: &str = "{output}";
/// The word before the version in the output of `java -version`.
pub const VERSION_WORD: &str = "version";

/// In front of the name of a tool in the message keys of its purpose and install hint.
pub const TOOL_KEY_PREFIX: &str = "tool-";
/// After the name of a tool: the message key of what the tool is needed for.
pub const TOOL_PURPOSE_SUFFIX: &str = "-purpose";
/// After the name of a tool: the message key of how to install it.
pub const TOOL_INSTALL_SUFFIX: &str = "-install";
/// Separators of a date such as `2024-05-01` or `2024/05/01`.
pub const DATE_SEPARATORS: &[char] = &['-', '/'];
/// A number of this many digits or more is a date or an id (`20240501`), not a version.
pub const DATE_MIN_DIGITS: usize = 8;

/// The patch numbers of one feature band of the .NET SDK (8.0.100 to 8.0.199).
pub const FEATURE_BAND_SIZE: u32 = 100;

/// The event name of the log line for a warning of a scan.
pub const LOG_SCAN_WARNING: &str = "scan_warning";
/// Log event: a file was left out of the snapshot because it alone could not be analysed.
pub const LOG_FILE_SKIPPED: &str = "scan_file_skipped";
/// One more file left out under a code.
pub const LEFT_OUT_STEP: u64 = 1;

/// Log event: references that level 2 had verified were discarded because their target is gone.
pub const LOG_VERIFIED_DISCARDED: &str = "verified_references_discarded";

/// Log event: the critical symbols were compared with their baselines after a snapshot (ADR 0068).
pub const LOG_ALERTS_CHECKED: &str = "alerts_checked";

/// Log event: the alert step failed; the snapshot stays as it is.
pub const LOG_ALERT_STEP_FAILED: &str = "alert_step_failed";
/// Log event: the report of a scan could not be stored (`PRJ-502`).
pub const LOG_REPORT_NOT_STORED: &str = "scan_report_not_stored";
/// Log event: the stored report could not be read, so a run for one language starts from none.
pub const LOG_REPORT_NOT_READ: &str = "scan_report_not_read";
/// Log event: the id sequence could not be moved up to the store before level 2 ran.
pub const LOG_IDS_NOT_CAUGHT_UP: &str = "ids_not_caught_up";
/// Log event: a snapshot stores fewer ambiguous places than its references show (ADR 0086).
pub const LOG_AMBIGUOUS_COUNT_TOO_LOW: &str = "ambiguous_count_too_low";
/// Log event: the system gave no memory reading, so the memory limit of a scan does not protect (ADR 0087).
pub const LOG_MEMORY_READING_MISSING: &str = "memory_reading_missing";
/// Log event: the memory watch thread could not be started, the scan runs unwatched (ADR 0087).
pub const LOG_MEMORY_WATCH_NOT_STARTED: &str = "memory_watch_not_started";
/// Log event: a scan passed the memory limit and is stopped (`PRJ-503`).
pub const LOG_MEMORY_LIMIT_PASSED: &str = "memory_limit_passed";
/// Name of the thread that watches the memory of a scan.
pub const MEMORY_WATCH_THREAD: &str = "scan-memory-watch";

/// Wire word of `TokenClass::Comment`.
pub const TOKEN_COMMENT: &str = "comment";
/// Wire word of `TokenClass::String`.
pub const TOKEN_STRING: &str = "string";
/// Wire word of `TokenClass::Number`.
pub const TOKEN_NUMBER: &str = "number";
/// Wire word of `TokenClass::Keyword`.
pub const TOKEN_KEYWORD: &str = "keyword";
/// Wire word of `TokenClass::Type`.
pub const TOKEN_TYPE: &str = "type";
/// Wire word of `TokenClass::Function`.
pub const TOKEN_FUNCTION: &str = "function";
/// Wire word of `TokenClass::Attribute`.
pub const TOKEN_ATTRIBUTE: &str = "attribute";

/// Characters besides the control characters that can hide or reorder text on a terminal: the
/// direction marks and overrides, the zero-width characters, the line and paragraph separators
/// and the byte order mark. A project whose name, location or branch holds one is never shown in a
/// prompt, and no window is opened for it (ADR 0033, decision 10; ADR 0071, decision 11).
pub const INVISIBLE_RANGES: [(char, char); 7] = [
    ('\u{061C}', '\u{061C}'),
    ('\u{200B}', '\u{200F}'),
    ('\u{2028}', '\u{202E}'),
    ('\u{2060}', '\u{2064}'),
    ('\u{2066}', '\u{2069}'),
    ('\u{FEFF}', '\u{FEFF}'),
    ('\u{00AD}', '\u{00AD}'),
];

/// The program that the command to copy names. The copy is for a person at a terminal, so it
/// uses the name the person types, not the path of the running file.
pub const COPY_COMMAND_PROGRAM: &str = "codeskimr";
/// The words after the program in the command to copy: the command `level2 confirm`.
pub const COPY_COMMAND_WORDS: [&str; 2] = ["level2", "confirm"];
/// Separates the words of the command to copy.
pub const COPY_COMMAND_SEPARATOR: char = ' ';

/// The byte between the user name and the secret in the stored form of a credential (ADR 0077).
pub const FIELD_SEPARATOR: u8 = 0;
/// Length of [`FIELD_SEPARATOR`] in bytes.
pub const SEPARATOR_LEN: usize = 1;

/// Bits of a snapshot id that count the snapshots of one project; the project number sits above
/// them (ADR 0095). 20 + 33 = 53 bits, the largest integer range a browser reads exactly.
pub const COUNTER_BITS: u32 = 33;
/// The highest project number a snapshot id can carry.
pub const MAX_PROJECT_NUMBER: u64 = (1 << 20) - 1;
/// Start of a project file name that matches any prefix (`*.sln`).
pub const PROJECT_FILE_WILDCARD: char = '*';
/// Longest name of a language in a scope, in characters; the ids of the packs are far shorter.
pub const SCOPE_LANGUAGE_MAX_CHARS: usize = 64;
/// Characters besides letters and digits that the name of a language may hold in a scope.
pub const SCOPE_LANGUAGE_EXTRA: [char; 3] = ['-', '_', '+'];
/// The part of a path that goes up one folder; it is never part of a folder of a scope.
pub const PARENT_DIR: &str = "..";
/// The mark of a drive in a Windows path (`C:`); a scope folder never names one.
pub const DRIVE_MARK: char = ':';

/// Most usages of one symbol that the rename preview reads; a symbol with more places than this
/// is renamed by a tool, not by a list.
pub const RENAME_MAX_SITES: u32 = 100_000;
/// Largest file the text search of the rename preview reads, in bytes.
pub const RENAME_MAX_FILE_BYTES: u64 = 2_097_152;
/// How many symbols of the name search are looked at for a name conflict.
pub const RENAME_CONFLICT_SEARCH_LIMIT: u32 = 100;
/// Extensions of documentation files.
pub const RENAME_DOC_EXTENSIONS: [&str; 6] = ["md", "mdx", "markdown", "txt", "rst", "adoc"];
/// Extensions of configuration files.
pub const RENAME_CONFIG_EXTENSIONS: [&str; 11] = [
    "json",
    "toml",
    "yaml",
    "yml",
    "xml",
    "ini",
    "properties",
    "config",
    "env",
    "cfg",
    "conf",
];
/// What the compiler follows when a name changes: the argument of this call is a use of the name.
pub const RENAME_NAMEOF: &str = "nameof(";
/// Where a comment of a code file starts.
pub const RENAME_COMMENT_PREFIXES: [&str; 6] = ["//", "#", "/*", "*", "--", ";"];
/// Characters that open and close a string.
pub const RENAME_QUOTES: [char; 3] = ['"', '\'', '`'];
/// Quotes come in pairs: an odd number before a place means the place is inside a string.
pub const RENAME_QUOTE_PAIR: usize = 2;
/// Separates a file name from its extension.
pub const RENAME_EXTENSION_SEPARATOR: char = '.';
/// The extension of a file that has none.
pub const RENAME_NO_EXTENSION: &str = "";
/// The one character besides letters and digits that a name may hold.
pub const RENAME_UNDERSCORE: char = '_';
/// Label of a place that is a declaration.
pub const RENAME_SITE_DECLARATION: &str = "declaration";
/// Label of a place that uses the symbol.
pub const RENAME_SITE_USAGE: &str = "usage";
/// Label of a text line in a documentation file.
pub const RENAME_TEXT_DOCUMENTATION: &str = "documentation";
/// Label of a text line in a configuration file.
pub const RENAME_TEXT_CONFIGURATION: &str = "configuration";
/// Label of a text line inside quotes.
pub const RENAME_TEXT_STRING: &str = "string";
/// Label of a text line with `nameof`.
pub const RENAME_TEXT_NAMEOF: &str = "nameof";
/// Label of a text line that is a comment.
pub const RENAME_TEXT_COMMENT: &str = "comment";
/// Label of a text line that no usage explains.
pub const RENAME_TEXT_CODE: &str = "code";
/// Most characters of the code line of a place that a rename preview keeps.
pub const RENAME_CODE_CHARS: usize = 240;
/// Most lines one read of a file may span when the code lines of the places in it are read.
pub const RENAME_CODE_MAX_SPAN: u32 = 5_000;
/// Offset between a zero-based line of the index and the one-based line people read.
pub const RENAME_LINE_OFFSET: u32 = 1;
/// Offset between a zero-based column of the index and the one-based column people read.
pub const RENAME_COLUMN_OFFSET: u32 = 1;
