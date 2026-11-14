//! Starting a tool to learn its version.
//!
//! The only processes this adapter starts are the version queries of the tool check (ADR 0033,
//! decision 8): a fixed program with fixed arguments from the language pack, no shell, no project
//! folder as working directory, an environment that holds only the allow-listed variables, and a
//! time limit after which the process is killed.

use std::ffi::OsString;
use std::io::{self, Read};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

use codeskimr_application::ports::{ProbeOutcome, ToolProbe};

use crate::consts::{OUTPUT_JOINER, PROBE_POLL_MS};

/// What a probe is allowed to do.
#[derive(Debug, Clone)]
pub struct ProbeSettings {
    /// Time a tool has to finish.
    pub timeout: Duration,
    /// Most bytes of output that are kept, per stream.
    pub max_output: u64,
    /// Names of the environment variables the process gets.
    pub env_allowlist: Vec<String>,
    /// The working directory of the process: a folder of the application, never a project.
    pub work_dir: PathBuf,
}

/// Starts tools as child processes.
#[derive(Debug, Clone)]
pub struct ProcessProbe {
    settings: ProbeSettings,
}

impl ProcessProbe {
    /// Creates the probe.
    #[must_use]
    pub const fn new(settings: ProbeSettings) -> Self {
        Self { settings }
    }

    fn command(&self, program: &str, args: &[String]) -> Command {
        let allowed: Vec<(OsString, OsString)> = std::env::vars_os()
            .filter(|(name, _)| {
                self.settings
                    .env_allowlist
                    .iter()
                    .any(|allowed| name.to_string_lossy() == allowed.as_str())
            })
            .collect();
        let mut command = Command::new(program);
        command
            .args(args)
            .current_dir(&self.settings.work_dir)
            .env_clear()
            .envs(allowed)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        command
    }
}

/// Reads a stream on its own thread, keeps the first `max` bytes and drops the rest, so the tool
/// never blocks on a full pipe.
fn collect(mut stream: impl Read + Send + 'static, max: u64) -> Receiver<Vec<u8>> {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let mut kept = Vec::new();
        // A stream that fails to read leaves what was read so far.
        let _ = (&mut stream).take(max).read_to_end(&mut kept);
        let _ = io::copy(&mut stream, &mut io::sink());
        let _ = sender.send(kept);
    });
    receiver
}

/// Waits for the process; kills it when `deadline` passes. `false` means it was killed.
fn finished_before(child: &mut Child, deadline: Instant) -> bool {
    loop {
        match child.try_wait() {
            Ok(Some(_)) => return true,
            Ok(None) if Instant::now() < deadline => {
                thread::sleep(Duration::from_millis(PROBE_POLL_MS));
            }
            Ok(None) | Err(_) => {
                // The process may have exited meanwhile; either way it is gone afterwards.
                let _ = child.kill();
                let _ = child.wait();
                return false;
            }
        }
    }
}

fn text_of(receiver: &Receiver<Vec<u8>>, deadline: Instant) -> String {
    let wait = deadline.saturating_duration_since(Instant::now());
    receiver
        .recv_timeout(wait)
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
        .unwrap_or_default()
}

impl ToolProbe for ProcessProbe {
    fn probe(&self, command: &str, args: &[String]) -> ProbeOutcome {
        let deadline = Instant::now() + self.settings.timeout;
        let Ok(mut child) = self.command(command, args).spawn() else {
            return ProbeOutcome::NotFound;
        };
        let (Some(stdout), Some(stderr)) = (child.stdout.take(), child.stderr.take()) else {
            let _ = child.kill();
            let _ = child.wait();
            return ProbeOutcome::NotFound;
        };
        let max = self.settings.max_output;
        let (out, err) = (collect(stdout, max), collect(stderr, max));
        if !finished_before(&mut child, deadline) {
            return ProbeOutcome::TimedOut;
        }
        // Standard error counts: `java -version` prints there.
        let output = [text_of(&out, deadline), text_of(&err, deadline)].join(OUTPUT_JOINER);
        ProbeOutcome::Ran(output)
    }
}
