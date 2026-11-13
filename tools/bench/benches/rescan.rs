//! Benchmark of the incremental rescan (blueprint 16.1: under 300 ms after a file is saved). It
//! runs the real binary in watch mode on the generated corpus, appends a line to one file and
//! takes the time that watch mode reports for the rescan. Rescans before the measurement fill
//! the retention of snapshots, so the measured ones also prune the oldest snapshot, as every
//! rescan of a long session does.

use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::process::{Child, ChildStdout, Command, Stdio};
use std::time::Duration;

use criterion::Criterion;

mod common;

use common::{SCRATCH, build_binary, build_corpus, measure, root};

const SAMPLES: usize = 30;
const MEASUREMENT_SECONDS: u64 = 20;
/// Rescans before the measurement; more than the snapshots that are kept.
const WARM_UP_RESCANS: usize = 7;
/// The file that gets a line appended; it exists in the generated corpus.
const CHANGED_FILE: &str = "part0/thing0.rs";
/// Line that appears when the first scan is done and watching begins.
const WATCHING_MARKER: &str = "Watching";
/// Text in front of the duration in the report of a rescan (English texts, see `LANGUAGE_VARIABLE`).
const RESCAN_MARKER: &str = "again in ";
const LANGUAGE_VARIABLE: &str = "CODESKIMR_LANG";
const LANGUAGE: &str = "en";
const APPENDED_LINE: &[u8] = b"// changed\n";
const MS_PER_SECOND: f64 = 1_000.0;

/// The watching process and what it prints.
struct Watch {
    child: Child,
    lines: io::Lines<BufReader<ChildStdout>>,
    changed: std::path::PathBuf,
}

impl Watch {
    fn start() -> io::Result<Self> {
        let scratch = root().join(SCRATCH);
        let corpus = build_corpus(&scratch)?;
        let binary = build_binary()?;
        let data = scratch.join("data-rescan");
        match fs::remove_dir_all(&data) {
            Err(error) if error.kind() != io::ErrorKind::NotFound => return Err(error),
            _ => {}
        }
        let mut child = Command::new(binary)
            .current_dir(root())
            .env(LANGUAGE_VARIABLE, LANGUAGE)
            .arg("scan")
            .arg(&corpus)
            .arg("--watch")
            .arg("--data-dir")
            .arg(&data)
            .stdout(Stdio::piped())
            .spawn()?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| io::Error::other("no output"))?;
        let mut watch = Self {
            child,
            lines: BufReader::new(stdout).lines(),
            changed: corpus.join(CHANGED_FILE),
        };
        watch.wait_for(WATCHING_MARKER)?;
        Ok(watch)
    }

    /// Reads lines until one contains `marker` and returns it.
    fn wait_for(&mut self, marker: &str) -> io::Result<String> {
        for line in self.lines.by_ref() {
            let line = line?;
            if line.contains(marker) {
                return Ok(line);
            }
        }
        Err(io::Error::other("the process ended"))
    }

    /// Changes the file and returns the time of the rescan that watch mode reports.
    fn rescan(&mut self) -> io::Result<Duration> {
        OpenOptions::new()
            .append(true)
            .open(&self.changed)?
            .write_all(APPENDED_LINE)?;
        let line = self.wait_for(RESCAN_MARKER)?;
        let millis: f64 = line
            .split(RESCAN_MARKER)
            .nth(1)
            .map(|rest| {
                rest.chars()
                    .take_while(char::is_ascii_digit)
                    .collect::<String>()
            })
            .and_then(|digits| digits.parse().ok())
            .ok_or_else(|| io::Error::other("no duration in the report"))?;
        Ok(Duration::from_secs_f64(millis / MS_PER_SECOND))
    }
}

impl Drop for Watch {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn rescans(c: &mut Criterion) -> io::Result<()> {
    let mut watch = Watch::start()?;
    for _ in 0..WARM_UP_RESCANS {
        watch.rescan()?;
    }
    measure(
        c,
        ("rescan", "one_file_100k_loc"),
        (SAMPLES, MEASUREMENT_SECONDS),
        || watch.rescan(),
    );
    Ok(())
}

fn main() -> io::Result<()> {
    let mut criterion = Criterion::default().configure_from_args();
    rescans(&mut criterion)?;
    criterion.final_summary();
    Ok(())
}
