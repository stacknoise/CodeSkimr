//! Fixed values of the indexer adapter.

use std::time::Duration;

/// Environment variables every indexer gets (compared without regard to case, because Windows
/// does not distinguish). Everything else of the application's environment is dropped; the
/// language pack adds what its indexer needs (`env_allow`).
pub const BASE_ENV_ALLOW: &[&str] = &[
    "PATH",
    "PATHEXT",
    "SYSTEMROOT",
    "SYSTEMDRIVE",
    "WINDIR",
    "COMSPEC",
    "PROGRAMFILES",
    "PROGRAMFILES(X86)",
    "PROGRAMDATA",
    "LANG",
    "LC_ALL",
    "LC_CTYPE",
];
/// Variables that name the home of the user. The indexer gets an empty folder instead, so a tool
/// that looks for `~/.ssh`, `~/.aws`, `~/.npmrc` or `~/.git-credentials` finds nothing; what a tool
/// needs from the real home is named by the pack (`home_dirs`).
pub const HOME_ENV: &[&str] = &["HOME", "USERPROFILE"];
/// Variables that name a folder of settings below the home on Windows, with the folder below the
/// empty home they point to.
pub const HOME_SETTINGS_ENV: &[(&str, &[&str])] = &[
    ("APPDATA", &["AppData", "Roaming"]),
    ("LOCALAPPDATA", &["AppData", "Local"]),
];
/// A variable whose name contains one of these parts never reaches an indexer, even if a language
/// pack lists it: the name says it holds a secret.
pub const SECRET_NAME_PARTS: &[&str] = &[
    "TOKEN",
    "SECRET",
    "PASSWORD",
    "PASSWD",
    "CREDENTIAL",
    "API_KEY",
    "APIKEY",
    "PRIVATE_KEY",
    "AUTH",
];
/// A variable that starts with this belongs to the application and never reaches an indexer.
pub const APPLICATION_ENV_PREFIX: &str = "CODESKIMR";
/// Variables that name the temporary folder; they point into the output folder of the run.
pub const TEMP_ENV: &[&str] = &["TMPDIR", "TEMP", "TMP"];
/// Variables every indexer gets with a fixed value. On Windows the first one stops tools that the
/// indexer starts from taking a program of the working folder (the project) before the `PATH`.
pub const FIXED_ENV: &[(&str, &str)] = &[("NoDefaultCurrentDirectoryInExePath", "1")];
/// Most characters of the version output that an error or the tool check keeps.
pub const VERSION_EXCERPT_CHARS: usize = 200;
/// Name of the `PATH` variable.
pub const PATH_ENV: &str = "PATH";

/// Prefix of the temporary output folder inside the data directory.
pub const OUTPUT_DIR_PREFIX: &str = "level2-";
/// Name of the index file inside the output folder.
pub const INDEX_FILE_NAME: &str = "index.scip";
/// Name of the empty home folder of the indexer, inside the output folder.
pub const HOME_DIR_NAME: &str = "home";
/// Name of the folder where the indexer may write besides its index (cache, logs).
pub const SCRATCH_DIR_NAME: &str = "scratch";

/// Endings a program may have, tried in order when it is looked up in `PATH`.
pub const EXECUTABLE_SUFFIXES: &[&str] = if cfg!(windows) {
    &[".exe", ".cmd", ".bat"]
} else {
    &[""]
};

/// How long a loop waits before it looks at the child again.
pub const POLL_INTERVAL: Duration = Duration::from_millis(20);
/// How often memory and output folder are measured; both cost more than a look at the child.
pub const MEASURE_INTERVAL: Duration = Duration::from_millis(250);
/// How long the version probe of a tool may take.
pub const PROBE_TIMEOUT: Duration = Duration::from_secs(20);
/// Memory a version probe may use, in bytes (1 GiB).
pub const PROBE_MAX_MEMORY: u64 = 1024 * 1024 * 1024;
/// Output a version probe may write, in bytes.
pub const PROBE_MAX_OUTPUT: u64 = 64 * 1024;
/// How long to wait for the readers of the console after the child ended.
pub const DRAIN_GRACE: Duration = Duration::from_millis(500);
/// Size of one read from a console pipe.
pub const READ_CHUNK: usize = 8 * 1024;
/// Most bytes of each console stream that are kept (the end of it).
pub const CAPTURE_BYTES: usize = 32 * 1024;
/// Most bytes of the console that go into a log line or an error.
pub const EXCERPT_BYTES: usize = 2 * 1024;
/// Deepest folder level that is measured in the output folder.
pub const MAX_SCAN_DEPTH: usize = 32;
/// Name of the log field that carries the redacted excerpt.
pub const LOG_EXCERPT: &str = "excerpt";
/// Name of the log field that carries the indexer name.
pub const LOG_INDEXER: &str = "indexer";
/// Name of the log field that carries the error code.
pub const LOG_CODE: &str = "code";
/// Joins the console streams of a version probe.
pub const WORD_SEPARATOR: &str = " ";

/// Name of the note in a run folder that says which application instance owns it.
pub const OWNER_NOTE_NAME: &str = "owner";
/// Name of the note in a run folder that says which process the run started.
pub const INDEXER_NOTE_NAME: &str = "indexer";
/// Separates the process id from its start time in a note.
pub const NOTE_SEPARATOR: char = ' ';
/// How long a run folder without a note is left alone: it may just have been created.
pub const ORPHAN_GRACE: Duration = Duration::from_secs(60);
/// Reason of an error: the list of running processes cannot be read.
pub const NO_PROCESS_LIST: &str = "no process list";
/// Reason of an error: a process to note does not exist.
pub const NO_SUCH_PROCESS: &str = "no such process";
/// How many seconds the start time of a process may differ from the one in a note: the start
/// time is computed from the boot time of the system, which can shift a little.
pub const START_TOLERANCE_SECONDS: u64 = 2;
