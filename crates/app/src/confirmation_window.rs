//! The adapter that opens the console window of the level 2 confirmation (ADR 0071, decision 5).
//!
//! It is the only place in the product that starts a process for a request of the browser, and
//! what it starts is fixed: the running program itself (`std::env::current_exe`, an absolute path,
//! never a lookup in `PATH`), with the arguments `level2 confirm --from-ui <id>` and the global
//! options the server was started with, each as its own argument. No shell sees any of it. On
//! Windows that is a new console window; on macOS `osascript` runs a constant script that opens
//! Terminal, and the values are its arguments, never part of its text. On every other system
//! nothing is opened and the screen shows the command to copy.
//!
//! The program that is started asks the owner to type the name of the project. Nothing that this
//! adapter or the request holds can carry that consent.

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::{Mutex, PoisonError};

use codeskimr_application::error::CodedError;
use codeskimr_application::level2::Launch;
use codeskimr_application::ports::{ConfirmationWindow, WindowOutcome};
use codeskimr_cli::GlobalOptions;
use codeskimr_codes::ErrorCode;
use codeskimr_domain::ProjectId;

use crate::consts::{
    CONFIRM_WINDOW_WORDS, DESKTOP_SESSION_VARIABLES, ENV_TRAY_CHILD, OSASCRIPT_END_OF_OPTIONS,
    OSASCRIPT_PROGRAM, OSASCRIPT_SCRIPT_OPTION, REMOTE_SESSION_VARIABLES, TERMINAL_SCRIPT,
};

/// Whether this system has a terminal program the server may start (Windows and macOS).
const HAS_TERMINAL: bool = cfg!(any(windows, target_os = "macos"));
/// Whether the program can see that a window it opened is still open. On Windows it holds the
/// process; on macOS `osascript` ends at once and says nothing about the window.
const SEES_WINDOW: bool = cfg!(windows);

/// Opens the confirmation window of this program.
pub struct ConsoleWindow {
    program: Option<PathBuf>,
    options: Vec<OsString>,
    open: Mutex<BTreeMap<u64, Child>>,
}

impl ConsoleWindow {
    /// An adapter for the program that is running now, which passes `global` on to the window.
    pub fn new(global: &GlobalOptions) -> Self {
        Self {
            program: std::env::current_exe().ok(),
            options: global.arguments(),
            open: Mutex::new(BTreeMap::new()),
        }
    }

    /// What this system can do: open a window, or leave it to the screen to show the command.
    /// Decided from the system and the session variables, never by trying.
    pub fn launch(&self, has_variable: &dyn Fn(&str) -> bool) -> Launch {
        let remote = REMOTE_SESSION_VARIABLES
            .iter()
            .any(|name| has_variable(name));
        let desktop = DESKTOP_SESSION_VARIABLES
            .iter()
            .any(|name| has_variable(name));
        if HAS_TERMINAL && desktop && !remote && self.program.is_some() {
            Launch::Window
        } else {
            Launch::Copy
        }
    }

    /// The command of the system, ready to start; `None` where there is no terminal.
    fn command(&self, project: ProjectId) -> Option<Command> {
        let program = self.program.as_deref()?;
        let arguments = confirm_arguments(project, &self.options);
        if cfg!(windows) {
            Some(console_command(program, &arguments))
        } else if cfg!(target_os = "macos") {
            Some(terminal_command(program, &arguments))
        } else {
            None
        }
    }

    /// Whether the window of an earlier request for `project` is still open.
    fn still_open(&self, project: ProjectId) -> bool {
        if !SEES_WINDOW {
            return false;
        }
        let mut open = self.open.lock().unwrap_or_else(PoisonError::into_inner);
        let running = open
            .get_mut(&project.get())
            .is_some_and(|child| matches!(child.try_wait(), Ok(None)));
        if !running {
            open.remove(&project.get());
        }
        running
    }

    /// Keeps the process where the program can see it end, or lets a thread reap it.
    fn keep(&self, project: ProjectId, mut child: Child) {
        if SEES_WINDOW {
            self.open
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .insert(project.get(), child);
        } else {
            // The starter of the terminal ends at once; a thread collects it so that none stays
            // behind. A thread that cannot start leaves it to the end of the program.
            std::thread::Builder::new()
                .spawn(move || child.wait().ok())
                .ok();
        }
    }
}

impl ConfirmationWindow for ConsoleWindow {
    fn open(&self, project: ProjectId) -> Result<WindowOutcome, CodedError> {
        if self.still_open(project) {
            return Ok(WindowOutcome::AlreadyOpen);
        }
        let Some(mut command) = self.command(project) else {
            return Ok(WindowOutcome::Unavailable);
        };
        let child = command.spawn().map_err(|error| {
            // The cause stays in the log; the screen shows the command instead.
            tracing::debug!(cause = %error);
            CodedError::new(ErrorCode::ConfirmWindowFailed)
        })?;
        self.keep(project, child);
        Ok(WindowOutcome::Opened)
    }
}

/// The arguments of the program in the window: the fixed words, the numeric id that the server
/// writes itself from the project record, and the global options of this run.
pub(crate) fn confirm_arguments(project: ProjectId, options: &[OsString]) -> Vec<OsString> {
    CONFIRM_WINDOW_WORDS
        .iter()
        .map(OsString::from)
        .chain([OsString::from(project.get().to_string())])
        .chain(options.iter().cloned())
        .collect()
}

