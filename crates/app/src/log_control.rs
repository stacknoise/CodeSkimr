//! The log level that can change while the program runs, and the log file of the data folder
//! (ADR 0115).
//!
//! The subscriber is installed once; this module keeps the handle that changes its level and the
//! file that every log line is copied to. A level other than the default falls back to the
//! default after [`LOG_LEVEL_HOLD`], so that a forgotten switch cannot fill the disk.

use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard, OnceLock, PoisonError};
use std::time::{Duration, Instant};

use tracing_subscriber::Registry;
use tracing_subscriber::filter::{LevelFilter, Targets};
use tracing_subscriber::reload::Handle;

use crate::consts::{
    DEFAULT_LOG_LEVEL, LOG_DIR, LOG_FILE, LOG_LEVEL_HOLD, LOG_RESET_THREAD, LOG_TARGET_PREFIX,
};
use crate::log_file::LogFile;

/// The filter for a chosen level: the program itself logs at `level`, libraries at most at the
/// default level, so that Debug and Trace do not show what their requests carry.
#[must_use]
pub fn filter_for(level: LevelFilter) -> Targets {
    Targets::new()
        .with_target(LOG_TARGET_PREFIX, level)
        .with_default(level.min(DEFAULT_LOG_LEVEL))
}

/// The filter of the start: the level of `CODESKIMR_LOG` (or the default) counts for every target,
/// as it always did. Whoever sets the variable asked for it.
#[must_use]
pub fn startup_filter(level: LevelFilter) -> Targets {
    Targets::new().with_default(level)
}

/// The handle of the installed subscriber and the level that counts when nothing was chosen.
struct Control {
    handle: Handle<Targets, Registry>,
    default: LevelFilter,
}

static CONTROL: OnceLock<Control> = OnceLock::new();
static FILE: Mutex<Option<LogFile>> = Mutex::new(None);
/// The level that counts now; `None` until the subscriber is registered.
static CURRENT: Mutex<Option<LevelFilter>> = Mutex::new(None);
/// When the chosen level ends, if it is not the default.
static DEADLINE: Mutex<Option<Instant>> = Mutex::new(None);
/// Whether the one thread that takes the default level back is running.
static TIMER_RUNNING: AtomicBool = AtomicBool::new(false);

fn locked<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Remembers the handle of the subscriber that was installed and its starting level.
pub fn register(handle: Handle<Targets, Registry>, default: LevelFilter) {
    if CONTROL.set(Control { handle, default }).is_ok() {
        *locked(&CURRENT) = Some(default);
    }
}

/// The level that counts now.
#[must_use]
pub fn level() -> LevelFilter {
    (*locked(&CURRENT)).unwrap_or(DEFAULT_LOG_LEVEL)
}

/// The level that counts when nothing was chosen.
#[must_use]
pub fn default_level() -> LevelFilter {
    CONTROL
        .get()
        .map_or(DEFAULT_LOG_LEVEL, |control| control.default)
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
    let filter = if next == control.default {
        startup_filter(next)
    } else {
        filter_for(next)
    };
    if control.handle.reload(filter).is_err() {
        return false;
    }
    *locked(&CURRENT) = Some(next);
    let deadline = (next != control.default).then(|| Instant::now() + LOG_LEVEL_HOLD);
    *locked(&DEADLINE) = deadline;
    if deadline.is_some() {
        ensure_timer();
    }
    true
}

/// Starts the thread that takes the default level back, unless one is running. It reads the end
/// again after every sleep, so a new choice only moves the end and never starts another thread.
fn ensure_timer() {
    if TIMER_RUNNING.swap(true, Ordering::SeqCst) {
        return;
    }
    let started = std::thread::Builder::new()
        .name(String::from(LOG_RESET_THREAD))
        .spawn(run_timer);
    if started.is_err() {
        TIMER_RUNNING.store(false, Ordering::SeqCst);
    }
}

/// Sleeps until the end of the chosen level and then puts the default back; ends when no level
/// is chosen any more.
fn run_timer() {
    loop {
        let Some(deadline) = *locked(&DEADLINE) else {
            TIMER_RUNNING.store(false, Ordering::SeqCst);
            // A choice may have been made between the look and the store; it started no thread.
            if locked(&DEADLINE).is_some() && !TIMER_RUNNING.swap(true, Ordering::SeqCst) {
                continue;
            }
            return;
        };
        let left = deadline.saturating_duration_since(Instant::now());
        if left.is_zero() {
            reset();
        } else {
            std::thread::sleep(left);
        }
    }
}

/// Puts the default level back.
fn reset() {
    if let Some(control) = CONTROL.get() {
        let _ = control.handle.reload(startup_filter(control.default));
        *locked(&CURRENT) = Some(control.default);
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
