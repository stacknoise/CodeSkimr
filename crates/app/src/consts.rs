//! Fixed values of the composition root: environment variables, folder and file names.

use std::time::Duration;

/// Environment variable that sets the data folder.
pub const ENV_DATA_DIR: &str = "CODESKIMR_DATA_DIR";
/// Environment variable that sets the language pack folder.
pub const ENV_LANGUAGES_DIR: &str = "CODESKIMR_LANGUAGES_DIR";
/// Environment variable that sets the user configuration file.
pub const ENV_CONFIG: &str = "CODESKIMR_CONFIG";
/// Environment variables that name the language of the texts, strongest first.
pub const ENV_LANGUAGE_TAGS: [&str; 4] = ["CODESKIMR_LANG", "LC_ALL", "LC_MESSAGES", "LANG"];

/// Home folder of the user on Unix.
pub const ENV_HOME: &str = "HOME";
/// Roaming application data on Windows.
pub const ENV_APPDATA: &str = "APPDATA";
/// Local application data on Windows.
pub const ENV_LOCAL_APPDATA: &str = "LOCALAPPDATA";
/// Base folder of user configuration on Linux.
pub const ENV_XDG_CONFIG_HOME: &str = "XDG_CONFIG_HOME";
/// Base folder of user data on Linux.
pub const ENV_XDG_DATA_HOME: &str = "XDG_DATA_HOME";

/// Folder of `CodeSkimr` inside the base folders above.
pub const APP_DIR: &str = "codeskimr";
/// Path of the configuration base below the home folder on Linux.
pub const HOME_CONFIG_PARTS: [&str; 1] = [".config"];
/// Path of the data base below the home folder on Linux.
pub const HOME_DATA_PARTS: [&str; 2] = [".local", "share"];
/// Path of the application support base below the home folder on macOS.
pub const HOME_MACOS_PARTS: [&str; 2] = ["Library", "Application Support"];

/// The catalog database inside the data folder; the project files lie in the folder `projects` beside it (ADR 0095).
pub const DATABASE_FILE: &str = "catalog.db";
/// Empty file whose lock serialises the creation of the installation token.
/// How long the first start waits for the token lock of another program.
pub const TOKEN_LOCK_WAIT: Duration = Duration::from_secs(10);
/// How often it looks at the lock while it waits.
pub const TOKEN_LOCK_POLL: Duration = Duration::from_millis(50);
pub const TOKEN_LOCK_FILE: &str = "token.lock";
/// The folder of the Git clones inside the data folder.
pub const GIT_CACHE_DIR: &str = "git";
/// The user configuration file inside the configuration folder.
pub const USER_CONFIG_FILE: &str = "config.toml";
/// The folder of the language packs next to the executable or the current folder.
pub const LANGUAGES_DIR: &str = "languages";
/// The file that makes a folder a language pack.
pub const PACK_FILE: &str = "language.toml";
/// The folder inside the data folder that holds the language packs written from the program.
pub const PACKS_CACHE_DIR: &str = "languages";
/// The file that says a folder of embedded packs was written completely.
pub const PACKS_COMPLETE_MARKER: &str = ".complete";
/// Age from which an unfinished pack folder counts as left behind by a crashed start.
pub const PACKS_STAGING_MAX_AGE: std::time::Duration = std::time::Duration::from_secs(24 * 60 * 60);
/// Age from which the pack folder of a program that was replaced is removed.
pub const PACKS_REPLACED_MAX_AGE: std::time::Duration =
    std::time::Duration::from_secs(30 * 24 * 60 * 60);
/// Between the fingerprint and the process number in the name of a folder that is being written.
pub const PACKS_STAGING_SEPARATOR: &str = ".writing-";

/// Branch of a Git source when none is given.
pub const DEFAULT_BRANCH: &str = "main";
/// Beginnings that mark a source as a Git URL.
pub const GIT_URL_PREFIXES: [&str; 5] = ["https://", "http://", "ssh://", "git://", "git@"];
/// End of a Git URL that is not recognised by its beginning.
pub const GIT_URL_SUFFIX: &str = ".git";
/// Separates the scheme from the rest of a URL.
pub const SCHEME_SEPARATOR: &str = "://";
/// Marks credentials in the authority part of a URL.
pub const USERINFO_MARKER: char = '@';
/// Scheme prefix of `http` and `https` URLs.
pub const HTTP_SCHEME: &str = "http";
/// Separates user name and password in the authority part of a URL.
pub const PASSWORD_SEPARATOR: char = ':';
/// Characters that end the authority part of a URL.
pub const AUTHORITY_END: [char; 3] = ['/', '?', '#'];
/// Stands for the scheme of an address that has none.
pub const NO_SCHEME: &str = "";
/// Where tokens hide in a URL: query and fragment.
pub const QUERY_MARKERS: [char; 2] = ['?', '#'];
/// Prefix of a Windows verbatim path.
pub const VERBATIM_PREFIX: &str = "\\\\?\\";
/// What follows the verbatim prefix of a UNC path.
pub const VERBATIM_UNC: &str = "UNC\\";
/// The start of a UNC path as users and the system opener know it (`\\server\share`).
#[cfg(any(windows, test))]
pub const UNC_PREFIX: &str = "\\\\";
/// Separates the parts of a URL path.
pub const URL_PATH_SEPARATOR: char = '/';
/// Separates host and path in an scp-style Git address.
pub const SCP_PATH_SEPARATOR: char = ':';

