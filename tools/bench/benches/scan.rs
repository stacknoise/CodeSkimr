//! Benchmark of the full scan (blueprint 16.1: 100k lines in under 1 s). It runs the real binary
//! on a generated corpus of about 100k lines of Rust. The corpus is generated, so it needs no
//! download, is the same on every machine and has no duplicate names that would multiply the
//! references.

use std::fmt::Write as _;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use codeskimr_bench::random::Lcg;
use criterion::Criterion;

/// Files of the corpus.
const FILES: usize = 400;
/// Functions per file; every function is about eight lines, with the type around it a file has
/// about 125 lines.
const FUNCTIONS: usize = 14;
/// Files per folder.
const FILES_PER_FOLDER: usize = 25;
const SEED: u64 = 7;
const SAMPLES: usize = 10;
const MEASUREMENT_SECONDS: u64 = 30;
const SCRATCH: &str = "target/bench-scan";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The source of file `index`: a type with methods that call the helper of the same number in
/// their own file and in another file.
fn file_source(index: usize, random: &mut Lcg) -> String {
    let mut text = String::new();
    let _ = writeln!(text, "//! Generated module {index}.\n");
    let _ = writeln!(
        text,
        "/// The type of module {index}.\npub struct Thing{index} {{\n    value{index}: u64,\n}}\n"
    );
    let _ = writeln!(text, "impl Thing{index} {{");
    let _ = writeln!(
        text,
        "    /// Creates the value.\n    pub fn new() -> Self {{\n        Self {{ value{index}: {index} }}\n    }}\n"
    );
    for function in 0..FUNCTIONS {
        let other = random.below(FILES);
        let _ = writeln!(
            text,
            "    /// Method {function}.\n    pub fn method_{index}_{function}(&self, x: u64) -> u64 {{\n        let y = helper_{index}_{function}(x);\n        if y > {function} {{\n            y + self.value{index} + helper_{other}_{function}(y)\n        }} else {{\n            y\n        }}\n    }}\n"
        );
    }
    let _ = writeln!(text, "}}\n");
    for function in 0..FUNCTIONS {
        let _ = writeln!(
            text,
            "fn helper_{index}_{function}(x: u64) -> u64 {{\n    if x > 3 {{\n        x * 2\n    }} else {{\n        x + 1\n    }}\n}}\n"
        );
    }
    text
}

fn build_corpus(scratch: &Path) -> io::Result<PathBuf> {
    let corpus = scratch.join("corpus");
    fs::remove_dir_all(&corpus).or_else(|error| match error.kind() {
        io::ErrorKind::NotFound => Ok(()),
        _ => Err(error),
    })?;
    let mut random = Lcg::new(SEED);
    for index in 0..FILES {
        let folder = corpus.join(format!("part{}", index / FILES_PER_FOLDER));
        fs::create_dir_all(&folder)?;
        fs::write(
            folder.join(format!("thing{index}.rs")),
            file_source(index, &mut random),
        )?;
    }
    Ok(corpus)
}

/// Builds the release binary the benchmark runs; it is the product as users get it.
fn build_binary() -> io::Result<PathBuf> {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let status = Command::new(cargo)
        .args(["build", "--release", "--locked", "-p", "codeskimr"])
        .current_dir(root())
        .status()?;
    if !status.success() {
        return Err(io::Error::other("building the codeskimr binary failed"));
    }
    Ok(root().join("target/release/codeskimr"))
}

fn scan_once(binary: &Path, corpus: &Path, data: &Path) -> io::Result<()> {
    fs::remove_dir_all(data).or_else(|error| match error.kind() {
        io::ErrorKind::NotFound => Ok(()),
        _ => Err(error),
    })?;
    let status = Command::new(binary)
        .current_dir(root())
        .arg("scan")
        .arg(corpus)
        .arg("--json")
        .arg("--data-dir")
        .arg(data)
        .stdout(Stdio::null())
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other("the scan failed"))
    }
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
        b.iter(|| scan_once(&binary, &corpus, &data).is_ok());
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
