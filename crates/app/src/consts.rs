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
/// Separates the parts of a URL path.
pub const URL_PATH_SEPARATOR: char = '/';
/// Separates host and path in an scp-style Git address.
pub const SCP_PATH_SEPARATOR: char = ':';

/// Bytes in a megabyte, for the size limit of a source.
pub const BYTES_PER_MB: u64 = 1_048_576;

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