/// Most scans that run at the same time, over all projects. Each scan fills the cores and holds
/// the parsed files in memory, so more only slow down all of them (and let a caller exhaust the
/// machine); one more is refused with `PRJ-501`.
pub const MAX_PARALLEL_SCANS: usize = 4;
/// Longest Git address that is accepted, in characters.
pub const MAX_GIT_URL_CHARS: usize = 2048;
/// Longest branch name that is accepted, in characters.
pub const MAX_BRANCH_CHARS: usize = 255;
/// The longest display name of a project, in characters.
pub const MAX_PROJECT_NAME_CHARS: usize = 80;
/// Folder names that tell little on their own; a project in such a folder is named after the
/// folder above it as well (`webshop/src`).
pub const GENERIC_FOLDER_NAMES: &[&str] = &["src", "source", "sources", "lib", "www"];
/// Joins the parent folder name and a generic folder name in the default project name.
pub const NAME_PART_SEPARATOR: char = '/';
/// A Git address or branch name that starts with this could be read as an option by `git` or `ssh`.
pub const OPTION_MARKER: char = '-';
/// Characters that no branch name may hold (`git check-ref-format`), besides whitespace and
/// control characters.
pub const BRANCH_FORBIDDEN: [char; 6] = ['~', '^', ':', '?', '*', '['];
/// Text that no branch name holds: two dots, a backslash, `@{` and two slashes in a row.
pub const BRANCH_FORBIDDEN_PARTS: [&str; 4] = ["..", "\\", "@{", "//"];
/// Endings that no branch name has: `.lock`, a dot and a slash.
pub const BRANCH_FORBIDDEN_ENDS: [&str; 3] = [".lock", ".", "/"];
/// Separates the parts of a branch name; a name does not start with it.
pub const BRANCH_SEPARATOR: char = '/';
/// Marks a transport helper in a Git address (`ext::command`), which would run a program.
pub const TRANSPORT_HELPER_MARKER: &str = "::";

/// Bytes in a megabyte, for the size limit of a source.
pub const BYTES_PER_MB: u64 = 1_048_576;
/// How often a running scan checks its memory, in milliseconds (ADR 0087).
pub const MEMORY_CHECK_INTERVAL_MS: u64 = 500;
/// The share of the installed memory that a scan may hold at most, in percent (ADR 0087).
pub const MEMORY_RAM_SHARE_PERCENT: u64 = 70;
/// The whole in percent: the divisor of a percentage.
pub const PERCENT_WHOLE: u64 = 100;
/// Bytes in a kilobyte, for the size of an export file.
pub const BYTES_PER_KILOBYTE: usize = 1_024;

/// Separates code and description when no message texts could be loaded.
pub const PLAIN_ERROR_SEPARATOR: &str = ": ";

/// Operating system name of macOS in `std::env::consts::OS`.
pub const OS_MACOS: &str = "macos";

/// Exit code: a command failed.
pub const EXIT_FAILURE: u8 = 1;
/// Exit code: the command line is invalid.
pub const EXIT_USAGE: u8 = 2;

/// Ends a printed line.
pub const LINE_BREAK: &str = "\n";

/// Environment variable that sets the log level (`error`, `warn`, `info`, `debug`, `trace`, `off`).
pub const ENV_LOG: &str = "CODESKIMR_LOG";
/// Log level when `CODESKIMR_LOG` is not set or not understood.
pub const DEFAULT_LOG_LEVEL: tracing_subscriber::filter::LevelFilter =
    tracing_subscriber::filter::LevelFilter::WARN;

