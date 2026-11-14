//! One child process under limits: its own process group (a job object on Windows), a timeout, a
//! memory limit, an output limit and cancellation. Whatever ends the run, the whole group is
//! killed (ADR 0033, decision 11).

use std::ffi::OsString;
use std::io::{self, Read};
use std::ops::ControlFlow;
use std::path::{Path, PathBuf};
use std::process::{ExitStatus, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread;
use std::time::{Duration, Instant};

use codeskimr_application::cancel::CancelToken;
use process_wrap::std::{ChildWrapper, CommandWrap};

use crate::consts::{
    CAPTURE_BYTES, DRAIN_GRACE, KILL_POLL, KILL_WAIT, MAX_SCAN_DEPTH, MEASURE_INTERVAL,
    POLL_INTERVAL, READ_CHUNK,
};
use crate::memory::TreeMemory;
use crate::orphans;

/// The limits of one run.
#[derive(Debug, Clone, Copy)]
pub struct Limits {
    /// Longest run time.
    pub timeout: Duration,
    /// Most resident memory of the process tree, in bytes.
    pub max_memory_bytes: u64,
    /// Most bytes written in total: console plus the files of the watched folder.
    pub max_output_bytes: u64,
}

/// What to start.
#[derive(Debug, Clone)]
pub struct Launch {
    /// Absolute path of the program.
    pub program: PathBuf,
    /// Its arguments.
    pub args: Vec<OsString>,
    /// Its whole environment; nothing is inherited.
    pub env: Vec<(OsString, OsString)>,
    /// Its working folder.
    pub cwd: PathBuf,
    /// The limits.
    pub limits: Limits,
    /// A folder whose size counts toward the output limit.
    pub watch_dir: Option<PathBuf>,
    /// A folder in which the started process is noted, so that the next start of the application
    /// can end it if this one dies without cleaning up (issue #223).
    pub note_dir: Option<PathBuf>,
}

/// Why a run was stopped from outside.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stop {
    /// The timeout passed.
    Timeout,
    /// The memory limit was exceeded.
    Memory,
    /// The output limit was exceeded.
    Output,
    /// The caller cancelled.
    Cancelled,
}

/// Why a run gave no result.
#[derive(Debug)]
pub enum RunError {
    /// The process could not be started.
    Io(io::Error),
    /// The process could not be watched any more (its state cannot be read) and was killed.
    /// `survived` is set if the leader was still running after the bound for the wait.
    Watch {
        /// What went wrong.
        cause: io::Error,
        /// Whether the leader outlived the kill.
        survived: bool,
    },
    /// The run was stopped and its process group killed. `survived` is set if the leader was
    /// still running after the bound for the wait: nothing may delete what it could still use.
    Stopped {
        /// Why it was stopped.
        stop: Stop,
        /// Whether the leader outlived the kill.
        survived: bool,
    },
}

/// A run that ended by itself.
#[derive(Debug)]
pub struct Finished {
    /// Whether it ended with success.
    pub success: bool,
    /// Its exit code, absent if a signal ended it.
    pub code: Option<i32>,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

impl Finished {
    /// The end of standard output as text.
    #[must_use]
    pub fn stdout_text(&self) -> String {
        String::from_utf8_lossy(&self.stdout).into_owned()
    }

    /// The end of standard error as text.
    #[must_use]
    pub fn stderr_text(&self) -> String {
        String::from_utf8_lossy(&self.stderr).into_owned()
    }
}

/// The end of a stream, kept while it is read.
#[derive(Default)]
struct Tail(Vec<u8>);

impl Tail {
    fn push(&mut self, chunk: &[u8]) {
        self.0.extend_from_slice(chunk);
        let excess = self.0.len().saturating_sub(CAPTURE_BYTES);
        self.0.drain(..excess);
    }
}

/// Reads one stream in a thread of its own so that a full pipe never blocks the child.
struct Capture {
    tail: Arc<Mutex<Tail>>,
    done: Receiver<()>,
}

impl Capture {
    fn start(mut stream: impl Read + Send + 'static, written: Arc<AtomicU64>) -> Self {
        let tail = Arc::new(Mutex::new(Tail::default()));
        let (finished, done) = mpsc::channel();
        let shared = Arc::clone(&tail);
        thread::spawn(move || {
            let mut chunk = vec![u8::default(); READ_CHUNK];
            while let Ok(read) = stream.read(&mut chunk) {
                let Some(data) = chunk.get(..read).filter(|data| !data.is_empty()) else {
                    break;
                };
                written.fetch_add(
                    u64::try_from(data.len()).unwrap_or(u64::MAX),
                    Ordering::Relaxed,
                );
                shared
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .push(data);
            }
            // The receiver may be gone already; the tail stays readable.
            finished.send(()).ok();
        });
        Self { tail, done }
    }

