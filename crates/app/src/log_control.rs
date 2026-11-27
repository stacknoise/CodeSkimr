//! The log level that can change while the program runs, and the log file of the data folder
//! (ADR 0113).
//!
//! The subscriber is installed once; this module keeps the handle that changes its level and the
//! file that every log line is copied to. A level other than the default falls back to the
//! default after [`LOG_LEVEL_HOLD`], so that a forgotten switch cannot fill the disk.

use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard, OnceLock, PoisonError};
use std::time::{Duration, Instant};

use tracing_subscriber::Registry;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::reload::Handle;

use crate::consts::{LOG_DIR, LOG_FILE, LOG_LEVEL_HOLD, LOG_RESET_THREAD};
use crate::log_file::LogFile;

/// The levels the UI offers, from silent to everything.
pub const LEVELS: [LevelFilter; 5] = [
    LevelFilter::OFF,
    LevelFilter::WARN,
    LevelFilter::INFO,
    LevelFilter::DEBUG,
    LevelFilter::TRACE,
];

/// The handle of the installed subscriber and the level that counts when nothing was chosen.
struct Control {
    handle: Handle<LevelFilter, Registry>,
    default: LevelFilter,
}

static CONTROL: OnceLock<Control> = OnceLock::new();
static FILE: Mutex<Option<LogFile>> = Mutex::new(None);
/// When the chosen level ends, if it is not the default.
static DEADLINE: Mutex<Option<Instant>> = Mutex::new(None);
/// Counts the choices, so that the timer of an older one does nothing.
static CHOICES: AtomicU64 = AtomicU64::new(0);

fn locked<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Remembers the handle of the subscriber that was installed and its starting level.
pub fn register(handle: Handle<LevelFilter, Registry>, default: LevelFilter) {
    let _ = CONTROL.set(Control { handle, default });
}

/// The level that counts now.
#[must_use]
pub fn level() -> LevelFilter {
    CONTROL
        .get()
        .and_then(|control| control.handle.clone_current())
        .unwrap_or(crate::consts::DEFAULT_LOG_LEVEL)
}

/// The level that counts when nothing was chosen.
#[must_use]
pub fn default_level() -> LevelFilter {
    CONTROL
        .get()
        .map_or(crate::consts::DEFAULT_LOG_LEVEL, |control| control.default)
}

/// Whether a subscriber that this module controls is installed.
#[must_use]
pub fn is_installed() -> bool {
    CONTROL.get().is_some()
}

/// Changes the level at once. A level other than the default ends after [`LOG_LEVEL_HOLD`].
/// Returns `false` if another subscriber owns the log, so that nothing could be changed.
pub fn set_level(next: LevelFilter) -> bool {
    let Some(control) = CONTROL.get() else {
        return false;
    };
    let choice = CHOICES.fetch_add(1, Ordering::SeqCst) + 1;
    if control.handle.modify(|filter| *filter = next).is_err() {
        return false;
    }
    let deadline = (next != control.default).then(|| Instant::now() + LOG_LEVEL_HOLD);
    *locked(&DEADLINE) = deadline;
    if deadline.is_some() {
        spawn_reset(choice, LOG_LEVEL_HOLD);
    }
    true
}

/// Starts the timer that takes the default level back, unless another level was chosen by then.
fn spawn_reset(choice: u64, after: Duration) {
    let _ = std::thread::Builder::new()
        .name(String::from(LOG_RESET_THREAD))
        .spawn(move || {
            std::thread::sleep(after);
            if CHOICES.load(Ordering::SeqCst) == choice {
                reset();
            }
        });
}

/// Puts the default level back.
fn reset() {
    if let Some(control) = CONTROL.get() {
        let _ = control.handle.modify(|filter| *filter = control.default);
    }
    *locked(&DEADLINE) = None;
}

/// How long the chosen level lasts, if it is not the default.
#[must_use]
pub fn resets_in() -> Option<Duration> {
    let deadline = (*locked(&DEADLINE))?;
    Some(deadline.saturating_duration_since(Instant::now()))
}

/// The path the log file has inside `data_dir`.
#[must_use]
pub fn file_in(data_dir: &Path) -> PathBuf {
    data_dir.join(LOG_DIR).join(LOG_FILE)
}

/// Starts copying the log into the file of `data_dir`. A file that cannot be opened leaves the log
/// on standard error only.
pub fn attach_file(data_dir: &Path) {
    *locked(&FILE) = LogFile::open(&file_in(data_dir)).ok();
}

/// Copies one log line into the file, if there is one. A failure is ignored: the log must never
/// stop the program.
pub fn write_file(line: &[u8]) {
    if let Some(file) = locked(&FILE).as_mut() {
        let _ = file.write(line);
    }
}

/// Runs `action` on the log file; `None` if no file is attached.
pub fn with_file<T>(action: impl FnOnce(&mut LogFile) -> io::Result<T>) -> Option<io::Result<T>> {
    locked(&FILE).as_mut().map(action)
}

/// The path and the size of the attached file.
#[must_use]
pub fn file_info() -> Option<(PathBuf, u64)> {
    locked(&FILE)
        .as_ref()
        .map(|file| (file.path().to_path_buf(), file.size()))
}