/// Replaces every secret that is removed from a log line.
pub const REDACTED: &str = "[redacted]";
/// Header and field names whose value runs to the end of the line, case-insensitive.
pub const LINE_KEYS: [&str; 2] = ["authorization", "cookie"];
/// Field and parameter names whose value is one word, case-insensitive.
pub const VALUE_KEYS: [&str; 15] = [
    "passphrase",
    "password",
    "passwd",
    "secret",
    "token",
    "api_key",
    "api-key",
    "apikey",
    "private_key",
    "credential",
    "session_id",
    // Pre-signed URLs: `X-Amz-Signature`, `X-Amz-Credential`, Azure `?sig=`.
    "signature",
    "x-amz-credential",
    "?sig",
    "&sig",
];
/// Scheme word of a bearer token; its value follows after a blank, case-insensitive.
pub const BEARER_KEY: &str = "bearer";
/// Separates a key from its value.
pub const KEY_SEPARATORS: [char; 3] = [':', '=', '('];
/// Quote characters around a key or a value.
pub const QUOTES: [char; 2] = ['"', '\''];
/// Blanks between a key, its separator and its value.
pub const BLANKS: [char; 2] = [' ', '\t'];
/// Characters that end an unquoted one-word value. A secret may contain commas and brackets, so
/// only the ones that separate parameters or open a quote end it.
pub const VALUE_END: [char; 3] = ['&', '"', '\''];
/// Follows the separator in `"key" => "value"`.
pub const ARROW: char = '>';
/// Opens the content of a `Debug`-formatted wrapper such as `Some("x")` or `Token("x")`.
pub const CALL_OPEN: char = '(';
/// Beginning of a PEM block.
pub const PEM_BEGIN: &str = "-----BEGIN ";
/// What the header of a PEM block says if the block holds a private key.
pub const PEM_PRIVATE: &str = "PRIVATE KEY";
/// Beginning of the last line of a PEM block.
pub const PEM_END: &str = "-----END ";
/// Beginnings of well-known tokens: GitHub, GitLab, Slack, AWS, JWT, API keys, and the `csk_`
/// tokens of `CodeSkimr` itself (ADR 0019, S-01).
pub const TOKEN_PREFIXES: [&str; 23] = [
    "csk_",
    "ASIA",
    "sk_live_",
    "sk_test_",
    "npm_",
    "xapp-",
    "xoxe-",
    "xoxs-",
    "xoxr-",
    "github_pat_",
    "ghp_",
    "gho_",
    "ghu_",
    "ghs_",
    "ghr_",
    "glpat-",
    "xoxa-",
    "xoxb-",
    "xoxp-",
    "AKIA",
    "eyJ",
    "sk-",
    "AIza",
];
/// A token has at least this many characters after its beginning; shorter runs are words.
pub const TOKEN_MIN_TAIL: usize = 16;
/// Characters besides letters and digits that occur inside tokens.
pub const TOKEN_EXTRA_CHARS: [char; 3] = ['_', '-', '.'];
/// Ends a line of log output.
pub const LINE_END: char = '\n';
/// Escapes a quote inside a `Debug`-formatted text.
pub const ESCAPE: char = '\\';
/// Backslashes that escape each other: an even run does not escape the character behind it.
pub const ESCAPE_PAIR: usize = 2;

/// Name under which the installation token lives in the credential store (blueprint 12.2).
pub const TOKEN_CREDENTIAL_ID: &str = "installation-token";
/// Whether `name` is the installation token under any spelling a keystore could fold together
/// (case, surrounding blanks): such a name must never be managed or lent to a remote.
#[must_use]
pub fn is_token_credential(name: &str) -> bool {
    name.trim().eq_ignore_ascii_case(TOKEN_CREDENTIAL_ID)
}
/// Ending of the name under which the refresh token of a credential is kept.
pub const REFRESH_SUFFIX: &str = ".refresh";
/// Number of fields in the user field of a refresh entry: provider, application id and host.
pub const REFRESH_LINK_FIELDS: usize = 3;
/// Separates provider, application id and host in the user field of a refresh entry.
pub const REFRESH_LINK_SEPARATOR: char = '\t';
/// Whether `name` is the refresh entry of another credential: it belongs to the program, a user
/// manages the credential and never the entry next to it.
#[must_use]
pub fn is_refresh_credential(name: &str) -> bool {
    name.trim().to_ascii_lowercase().ends_with(REFRESH_SUFFIX)
}
/// Whether `name` belongs to the program and is neither managed by a user nor lent to a remote:
/// the installation token and the refresh entry of a credential.
#[must_use]
pub fn is_reserved_credential(name: &str) -> bool {
    is_token_credential(name) || is_refresh_credential(name)
}
/// User name stored next to the installation token; the credential format needs one.
pub const TOKEN_CREDENTIAL_USER: &str = "local-owner";
/// Number of random bytes of an installation token: 256 bits (ADR 0019).
pub const TOKEN_RANDOM_BYTES: usize = 32;
/// Length of the hexadecimal text of the random bytes: two digits per byte.
pub const TOKEN_HEX_LEN: usize = 2 * TOKEN_RANDOM_BYTES;
/// Radix of the text form of the random bytes.
pub const HEX_RADIX: u32 = 16;
/// Bits in one hexadecimal digit.
pub const HEX_DIGIT_BITS: u32 = 4;
/// Mask of the low hexadecimal digit of a byte.
pub const HEX_LOW_MASK: u8 = 0x0f;
/// Scheme of the address the local server is reached at.
pub const URL_SCHEME: &str = "http://";