/// Windows: the program itself in a new console window. No `cmd.exe`, no shell, nothing to quote.
pub(crate) fn console_command(program: &Path, arguments: &[OsString]) -> Command {
    let mut command = Command::new(program);
    command.args(arguments).env_remove(ENV_TRAY_CHILD);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(crate::consts::CREATE_NEW_CONSOLE);
    }
    command
}

/// macOS: `osascript` runs the constant script, and the program and its arguments are the
/// arguments of the script.
pub(crate) fn terminal_command(program: &Path, arguments: &[OsString]) -> Command {
    let mut command = Command::new(OSASCRIPT_PROGRAM);
    command
        .arg(OSASCRIPT_SCRIPT_OPTION)
        .arg(TERMINAL_SCRIPT)
        .arg(OSASCRIPT_END_OF_OPTIONS)
        .arg(program)
        .args(arguments)
        .env_remove(ENV_TRAY_CHILD);
    command
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;

    use super::*;

    fn args(command: &Command) -> Vec<String> {
        command
            .get_args()
            .map(|argument| argument.to_string_lossy().into_owned())
            .collect()
    }

    fn global() -> GlobalOptions {
        GlobalOptions {
            data_dir: Some(PathBuf::from("/data dir/with 'quote")),
            overrides: vec![("level2.timeout_seconds".to_owned(), "9".to_owned())],
            ..GlobalOptions::default()
        }
    }

    #[test]
    fn the_arguments_are_fixed_words_the_numeric_id_and_the_global_options_as_a_list() {
        let window = ConsoleWindow::new(&global());
        let arguments = confirm_arguments(ProjectId::new(42), &window.options);
        let seen: Vec<String> = arguments
            .iter()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert_eq!(
            seen,
            [
                "level2",
                "confirm",
                "--from-ui",
                "42",
                "--data-dir",
                "/data dir/with 'quote",
                "--set",
                "level2.timeout_seconds=9"
            ]
        );
    }

    #[test]
    fn windows_starts_the_program_itself_without_a_shell_and_without_the_tray_mark() {
        let arguments = confirm_arguments(ProjectId::new(3), &[]);
        let command = console_command(Path::new("/opt/codeskimr"), &arguments);
        assert_eq!(command.get_program(), OsStr::new("/opt/codeskimr"));
        assert_eq!(args(&command), ["level2", "confirm", "--from-ui", "3"]);
        let removed = command
            .get_envs()
            .any(|(name, value)| name == ENV_TRAY_CHILD && value.is_none());
        assert!(removed, "the tray mark must not reach the window");
    }

    #[test]
    fn macos_runs_a_constant_script_and_hands_every_value_over_as_an_argument() {
        let hostile = OsString::from("x'; do shell script \"touch /tmp/pwned\"; $(id) `id` '");
        let arguments = confirm_arguments(
            ProjectId::new(5),
            &[OsString::from("--config"), hostile.clone()],
        );
        let tricky = Path::new("/Apps/My \"Code\" 'Skimr' $HOME `id`/codeskimr");
        let command = terminal_command(tricky, &arguments);
        assert_eq!(command.get_program(), OsStr::new(OSASCRIPT_PROGRAM));
        let seen = args(&command);
        assert_eq!(seen[0], OSASCRIPT_SCRIPT_OPTION);
        // The script text is the constant and nothing else; no value is part of it.
        assert_eq!(seen[1], TERMINAL_SCRIPT);
        assert!(!TERMINAL_SCRIPT.contains("touch"));
        // Every item of argv is quoted alone, and nothing is added to the line but the quotes.
        assert!(TERMINAL_SCRIPT.contains("quoted form of (piece as text)"));
        assert_eq!(TERMINAL_SCRIPT.matches("quoted form of").count(), 1);
        assert!(!TERMINAL_SCRIPT.contains('$') && !TERMINAL_SCRIPT.contains('`'));
        assert_eq!(seen[2], OSASCRIPT_END_OF_OPTIONS);
        assert_eq!(seen[3], tricky.to_string_lossy());
        assert_eq!(&seen[4..8], ["level2", "confirm", "--from-ui", "5"]);
        assert_eq!(seen[9], hostile.to_string_lossy());
        assert_eq!(seen.len(), 10);
    }

    #[test]
    fn the_id_is_only_digits_whatever_the_project_is_called() {
        let arguments = confirm_arguments(ProjectId::new(u64::MAX), &[]);
        let id = arguments[3].to_string_lossy();
        assert!(id.chars().all(|c| c.is_ascii_digit()), "{id}");
    }

    #[test]
    fn only_a_desktop_session_that_is_not_remote_gets_a_window() {
        let window = ConsoleWindow::new(&GlobalOptions::default());
        let desktop = |name: &str| DESKTOP_SESSION_VARIABLES.contains(&name);
        let expected = if HAS_TERMINAL {
            Launch::Window
        } else {
            Launch::Copy
        };
        assert_eq!(window.launch(&desktop), expected);
        // When the check cannot decide, the screen shows the command.
        assert_eq!(window.launch(&|_: &str| false), Launch::Copy);
        for name in REMOTE_SESSION_VARIABLES {
            let remote = |asked: &str| asked == name || DESKTOP_SESSION_VARIABLES.contains(&asked);
            assert_eq!(window.launch(&remote), Launch::Copy, "{name}");
        }
    }

    #[test]
    fn without_a_terminal_program_the_adapter_opens_nothing_and_says_so() {
        if HAS_TERMINAL {
            return;
        }
        let window = ConsoleWindow::new(&GlobalOptions::default());
        assert_eq!(
            window.open(ProjectId::new(1)).unwrap(),
            WindowOutcome::Unavailable
        );
    }
}
