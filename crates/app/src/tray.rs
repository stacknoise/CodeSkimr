//! The start in the system tray (ADR 0067), the part that works on every system.
//!
//! The bare `codeskimr` on Windows and macOS starts a second copy of itself without a console
//! window and ends, so that the terminal can be closed or used again. The copy is marked by an
//! environment variable, writes its log to a file in the data folder and shows the tray icon (see
//! `tray_ui`). `--console` and every other system keep the old way: server and browser in the
//! window that started them.

use std::ffi::OsString;
use std::fs::{File, OpenOptions, create_dir_all};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use codeskimr_application::error::CodedError;
#[cfg(any(windows, target_os = "macos"))]
use codeskimr_cli::OpenArgs;
use codeskimr_cli::{GlobalOptions, StartArgs, TrayLabels};
#[cfg(any(windows, target_os = "macos"))]
use codeskimr_codes::Coded;
use codeskimr_codes::ErrorCode;

use crate::consts::{ENV_TRAY_CHILD, TRAY_CHILD_ON, TRAY_LOG_FILE};
use crate::locations::{Locations, Lookup};
#[cfg(any(windows, target_os = "macos"))]
use crate::{browser, open};

/// Whether this system has a tray start at all.
pub(crate) const AVAILABLE: bool = cfg!(any(windows, target_os = "macos"));

/// What the bare start does on this run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Start {
    /// Server and browser in this window.
    Console,
    /// Start the background copy and end.
    Detach,
    /// This is the background copy: server and tray icon.
    Tray,
}

/// Decides the start: the tray where the system has one, unless `--console` was given.
pub(crate) fn mode(args: StartArgs, env: Lookup<'_>) -> Start {
    if !AVAILABLE {
        return Start::Console;
    }
    if env(ENV_TRAY_CHILD).is_some() {
        return Start::Tray;
    }
    if args.console {
        Start::Console
    } else {
        Start::Detach
    }
}

/// The log file of the tray program.
pub(crate) fn log_path(locations: &Locations) -> PathBuf {
    locations.data_dir.join(TRAY_LOG_FILE)
}

/// Opens the log file empty, and the data folder first if it does not exist yet.
fn open_log(path: &Path) -> std::io::Result<File> {
    if let Some(folder) = path.parent() {
        create_dir_all(folder)?;
    }
    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
}

/// The command that starts `program` again with `arguments`, marked as the tray copy, with no
/// console window, no input and its log in `log`.
pub(crate) fn detached_command(
    program: &Path,
    arguments: impl IntoIterator<Item = OsString>,
    log: File,
) -> Command {
    let mut command = Command::new(program);
    command
        .args(arguments)
        .env(ENV_TRAY_CHILD, TRAY_CHILD_ON)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::from(log));
    detach_from_terminal(&mut command);
    command
}

#[cfg(windows)]
fn detach_from_terminal(command: &mut Command) {
    use std::os::windows::process::CommandExt;
    command.creation_flags(crate::consts::CREATE_NO_WINDOW);
}

#[cfg(unix)]
fn detach_from_terminal(command: &mut Command) {
    use std::os::unix::process::CommandExt;
    // A group of its own: closing the terminal does not take the program with it.
    command.process_group(0);
}

/// Starts the background copy of this program with the arguments of this run and returns the
/// place of its log.
///
/// # Errors
/// `APP-603` if the log cannot be created or the program cannot be started again.
pub(crate) fn detach(
    locations: &Locations,
    arguments: impl IntoIterator<Item = OsString>,
) -> Result<PathBuf, CodedError> {
    let failed = |_| CodedError::new(ErrorCode::TrayStartFailed);
    let log = log_path(locations);
    let file = open_log(&log).map_err(failed)?;
    let program = std::env::current_exe().map_err(failed)?;
    detached_command(&program, arguments, file)
        .spawn()
        .map_err(failed)?;
    Ok(log)
}

/// Shows the tray icon and keeps the program in it until the user picks "Quit". The browser opens
/// once at the start and again for every "Open" in the menu.
///
/// # Errors
/// `APP-603` if the icon cannot be shown.
#[cfg(any(windows, target_os = "macos"))]
pub(crate) fn show(
    global: &GlobalOptions,
    locations: &Locations,
    labels: &TrayLabels,
) -> Result<(), CodedError> {
    std::thread::scope(|threads| {
        let open = || {
            threads.spawn(|| open_browser(global, locations));
        };
        open();
        let log = || {
            let _ = browser::open(&log_path(locations));
        };
        crate::tray_ui::run(labels, open, log)
    })
}

/// Without a tray on this system the tray copy is never started; this only keeps the call valid.
///
/// # Errors
/// Always `APP-603`.
#[cfg(not(any(windows, target_os = "macos")))]
pub(crate) fn show(
    _global: &GlobalOptions,
    _locations: &Locations,
    _labels: &TrayLabels,
) -> Result<(), CodedError> {
    Err(CodedError::new(ErrorCode::TrayStartFailed))
}

/// Gives the browser a launch code. A failure is not reported anywhere the user would see it, so it
/// goes to the log.
#[cfg(any(windows, target_os = "macos"))]
fn open_browser(global: &GlobalOptions, locations: &Locations) {
    let env = |name: &str| std::env::var(name).ok();
    let args = OpenArgs { no_browser: false };
    if let Err(error) = open::run(args, global, locations, &env, |_| ()) {
        tracing::warn!(code = error.code().id());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn no_env(_: &str) -> Option<String> {
        None
    }

    fn tray_env(name: &str) -> Option<String> {
        (name == ENV_TRAY_CHILD).then(|| TRAY_CHILD_ON.to_owned())
    }

    #[test]
    fn the_console_flag_keeps_the_window_where_a_tray_exists() {
        let console = StartArgs { console: true };
        assert_eq!(mode(console, &no_env), Start::Console);
    }

    #[test]
    fn the_bare_start_detaches_only_where_a_tray_exists() {
        let expected = if AVAILABLE {
            Start::Detach
        } else {
            Start::Console
        };
        assert_eq!(mode(StartArgs::default(), &no_env), expected);
    }

    #[test]
    fn the_marked_copy_runs_the_tray() {
        let expected = if AVAILABLE {
            Start::Tray
        } else {
            Start::Console
        };
        assert_eq!(mode(StartArgs::default(), &tray_env), expected);
    }

    #[test]
    fn the_background_copy_gets_the_arguments_the_mark_and_the_log() {
        let folder = tempfile::tempdir().unwrap();
        let log = open_log(&folder.path().join("nested").join("run.log")).unwrap();
        let arguments = vec![OsString::from("--data-dir"), OsString::from("x")];
        let command = detached_command(Path::new("codeskimr"), arguments.clone(), log);
        let seen: Vec<OsString> = command.get_args().map(OsString::from).collect();
        assert_eq!(seen, arguments);
        let marked = command
            .get_envs()
            .any(|(name, value)| name == ENV_TRAY_CHILD && value.is_some());
        assert!(marked);
        assert!(folder.path().join("nested").join("run.log").exists());
    }

    #[test]
    fn the_log_starts_empty_with_every_start() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("run.log");
        std::fs::write(&path, "old").unwrap();
        open_log(&path).unwrap();
        assert_eq!(std::fs::metadata(&path).unwrap().len(), 0);
    }
}