/// Folder of the launch files below the data folder (ADR 0028).
pub const LAUNCH_DIR: &str = "launch";
/// Environment variable of the private runtime folder of the user on Linux.
pub const ENV_XDG_RUNTIME_DIR: &str = "XDG_RUNTIME_DIR";
/// Beginning of the name of a launch file.
pub const LAUNCH_FILE_PREFIX: &str = "launch-";
/// End of the name of a launch file.
pub const LAUNCH_FILE_SUFFIX: &str = ".html";
/// Random bytes in the name of a launch file: 128 bits, so that the name cannot be guessed.
pub const LAUNCH_FILE_RANDOM_BYTES: usize = 16;
/// Mode of a launch file: the owner may read and write, nobody else.
#[cfg(unix)]
pub const LAUNCH_FILE_MODE: u32 = 0o600;
/// Mode of the folder of the launch files: the owner only.
#[cfg(unix)]
pub const LAUNCH_DIR_MODE: u32 = 0o700;
/// Mode bits that give a group or others any access.
#[cfg(unix)]
pub const OTHERS_MODE_MASK: u32 = 0o077;
/// The Windows tool that sets access rules.
#[cfg(windows)]
pub const ACL_TOOL: &str = "icacls.exe";
/// Windows file attribute of a reparse point: a junction or a link (`FILE_ATTRIBUTE_REPARSE_POINT`).
#[cfg(windows)]
pub const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
/// Opening flag that lets Windows open a folder as a handle (`FILE_FLAG_BACKUP_SEMANTICS`).
#[cfg(windows)]
pub const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
/// Opening flag that opens a link itself and does not follow it (`FILE_FLAG_OPEN_REPARSE_POINT`).
#[cfg(windows)]
pub const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
/// Owners besides the user that are fine for the launch folder and the folders above it, as SID
/// strings: the group of the administrators, the system account and the service that installs
/// Windows itself, which owns a drive root (the Windows counterparts of root on Unix).
#[cfg(windows)]
pub const TRUSTED_OWNER_SIDS: &[&str] = &[
    "S-1-5-32-544",
    "S-1-5-18",
    "S-1-5-80-956008885-3418522649-1831038044-1853292631-2271478464",
];
/// Access right that is enough to read who owns a file or folder (`READ_CONTROL`).
#[cfg(windows)]
pub const READ_CONTROL: u32 = 0x0002_0000;
/// Log event: a folder above the launch folder belongs to an account that is not trusted.
#[cfg(windows)]
pub const LOG_LAUNCH_PARENT_REFUSED: &str = "launch_parent_refused";
/// Environment variable with the Windows folder, below which the tool is looked for.
#[cfg(windows)]
pub const ENV_SYSTEM_ROOT: &str = "SYSTEMROOT";
/// Folder of the system programs below the Windows folder.
#[cfg(windows)]
pub const SYSTEM_TOOLS_DIR: &str = "System32";
/// `icacls` option: drop the rules the folder inherits (from a drive root they may give others access).
#[cfg(any(windows, test))]
pub const ACL_NO_INHERIT: &str = "/inheritance:r";
/// `icacls` option: replace the rules for the named account.
#[cfg(any(windows, test))]
pub const ACL_GRANT: &str = "/grant:r";
/// `icacls` option: no message about every file that was changed.
#[cfg(any(windows, test))]
pub const ACL_QUIET: &str = "/q";
/// Rule for the account: full access to the folder and, by inheritance, to the files created in it.
#[cfg(any(windows, test))]
pub const ACL_FULL_RULE: &str = ":(OI)(CI)F";
/// Environment variable with the name of the user on Windows.
#[cfg(any(windows, test))]
pub const ENV_USERNAME: &str = "USERNAME";
/// Environment variable with the domain (or computer name) of the user on Windows.
#[cfg(any(windows, test))]
pub const ENV_USERDOMAIN: &str = "USERDOMAIN";
/// Separates domain and user in a Windows account name.
#[cfg(any(windows, test))]
pub const ACCOUNT_SEPARATOR: &str = "\\";
/// Mode bit that lets everybody write. The group bit is left out on purpose: on many systems every
/// user has a private group and creates folders with group write.
#[cfg(unix)]
pub const OTHERS_WRITE_MASK: u32 = 0o002;
/// Mode bit of a folder in which only the owner of an entry may remove or rename it (`/tmp`).
#[cfg(unix)]
pub const STICKY_MASK: u32 = 0o1000;
/// The user id of the system administrator, who may own the folders above the launch folder.
#[cfg(unix)]
pub const ROOT_UID: u32 = 0;
/// Launch files older than this many seconds beyond the lifetime of a code are removed.
pub const LAUNCH_FILE_GRACE_SECONDS: u64 = 60;

