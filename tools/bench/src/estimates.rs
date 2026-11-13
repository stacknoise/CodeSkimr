//! Reading the medians that criterion stores per benchmark and baseline.

use std::collections::BTreeMap;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use serde::Deserialize;

use crate::consts::{ESTIMATES_FILE, ID_SEPARATOR, WINDOWS_SEPARATOR};
use crate::error::BenchError;

#[derive(Deserialize)]
struct Estimate {
    point_estimate: f64,
}

#[derive(Deserialize)]
struct Estimates {
    median: Estimate,
}

/// Median time in nanoseconds per benchmark id (`group/name`).
pub type Medians = BTreeMap<String, f64>;

fn io_error(path: &Path, source: std::io::Error) -> BenchError {
    BenchError::Io {
        path: path.to_path_buf(),
        source,
    }
}

/// Collects the medians of one named baseline below `criterion_dir`.
///
/// # Errors
/// Returns an error when a directory or an estimates file cannot be read or parsed.
pub fn collect(criterion_dir: &Path, baseline: &str) -> Result<Medians, BenchError> {
    let mut medians = Medians::new();
    walk(criterion_dir, criterion_dir, baseline, &mut medians)?;
    Ok(medians)
}

fn walk(root: &Path, dir: &Path, baseline: &str, medians: &mut Medians) -> Result<(), BenchError> {
    for entry in fs::read_dir(dir).map_err(|source| io_error(dir, source))? {
        let path = entry.map_err(|source| io_error(dir, source))?.path();
        if !path.is_dir() {
            continue;
        }
        if path.file_name().is_some_and(|name| name == baseline) {
            record(root, dir, &path, medians)?;
        } else {
            walk(root, &path, baseline, medians)?;
        }
    }
    Ok(())
}

fn record(
    root: &Path,
    bench_dir: &Path,
    baseline_dir: &Path,
    medians: &mut Medians,
) -> Result<(), BenchError> {
    let file = baseline_dir.join(ESTIMATES_FILE);
    let text = match fs::read_to_string(&file) {
        Ok(text) => text,
        Err(source) if source.kind() == ErrorKind::NotFound => return Ok(()),
        Err(source) => return Err(io_error(&file, source)),
    };
    let parsed: Estimates =
        serde_json::from_str(&text).map_err(|source| BenchError::Estimates {
            path: file.clone(),
            source,
        })?;
    let id = bench_dir
        .strip_prefix(root)
        .unwrap_or(bench_dir)
        .to_string_lossy()
        .replace(WINDOWS_SEPARATOR, ID_SEPARATOR);
    medians.insert(id, parsed.median.point_estimate);
    Ok(())
}
