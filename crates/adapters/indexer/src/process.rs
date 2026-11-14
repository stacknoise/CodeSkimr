//! One child process under limits: its own process group (a job object on Windows), a timeout, a
//! memory limit, an output limit and cancellation. Whatever ends the run, the whole group is
//! killed (ADR 0033, decision 11).

use std::ffi::OsString;
use std::io::{self, Read};
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
    CAPTURE_BYTES, DRAIN_GRACE, MAX_SCAN_DEPTH, MEASURE_INTERVAL, POLL_INTERVAL, READ_CHUNK,
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
    /// The process could not be started or watched.
    Io(io::Error),
    /// The run was stopped and its process group killed.
    Stopped(Stop),
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
    fn kill(&mut self) {
        // The group may be gone already; that is the goal, not an error.
        drop(self.0.start_kill());
        drop(self.0.wait());
    }
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

/// Starts the program and waits for it under the limits. The process group is dead when this
/// returns, whatever the result.
///
/// # Errors
/// [`RunError::Io`] if the process cannot be started or watched, [`RunError::Stopped`] if a limit
/// or the cancel token ended it.
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
        if let Some(status) = group.0.try_wait().map_err(RunError::Io)? {
            return Ok(status);
        }
        if let Some(stop) = watch.verdict(group.0.id()) {
            group.kill();
            return Err(RunError::Stopped(stop));
        }
        thread::sleep(POLL_INTERVAL);
    }
}
