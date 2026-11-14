//! Constants of the source adapter. Tunable values live in `config/default.toml`; what is here is
//! fixed by the formats the adapter speaks (paths, Git, hashing).

/// Separator of a `RelativePath`, independent of the operating system.
pub const RELATIVE_PATH_SEPARATOR: char = '/';

/// Name of the Git metadata folder; it is never scanned.
pub const GIT_DIR_NAME: &str = ".git";

/// Read buffer size when hashing a file, in bytes.
pub const HASH_BUFFER_BYTES: usize = 64 * 1024;

/// Depth of a shallow clone: only the newest commit is needed.
pub const SHALLOW_CLONE_DEPTH: u32 = 1;

/// How often a running clone checks whether the scan was cancelled, in milliseconds.
pub const CANCEL_POLL_MS: u64 = 50;

/// Prefix of a branch name in a full reference name.
pub const BRANCH_REF_PREFIX: &str = "refs/heads/";

/// Phrases in a transport error that show the remote asked for credentials (lower case).
pub const AUTH_FAILURE_HINTS: [&str; 4] = [
    "failed to obtain credentials",
    "could not obtain identity",
    "authentication failed",
    "permission denied (publickey",
];

/// Settings that every clone starts with: no prompt on the terminal. A server, an MCP session or a
/// scan in the background cannot answer a prompt and would hang (issue #58).
pub const CLONE_CONFIG_OVERRIDES: [&str; 1] = ["gitoxide.credentials.terminalPrompt=false"];

/// Phrase in a transport error for HTTP status 404 (lower case). `gix` has no variant for it, so
/// the text is the only sign; `tests/login.rs` fails if a `gix` update changes it.
pub const HTTP_NOT_FOUND_HINT: &str = "received http status 404";

/// Phrase in a fetch error that shows the requested branch does not exist (lower case).
pub const MISSING_BRANCH_HINT: &str = "didn't have any ref that matched";

/// Detail of the failure that is reported when a fetch was cancelled.
pub const CANCELLED_DETAIL: &str = "cancelled";

/// Prefix of the folders a clone is prepared in and the old copy is parked in.
pub const WORK_PREFIX: &str = ".work-";

/// Separator between the parts of a work folder name.
pub const WORK_SEPARATOR: char = '-';

/// Separator between the messages of an error chain.
pub const CHAIN_SEPARATOR: &str = ": ";

/// Text of the failure that stops a Git login when the secret store has no usable credential. The
/// reason itself travels as a coded error.
pub const NO_LOGIN_MESSAGE: &str = "no credential available";

/// Marker that separates the scheme of a URL from the host.
pub const URL_SCHEME_MARKER: &str = "://";

/// Ends a line of a source file.
pub const LINE_END: u8 = b'\n';

/// Precedes `LINE_END` in files with Windows line ends; it is not part of the line.
pub const CARRIAGE_RETURN: char = '\r';

/// Number of the first line of a file.
pub const FIRST_LINE: u32 = 1;

/// Marks a line of a plain-text requirement file as a comment.
pub const REQUIREMENT_COMMENT: char = '#';