/// Start of the launch page; the address follows.
pub const LAUNCH_PAGE_HEAD: &str = "<!doctype html>\n<meta charset=\"utf-8\">\n<meta name=\"referrer\" content=\"no-referrer\">\n<meta http-equiv=\"refresh\" content=\"0;url=";
/// End of the launch page.
pub const LAUNCH_PAGE_TAIL: &str = "\">\n";
/// Fragment parameter that carries the launch code (ADR 0028).
pub const LAUNCH_FRAGMENT: &str = "/#code=";
/// Characters of a launch code: 128 bits as hexadecimal digits.
pub const LAUNCH_CODE_CHARS: usize = 32;
/// Path of the launch code route on the local server.
pub const LAUNCH_CODE_PATH: &str = "/api/v1/session/launch-code";
/// Name of the field with the code in the answer.
pub const FIELD_CODE: &str = "code";
/// Name of the field that says whether a code is still waiting.
pub const FIELD_PENDING: &str = "pending";
/// Status of a successful answer.
pub const HTTP_OK: u16 = 200;
/// Status of an answer that refuses the token.
pub const HTTP_UNAUTHORIZED: u16 = 401;
/// Loopback address the local server listens on.
pub const LOOPBACK_HOST: &str = "127.0.0.1";
/// Longest wait for a connection to the local server, in milliseconds.
pub const CONNECT_TIMEOUT_MS: u64 = 2_000;
/// Longest wait for an answer of the local server, in milliseconds.
pub const REPLY_TIMEOUT_MS: u64 = 5_000;
/// Pause between two looks at whether the launch code has been taken, in milliseconds.
pub const CODE_POLL_MS: u64 = 500;
/// Largest answer of the local server that is read, in bytes, as a buffer size.
pub const MAX_REPLY_LEN: usize = 65_536;
/// Largest answer of the local server that is read, in bytes, as the reader counts them.
pub const MAX_REPLY_BYTES: u64 = MAX_REPLY_LEN as u64;
/// Separates the head of an HTTP message from its body.
pub const HTTP_HEAD_END: &str = "\r\n\r\n";
/// Line break of an HTTP message.
pub const HTTP_LINE_BREAK: &str = "\r\n";
/// Position of the status code in the status line.
pub const HTTP_STATUS_FIELD: usize = 1;

/// Program that opens a file with its default application on Linux and other Unix systems.
pub const OPENER_UNIX: &str = "xdg-open";
/// Program that opens a file with its default application on macOS.
pub const OPENER_MACOS: &str = "open";
/// Program that opens a file with its default application on Windows.
pub const OPENER_WINDOWS: &str = "explorer";
/// Protocol version in the request line.
pub const HTTP_VERSION: &str = "HTTP/1.1";
/// Headers of every request to the local server: no body, and the connection ends after the answer.
pub const HTTP_FIXED_HEADERS: [&str; 2] = ["Content-Length: 0", "Connection: close"];
/// Method of a call that reads.
pub const METHOD_GET: &str = "GET";
/// Method of a call that changes something.
pub const METHOD_POST: &str = "POST";

/// Separates the parts of an HTTP request line.
pub const HTTP_REQUEST_SEPARATOR: &str = " ";
/// Start of the `Host` header line.
pub const HTTP_HOST_HEADER: &str = "Host: ";
/// Start of the `Authorization` header line.
pub const HTTP_BEARER_HEADER: &str = "Authorization: Bearer ";
/// Separates the host and the port of an address.
pub const PORT_SEPARATOR: &str = ":";

/// Ends a line of standard input.
pub const LINE_FEED: char = '\n';
/// Precedes the line feed in input from Windows.
pub const CARRIAGE_RETURN: char = '\r';

