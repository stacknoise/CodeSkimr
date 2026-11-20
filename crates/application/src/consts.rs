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

/// Log event: references that level 2 had verified were discarded because their target is gone.
pub const LOG_VERIFIED_DISCARDED: &str = "verified_references_discarded";

/// Log event: the critical symbols were compared with their baselines after a snapshot (ADR 0068).
pub const LOG_ALERTS_CHECKED: &str = "alerts_checked";

/// Log event: the alert step failed; the snapshot stays as it is.
pub const LOG_ALERT_STEP_FAILED: &str = "alert_step_failed";
/// Log event: the report of a scan could not be stored (`PRJ-502`).
pub const LOG_REPORT_NOT_STORED: &str = "scan_report_not_stored";

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