    /// Waits a short while for the end of the stream, then takes what was read.
    fn take(&self) -> Vec<u8> {
        self.done.recv_timeout(DRAIN_GRACE).ok();
        std::mem::take(&mut self.tail.lock().unwrap_or_else(PoisonError::into_inner).0)
    }
}

/// Kills the group when it goes out of scope, so that no exit path leaves a process behind.
struct Group(Box<dyn ChildWrapper>);

impl Group {
    /// Kills the whole group and waits for the leader, but never without a bound.
    ///
    /// `ChildWrapper::wait` of a Windows job object ends with a blocking read of the completion
    /// port of the job, and `try_wait` (called in the watch loop) takes one message of that port
    /// per call. If the loop has taken the last message before it saw the exit of the leader,
    /// `wait` would wait for a message that never comes, so the end of the run polls instead.
    ///
    /// Returns whether the leader is confirmed gone. `false` means it was still running when the
    /// bound passed, or its state could not be read, so the end is not confirmed: the caller
    /// must then keep the run folder, whose notes let the next start find it.
    fn kill(&mut self) -> bool {
        // The group may be gone already; that is the goal, not an error.
        drop(self.0.start_kill());
        wait_until(KILL_WAIT, || match self.0.try_wait() {
            Ok(Some(_)) => ControlFlow::Break(true),
            // The state cannot be read: the end is not confirmed, so it counts as not ended.
            Err(_) => ControlFlow::Break(false),
            Ok(None) => ControlFlow::Continue(()),
        })
    }
}

/// Asks `check` every [`KILL_POLL`] until it ends the wait or `bound` has passed. Whether the
/// leader is confirmed gone: `Break(true)` is a yes, `Break(false)` (an error) and the end of the
/// bound are not.
fn wait_until(bound: Duration, mut check: impl FnMut() -> ControlFlow<bool>) -> bool {
    let until = Instant::now() + bound;
    while Instant::now() < until {
        if let ControlFlow::Break(gone) = check() {
            return gone;
        }
        thread::sleep(KILL_POLL);
    }
    matches!(check(), ControlFlow::Break(true))
}

impl Drop for Group {
    fn drop(&mut self) {
        self.kill();
    }
}

#[cfg(unix)]
fn into_group(command: &mut CommandWrap) {
    command.wrap(process_wrap::std::ProcessGroup::leader());
}

#[cfg(windows)]
fn into_group(command: &mut CommandWrap) {
    command.wrap(process_wrap::std::JobObject);
}

fn spawn(launch: &Launch) -> io::Result<Group> {
    let mut command = CommandWrap::with_new(&launch.program, |command| {
        command
            .args(&launch.args)
            .env_clear()
            .envs(launch.env.iter().map(|(name, value)| (name, value)))
            .current_dir(&launch.cwd)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
    });
    into_group(&mut command);
    command.spawn().map(Group)
}

/// What the supervisor watches besides the child.
struct Watch<'a> {
    limits: Limits,
    started: Instant,
    cancel: &'a CancelToken,
    written: Arc<AtomicU64>,
    folder: Option<&'a Path>,
    memory: TreeMemory,
    last_measure: Instant,
}

impl Watch<'_> {
    /// Why the run must stop now, if it must.
    fn verdict(&mut self, pid: u32) -> Option<Stop> {
        if self.cancel.is_cancelled() {
            return Some(Stop::Cancelled);
        }
        if self.started.elapsed() >= self.limits.timeout {
            return Some(Stop::Timeout);
        }
        if self.written.load(Ordering::Relaxed) > self.limits.max_output_bytes {
            return Some(Stop::Output);
        }
        if self.last_measure.elapsed() < MEASURE_INTERVAL {
            return None;
        }
        self.last_measure = Instant::now();
        self.measured(pid)
    }

    fn measured(&mut self, pid: u32) -> Option<Stop> {
        if self.memory.measure(pid) > self.limits.max_memory_bytes {
            return Some(Stop::Memory);
        }
        let on_disk = self.folder.map_or(0, |dir| folder_bytes(dir, 0));
        let total = self.written.load(Ordering::Relaxed).saturating_add(on_disk);
        (total > self.limits.max_output_bytes).then_some(Stop::Output)
    }
}