/// Longest secret that is read from standard input, in bytes.
pub const MAX_SECRET_BYTES: u64 = 65_536;
/// Log event: the check of the tools of a project failed; the scan goes on.
pub const LOG_TOOL_CHECK_FAILED: &str = "tool_check_failed";
/// Log event: the settings saved in the app break a rule of the configuration and are left out.
pub const LOG_SAVED_SETTINGS_IGNORED: &str = "saved_settings_ignored";

/// Largest file that is read for the version a project requires (`global.json`, `.nvmrc` and so
/// on), in bytes.
pub const REQUIREMENT_MAX_BYTES: u64 = 65_536;

/// Windows process creation flag: the new process gets a console window of its own.
#[cfg(windows)]
pub const CREATE_NEW_CONSOLE: u32 = 0x0000_0010;
/// The program that macOS starts to run an `AppleScript`. An absolute path, never a lookup.
pub const OSASCRIPT_PROGRAM: &str = "/usr/bin/osascript";
/// The option of `osascript` that takes the text of the script.
pub const OSASCRIPT_SCRIPT_OPTION: &str = "-e";
/// Ends the options of `osascript`: what follows is handed to the script as its arguments, also
/// words that start with a dash.
pub const OSASCRIPT_END_OF_OPTIONS: &str = "--";
/// The constant script that opens Terminal with the command. It holds no value of its own: the
/// program, the words and the project id arrive as `argv` items, and each one is quoted for the
/// shell with `quoted form of` before the line is assembled (ADR 0071, decision 5).
pub const TERMINAL_SCRIPT: &str = "on run argv
set parts to {}
repeat with piece in argv
set end of parts to quoted form of (piece as text)
end repeat
set AppleScript's text item delimiters to \" \"
tell application \"Terminal\"
activate
do script (parts as text)
end tell
end run";
/// The words after the program in the command that a confirmation window runs; the flag marks the
/// window as one that the server opened, and the project id follows.
pub const CONFIRM_WINDOW_WORDS: [&str; 3] = ["level2", "confirm", "--from-ui"];
/// Variables that name a remote session: a window opened there would appear on a screen that
/// nobody sits in front of (ADR 0071, decision 6).
pub const REMOTE_SESSION_VARIABLES: [&str; 3] = ["SSH_CONNECTION", "SSH_CLIENT", "SSH_TTY"];
/// Variables of which at least one is set in a session with a screen, per system (ADR 0071,
/// decision 6): Linux has a display (X11 or Wayland), Windows names its session of the window
/// station (`Console` or `RDP-Tcp#n`; a service has none), macOS has the id of the login session
/// of the user. Without one the check cannot decide for "interactive", so the answer is `copy`.
#[cfg(target_os = "linux")]
pub const DESKTOP_SESSION_VARIABLES: [&str; 2] = ["DISPLAY", "WAYLAND_DISPLAY"];
/// See the Linux definition.
#[cfg(windows)]
pub const DESKTOP_SESSION_VARIABLES: [&str; 1] = ["SESSIONNAME"];
/// See the Linux definition.
#[cfg(not(any(target_os = "linux", windows)))]
pub const DESKTOP_SESSION_VARIABLES: [&str; 1] = ["SECURITYSESSIONID"];
/// The console of the process on Windows, for input. A window the server opened reads its
/// answer from here, because its standard streams may be the ones of a program without a console
/// (ADR 0071).
#[cfg(windows)]
pub const CONSOLE_INPUT: &str = "CONIN$";
/// The console of the process on Windows, for output.
#[cfg(windows)]
pub const CONSOLE_OUTPUT: &str = "CONOUT$";

/// Pause between two looks at whether the runs have ended.
pub const IDLE_POLL: Duration = Duration::from_millis(50);
/// How long the end of the server waits for the runs it cancelled (ADR 0072).
pub const SHUTDOWN_GRACE: Duration = Duration::from_secs(5);
/// Name of the thread that deletes the dismissed alerts that are past their retention (ADR 0068).
pub const PURGE_THREAD_NAME: &str = "alert-purge";
/// How long the server waits between two purges of dismissed alerts: one day (ADR 0068).
pub const PURGE_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);
/// Name of the thread that runs level 2 after a scan from the browser (ADR 0072).
pub const LEVEL2_THREAD_NAME: &str = "level2-run";

