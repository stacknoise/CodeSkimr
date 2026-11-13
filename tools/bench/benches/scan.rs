//! Benchmark of the full scan (blueprint 16.1: 100k lines in under 1 s). It runs the real binary
//! on a generated corpus of about 100k lines of Rust. The corpus is generated, so it needs no
//! download, is the same on every machine and has no duplicate names that would multiply the
//! references.

use std::fs;
use std::io;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

use criterion::Criterion;

mod common;

use common::{SCRATCH, build_binary, build_corpus, root};

const SAMPLES: usize = 10;
const MEASUREMENT_SECONDS: u64 = 30;
/// Field of the JSON summary that holds the duration of the scan in seconds.
const SECONDS_FIELD: &str = "seconds";

/// Runs one scan with a fresh database and returns the time the scan itself reports. Starting the
/// process and loading the language packs have a budget of their own (cold start).
fn scan_once(binary: &Path, corpus: &Path, data: &Path) -> io::Result<Duration> {
    fs::remove_dir_all(data).or_else(|error| match error.kind() {
        io::ErrorKind::NotFound => Ok(()),
        _ => Err(error),
    })?;
    let output = Command::new(binary)
        .current_dir(root())
        .arg("scan")
        .arg(corpus)
        .arg("--json")
        .arg("--data-dir")
        .arg(data)
        .stdout(Stdio::piped())
        .output()?;
    if !output.status.success() {
        return Err(io::Error::other("the scan failed"));
    }
    let summary: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    summary
        .get(SECONDS_FIELD)
        .and_then(serde_json::Value::as_f64)
        .map(Duration::from_secs_f64)
        .ok_or_else(|| io::Error::other("the summary has no duration"))
}

fn scans(c: &mut Criterion) -> io::Result<()> {
    let scratch = root().join(SCRATCH);
    let corpus = build_corpus(&scratch)?;
    let binary = build_binary()?;
    let data = scratch.join("data");
    let mut group = c.benchmark_group("scan");
    group.sample_size(SAMPLES);
    group.measurement_time(Duration::from_secs(MEASUREMENT_SECONDS));
    group.bench_function("full_100k_loc", |b| {
        b.iter_custom(|iterations| {
            (0..iterations)
                .map(|_| scan_once(&binary, &corpus, &data).unwrap_or(Duration::MAX))
                .fold(Duration::ZERO, Duration::saturating_add)
        });
    });
    group.finish();
    Ok(())
}

fn main() -> io::Result<()> {
    let mut criterion = Criterion::default().configure_from_args();
    scans(&mut criterion)?;
    criterion.final_summary();
    Ok(())
}
