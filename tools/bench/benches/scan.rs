//! Benchmark of the full scan (blueprint 16.1: 100k lines in under 1 s). It runs the real binary
//! on a generated corpus of about 100k lines of Rust. The corpus is generated, so it needs no
//! download, is the same on every machine and has no duplicate names that would multiply the
//! references.

use std::cell::RefCell;
use std::fs;
use std::io;
use std::path::Path;
use std::time::Duration;

use codeskimr_bench::cpu::write;
use criterion::Criterion;

mod common;

use common::{SCRATCH, build_binary, build_corpus, measure, root, scan_summary};

/// Where criterion writes its results, relative to the repository root.
const CRITERION_DIR: &str = "target/criterion";
const SAMPLES: usize = 10;
const MEASUREMENT_SECONDS: u64 = 30;
/// Runs one scan with a fresh database and returns the time the scan itself reports and its CPU
/// time. Starting the process and loading the language packs have a budget of their own (cold
/// start).
fn scan_once(binary: &Path, corpus: &Path, data: &Path) -> io::Result<(Duration, Duration)> {
    fs::remove_dir_all(data).or_else(|error| match error.kind() {
        io::ErrorKind::NotFound => Ok(()),
        _ => Err(error),
    })?;
    scan_summary(binary, corpus, data, None)
}

fn scans(c: &mut Criterion) -> io::Result<()> {
    let scratch = root().join(SCRATCH);
    let corpus = build_corpus(&scratch)?;
    let binary = build_binary()?;
    let data = scratch.join("data");
    let cpu = RefCell::new(Vec::new());
    measure(
        c,
        ("scan", "full_100k_loc"),
        (SAMPLES, MEASUREMENT_SECONDS),
        || {
            let (wall, cpu_time) = scan_once(&binary, &corpus, &data)?;
            cpu.borrow_mut().push(cpu_time);
            Ok(wall)
        },
    );
    write(
        &root().join(CRITERION_DIR),
        ("scan", "full_100k_loc"),
        &cpu.borrow(),
    )
}

fn main() -> io::Result<()> {
    let mut criterion = Criterion::default().configure_from_args();
    scans(&mut criterion)?;
    criterion.final_summary();
    Ok(())
}