/// Environment variable that marks the program started by the bare `codeskimr` for the tray: it
/// runs the server and the tray icon, the starting program only starts it and ends (ADR 0067).
pub const ENV_TRAY_CHILD: &str = "CODESKIMR_TRAY_CHILD";
/// Value of [`ENV_TRAY_CHILD`].
pub const TRAY_CHILD_ON: &str = "1";
/// The log of the tray program, inside the data folder. It starts empty with every start.
pub const TRAY_LOG_FILE: &str = "codeskimr.log";
/// Windows process creation flag: the new process gets no console window.
#[cfg(windows)]
pub const CREATE_NO_WINDOW: u32 = 0x0800_0000;
/// How often the tray looks whether the server has ended on its own.
#[cfg(any(windows, target_os = "macos"))]
pub const TRAY_SERVER_POLL: std::time::Duration = std::time::Duration::from_secs(1);
/// Width and height of the tray icon in pixels.
#[cfg(any(windows, target_os = "macos"))]
pub const TRAY_ICON_SIZE: u32 = 64;
/// The tray icon: the app logo as raw RGBA pixels, `TRAY_ICON_SIZE` square (made by `tools/logo/make_icons.py`).
#[cfg(any(windows, target_os = "macos"))]
pub const TRAY_ICON_RGBA: &[u8] = include_bytes!("../assets/tray-icon.rgba");
// A wrongly made icon file fails the build, not the start of the tray.
#[cfg(any(windows, target_os = "macos"))]
const _: () = assert!(TRAY_ICON_RGBA.len() == (TRAY_ICON_SIZE * TRAY_ICON_SIZE * 4) as usize);

/// Separates the lines of the output of `annotations` and `alerts`.
pub const CLI_LINE_BREAK: &str = "\n";
/// The control characters that stay in the text output of `annotations` and `alerts`.
pub const CLI_KEPT_CONTROLS: [char; 2] = ['\t', '\n'];
/// Separates the parts of the summary of an annotation or of the detail of an alert.
pub const CLI_LIST_SEPARATOR: &str = ", ";

/// Names of the Fluent arguments of the messages of `annotations` and `alerts`.
pub mod note_arg {
    /// `{ $id }`: id of an annotation or an alert.
    pub const ID: &str = "id";
    /// `{ $kind }`: kind of an annotation or an alert.
    pub const KIND: &str = "kind";
    /// `{ $name }`: qualified name of the symbol.
    pub const NAME: &str = "name";
    /// `{ $state }`: state of an alert.
    pub const STATE: &str = "state";
    /// `{ $parts }`: 0 if the line has no summary or detail, 1 if it has.
    pub const PARTS: &str = "parts";
    /// `{ $summary }`: what an annotation says.
    pub const SUMMARY: &str = "summary";
    /// `{ $detail }`: what an alert found.
    pub const DETAIL: &str = "detail";
    /// `{ $shown }`: rows on this page.
    pub const SHOWN: &str = "shown";
    /// `{ $total }`: rows of the whole list.
    pub const TOTAL: &str = "total";
    /// `{ $next }`: the offset of the next page.
    pub const NEXT: &str = "next";
    /// `{ $open }`: open alerts.
    pub const OPEN: &str = "open";
    /// `{ $dismissed }`: alerts that were dismissed.
    pub const DISMISSED: &str = "dismissed";
}

/// Name of the thread that counts the numbers of the statistics after a scan (ADR 0091).
pub const STATISTICS_THREAD_NAME: &str = "statistics-count";
/// Log event: the numbers of the statistics could not be counted after a scan.
pub const LOG_STATISTICS_FAILED: &str = "statistics_count_failed";

/// What `codeskimr scope` prints for a number of files that was not counted.
pub const CLI_SCOPE_UNCOUNTED: &str = "?";
/// Value of the select key that marks a language or folder that scans cover.
pub const CLI_SCOPE_COVERED: &str = "covered";
/// Value of the select key that marks a language or folder that scans leave out.
pub const CLI_SCOPE_LEFT_OUT: &str = "left_out";

/// Names of the arguments of the messages of `codeskimr scope`.
pub mod scope_arg {
    /// `{ $project }`: id of the project.
    pub const PROJECT: &str = "project";
    /// `{ $state }`: `covered` or `left_out`.
    pub const STATE: &str = "state";
    /// `{ $language }`: id of a language pack.
    pub const LANGUAGE: &str = "language";
    /// `{ $path }`: a folder of the project.
    pub const PATH: &str = "path";
    /// `{ $files }`: number of files, or `?`.
    pub const FILES: &str = "files";
    /// `{ $count }`: number of folders that are not listed.
    pub const COUNT: &str = "count";
    /// `{ $folder }`: the starting folder of an abapGit project.
    pub const FOLDER: &str = "folder";
    /// `{ $logic }`: the folder logic of an abapGit project.
    pub const LOGIC: &str = "logic";
    /// `{ $objects }`: number of objects of an abapGit project.
    pub const OBJECTS: &str = "objects";
    /// `{ $packages }`: number of packages of an abapGit project.
    pub const PACKAGES: &str = "packages";
}

