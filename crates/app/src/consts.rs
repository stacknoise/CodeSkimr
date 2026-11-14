//! Fixed values of the composition root: environment variables, folder and file names.

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

/// The `SQLite` database inside the data folder.
pub const DATABASE_FILE: &str = "codeskimr.db";
/// The folder of the Git clones inside the data folder.
pub const GIT_CACHE_DIR: &str = "git";
/// The user configuration file inside the configuration folder.
pub const USER_CONFIG_FILE: &str = "config.toml";
/// The folder of the language packs next to the executable or the current folder.
pub const LANGUAGES_DIR: &str = "languages";

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
/// Separates the parts of a URL path.
pub const URL_PATH_SEPARATOR: char = '/';
/// Separates host and path in an scp-style Git address.
pub const SCP_PATH_SEPARATOR: char = ':';

/// Bytes in a megabyte, for the size limit of a source.
pub const BYTES_PER_MB: u64 = 1_048_576;
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

/// Name under which the installation token lives in the credential store (blueprint 12.2).
pub const TOKEN_CREDENTIAL_ID: &str = "installation-token";
/// Whether `name` is the installation token under any spelling a keystore could fold together
/// (case, surrounding blanks): such a name must never be managed or lent to a remote.
#[must_use]
pub fn is_token_credential(name: &str) -> bool {
    name.trim().eq_ignore_ascii_case(TOKEN_CREDENTIAL_ID)
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
/// Largest answer of the local server that is read, in bytes.
pub const MAX_REPLY_BYTES: u64 = 65_536;
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

/// Largest file that is read for the version a project requires (`global.json`, `.nvmrc` and so
/// on), in bytes.
pub const REQUIREMENT_MAX_BYTES: u64 = 65_536;

/// Characters besides the control characters that can hide or reorder text on a terminal: the
/// direction marks and overrides, the zero-width characters, the line and paragraph separators
/// and the byte order mark. A name that holds one is not shown in a prompt.
pub const INVISIBLE_RANGES: [(char, char); 7] = [
    ('\u{061C}', '\u{061C}'),
    ('\u{200B}', '\u{200F}'),
    ('\u{2028}', '\u{202E}'),
    ('\u{2060}', '\u{2064}'),
    ('\u{2066}', '\u{2069}'),
    ('\u{FEFF}', '\u{FEFF}'),
    ('\u{00AD}', '\u{00AD}'),
];
