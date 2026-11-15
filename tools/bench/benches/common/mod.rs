//! What the scan benchmarks share: the generated corpus, the paths and the release binary.

use std::ffi::OsStr;
use std::fmt::Write as _;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use criterion::Criterion;

use codeskimr_bench::random::Lcg;

/// Files of the corpus.
pub const FILES: usize = 400;
/// Functions per file; every function is about eight lines, with the type around it a file has
/// about 270 lines.
const FUNCTIONS: usize = 14;
/// Files per folder.
const FILES_PER_FOLDER: usize = 25;
pub const SEED: u64 = 7;
/// Where the corpus and the databases of the benchmarks live, relative to the repository root.
pub const SCRATCH: &str = "target/bench-scan";

pub fn root() -> PathBuf {
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

/// Removes a folder with everything below it; a folder that does not exist is no error.
pub fn wipe(path: &Path) -> io::Result<()> {
    fs::remove_dir_all(path).or_else(|error| match error.kind() {
        io::ErrorKind::NotFound => Ok(()),
        _ => Err(error),
    })
}

pub fn build_corpus(scratch: &Path) -> io::Result<PathBuf> {
    let corpus = scratch.join("corpus");
    wipe(&corpus)?;
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
pub fn build_binary() -> io::Result<PathBuf> {
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

/// Runs a benchmark whose iteration reports its own duration; a failed run counts as the longest
/// possible time, so that the budget check fails.
pub fn measure(
    c: &mut Criterion,
    (group, name): (&str, &str),
    (samples, seconds): (usize, u64),
    mut run: impl FnMut() -> io::Result<Duration>,
) {
    let mut group = c.benchmark_group(group);
    group.sample_size(samples);
    group.measurement_time(Duration::from_secs(seconds));
    group.bench_function(name, |b| {
        b.iter_custom(|iterations| {
            (0..iterations)
                .map(|_| run().unwrap_or(Duration::MAX))
                .fold(Duration::ZERO, Duration::saturating_add)
        });
    });
    group.finish();
}

/// Field of the JSON summary that holds the duration of the scan in seconds.
#[allow(dead_code, reason = "not every benchmark runs the scan binary")]
const SECONDS_FIELD: &str = "seconds";
/// Field of the JSON summary that holds the CPU time of the scan in seconds, summed over all threads.
#[allow(dead_code, reason = "not every benchmark runs the scan binary")]
const CPU_SECONDS_FIELD: &str = "cpu_seconds";

/// Runs `codeskimr scan` on a folder or a Git URL (with `branch`) against the database in `data`
/// and returns the time the scan itself reports and its CPU time.
#[allow(dead_code, reason = "not every benchmark runs the scan binary")]
pub fn scan_summary(
    binary: &Path,
    source: impl AsRef<OsStr>,
    data: &Path,
    branch: Option<&str>,
) -> io::Result<(Duration, Duration)> {
    let mut command = Command::new(binary);
    command
        .current_dir(root())
        .arg("scan")
        .arg(source)
        .arg("--json")
        .arg("--data-dir")
        .arg(data)
        .stdout(Stdio::piped());
    if let Some(branch) = branch {
        command.arg("--branch").arg(branch);
    }
    let output = command.output()?;
    if !output.status.success() {
        return Err(io::Error::other("the scan failed"));
    }
    let summary: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    let seconds = |field: &str| {
        summary
            .get(field)
            .and_then(serde_json::Value::as_f64)
            .map(Duration::from_secs_f64)
            .ok_or_else(|| io::Error::other("the summary has no duration"))
    };
    Ok((seconds(SECONDS_FIELD)?, seconds(CPU_SECONDS_FIELD)?))
}