/// Largest project file that is read; a bigger one is refused.
pub const PROJECT_FILE_MAX_BYTES: u64 = 1_048_576;
/// File in the data folder that holds the settings the UI saved (ADR 0103).
pub const UI_SETTINGS_FILE: &str = "ui-settings.json";
/// Largest settings file that is read; anything bigger is ignored.
pub const UI_SETTINGS_MAX_BYTES: u64 = 4096;
/// Header of the table of the configuration layer made from the saved memory limits.
pub const UI_LAYER_HEADER: &str = "[level2.indexer_memory_mb]";
/// Between a key and its value in a line of that layer.
pub const UI_LAYER_ASSIGN: &str = " = ";
/// Ending of a line of that layer.
pub const UI_LAYER_NEWLINE: &str = "\n";
/// Around the name of a table in the text of that layer.
pub const UI_LAYER_TABLE: (&str, &str) = ("[", "]");
/// Separates a table from its key in the dotted path of a setting.
pub const UI_KEY_SEPARATOR: char = '.';
/// The tables of the configuration whose numbers and switches the UI may save, and that every
/// scan, check and export reads again (ADR 0118): they count from the next one.
pub const UI_EDIT_SCAN_TABLES: [&str; 4] = ["scan", "conventions", "similarity", "export"];
/// The tables the UI may save that the running program reads when it starts.
pub const UI_EDIT_START_TABLES: [&str; 8] = [
    "metrics",
    "mcp",
    "snapshots",
    "watcher",
    "annotations",
    "graph",
    "compare",
    "keys",
];

/// Values the UI never changes although they are numbers in an editable table: they delete kept
/// data (old scans, dismissed alerts), so they stay with the configuration file (ADR 0118).
pub const UI_EDIT_LOCKED_KEYS: [&str; 2] = [
    "snapshots.retention_count",
    "annotations.dismissed_retention_days",
];

/// The field of the answer of `rules` that counts the violations the held state does not have.
pub const NEW_VIOLATIONS: &str = "new";

/// The folder of the log files inside the data folder (ADR 0115).
pub const LOG_DIR: &str = "logs";
/// The active log file inside [`LOG_DIR`]; older ones carry `.1`, `.2` and so on behind it.
pub const LOG_FILE: &str = "codeskimr.log";
/// The size at which the active log file is renamed and a new one starts.
pub const LOG_FILE_MAX_BYTES: u64 = 10 * 1024 * 1024;
/// How many log files exist at most, the active one included.
pub const LOG_FILES_KEPT: u32 = 5;
/// How much of the end of the log file the UI can read at once.
pub const LOG_TAIL_BYTES: u64 = 512 * 1024;
/// The most lines one read of the log gives.
pub const LOG_TAIL_MAX_LINES: usize = 2000;
/// How long a level other than the default stays; then the default counts again.
pub const LOG_LEVEL_HOLD: Duration = Duration::from_secs(24 * 60 * 60);
/// Permission bits of a log file on Unix: the owner only.
pub const LOG_FILE_MODE: u32 = 0o600;
/// Separates the name of the log file from the number of an old one: `codeskimr.log.1`.
pub const LOG_OLD_SEPARATOR: &str = ".";
/// Name of the thread that takes the default log level back.
pub const LOG_RESET_THREAD: &str = "log-level-reset";
/// The start of the targets (crate names) the chosen log level applies to. Libraries stay at the
/// default level, so that a level of Debug or Trace does not log their requests and headers.
pub const LOG_TARGET_PREFIX: &str = "codeskimr";
/// Permission bits of the folder of the log files on Unix: the owner only.
pub const LOG_DIR_MODE: u32 = 0o700;
/// The log levels the UI offers, from silent to everything.
pub const LOG_LEVELS: [tracing_subscriber::filter::LevelFilter; 5] = [
    tracing_subscriber::filter::LevelFilter::OFF,
    tracing_subscriber::filter::LevelFilter::WARN,
    tracing_subscriber::filter::LevelFilter::INFO,
    tracing_subscriber::filter::LevelFilter::DEBUG,
    tracing_subscriber::filter::LevelFilter::TRACE,
];

/// Most bytes read from `.abapgit.xml`; the file is a few hundred bytes long.
pub const ABAPGIT_CONFIG_MAX_BYTES: u64 = 65_536;
/// Most `package.devc.xml` files the explorer reads for the descriptions of the packages.
pub const ABAPGIT_PACKAGE_FILES_MAX: usize = 5_000;
