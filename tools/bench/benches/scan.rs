//! Benchmark of the full scan (blueprint 16.1: 100k lines in under 1 s). It runs the real binary
//! on a generated corpus of about 100k lines of Rust. The corpus is generated, so it needs no
//! download, is the same on every machine and has no duplicate names that would multiply the
//! references.

use std::cell::RefCell;
use std::io;
use std::path::Path;
use std::time::Duration;

use codeskimr_bench::cpu::write;
use criterion::Criterion;

mod common;

use common::{
    SCRATCH, build_binary, build_corpus, build_csharp_corpus, measure, root, scan_summary, wipe,
};

/// Where criterion writes its results, relative to the repository root.
const CRITERION_DIR: &str = "target/criterion";
const SAMPLES: usize = 10;
const MEASUREMENT_SECONDS: u64 = 30;

/// Runs one scan with a fresh database and returns the time the scan itself reports and its CPU
/// time. Starting the process has a budget of its own (cold start); loading the language
/// packs is left out of the reported time (ADR 0059) and not measured here yet.
fn scan_once(binary: &Path, corpus: &Path, data: &Path) -> io::Result<(Duration, Duration)> {
    wipe(data)?;
    scan_summary(binary, corpus, data, None)
}

fn scans(c: &mut Criterion) -> io::Result<()> {
    let scratch = root().join(SCRATCH);
    let binary = build_binary()?;
    scan_corpus(c, &binary, &build_corpus(&scratch)?, "full_100k_loc")?;
    // Dense references between many symbols of the same names, as in real C# projects. Only its
    // change against the base commit is checked until it keeps the budget of the Rust corpus.
    scan_corpus(
        c,
        &binary,
        &build_csharp_corpus(&scratch)?,
        "full_100k_loc_csharp",
    )
}

fn scan_corpus(c: &mut Criterion, binary: &Path, corpus: &Path, name: &str) -> io::Result<()> {
    let scratch = root().join(SCRATCH);
    let data = scratch.join("data");
    let cpu = RefCell::new(Vec::new());
    measure(c, ("scan", name), (SAMPLES, MEASUREMENT_SECONDS), || {
        let (wall, cpu_time) = scan_once(binary, corpus, &data)?;
        cpu.borrow_mut().push(cpu_time);
        Ok(wall)
    });
    write(&root().join(CRITERION_DIR), ("scan", name), &cpu.borrow())
}

fn main() -> io::Result<()> {
    let mut criterion = Criterion::default().configure_from_args();
    scans(&mut criterion)?;
    criterion.final_summary();
    Ok(())
}