/// Bytes of the regular files below `dir`; links are not followed, and a deep tree is cut off.
fn folder_bytes(dir: &Path, depth: usize) -> u64 {
    // A folder nested deeper than the measure goes counts as over the limit: an indexer must not
    // hide data below the cut.
    if depth > MAX_SCAN_DEPTH {
        return u64::MAX;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return 0;
    };
    entries
        .filter_map(Result::ok)
        .filter_map(|entry| entry.metadata().ok().map(|meta| (entry.path(), meta)))
        .map(|(path, meta)| {
            if meta.is_dir() {
                folder_bytes(&path, depth.saturating_add(1))
            } else if meta.is_file() {
                meta.len()
            } else {
                0
            }
        })
        .fold(0, u64::saturating_add)
}

/// Starts the program and waits for it under the limits. The process group is killed when this
/// returns, whatever the result. Only if the leader was still running ten seconds after the kill
/// (`survived` of [`RunError::Stopped`] and [`RunError::Watch`]) can something of it be left, and
/// the caller must then keep the run folder for the sweep of the next start.
///
/// # Errors
/// [`RunError::Io`] if the process cannot be started, [`RunError::Watch`] if it cannot be watched,
/// [`RunError::Stopped`] if a limit or the cancel token ended it.
pub fn run_guarded(launch: &Launch, cancel: &CancelToken) -> Result<Finished, RunError> {
    let memory = TreeMemory::new().map_err(RunError::Io)?;
    let mut group = spawn(launch).map_err(RunError::Io)?;
    if let Some(folder) = &launch.note_dir {
        // Best effort: without the note the process is simply not found after a hard stop.
        drop(orphans::note_indexer(folder, group.0.id()));
    }
    let written = Arc::new(AtomicU64::new(0));
    let stdout = group
        .0
        .stdout()
        .take()
        .map(|s| Capture::start(s, Arc::clone(&written)));
    let stderr = group
        .0
        .stderr()
        .take()
        .map(|s| Capture::start(s, Arc::clone(&written)));
    let mut watch = Watch {
        limits: launch.limits,
        started: Instant::now(),
        cancel,
        written,
        folder: launch.watch_dir.as_deref(),
        memory,
        last_measure: Instant::now(),
    };
    let status = wait_for(&mut group, &mut watch)?;
    // Daemons that the build tools left behind die with the group.
    group.kill();
    let take = |capture: Option<Capture>| capture.map(|c| c.take()).unwrap_or_default();
    Ok(Finished {
        success: status.success(),
        code: status.code(),
        stdout: take(stdout),
        stderr: take(stderr),
    })
}

fn wait_for(group: &mut Group, watch: &mut Watch<'_>) -> Result<ExitStatus, RunError> {
    loop {
        match group.0.try_wait() {
            Ok(Some(status)) => return Ok(status),
            Ok(None) => {}
            Err(cause) => {
                let survived = !group.kill();
                return Err(RunError::Watch { cause, survived });
            }
        }
        if let Some(stop) = watch.verdict(group.0.id()) {
            let survived = !group.kill();
            return Err(RunError::Stopped { stop, survived });
        }
        thread::sleep(POLL_INTERVAL);
    }
}

#[cfg(test)]
mod tests {
    use std::ops::ControlFlow;
    use std::time::Duration;

    use super::wait_until;

    #[test]
    fn a_leader_that_is_gone_in_time_is_reported_gone() {
        let mut asked = 0;
        let gone = wait_until(Duration::from_secs(5), || {
            asked += 1;
            if asked >= 3 {
                ControlFlow::Break(true)
            } else {
                ControlFlow::Continue(())
            }
        });
        assert!(gone);
    }

    #[test]
    fn a_leader_that_outlives_the_bound_is_reported_alive() {
        let alive = || ControlFlow::Continue(());
        assert!(!wait_until(Duration::from_millis(50), alive));
    }

    #[test]
    fn an_error_is_not_a_confirmed_end() {
        let started = std::time::Instant::now();
        assert!(!wait_until(Duration::from_secs(5), || ControlFlow::Break(
            false
        )));
        // It does not wait for the bound either.
        assert!(started.elapsed() < Duration::from_secs(5));
    }
}
