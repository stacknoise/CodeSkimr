//! Reading the medians that criterion stores per benchmark and baseline.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::consts::ESTIMATES_FILE;
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

/// Collects the medians of one named baseline below `criterion_dir`.
///
/// # Errors
/// Returns an error when a directory or an estimates file cannot be read.
pub fn collect(criterion_dir: &Path, baseline: &str) -> Result<Medians, BenchError> {
    let mut medians = Medians::new();
    walk(criterion_dir, criterion_dir, baseline, &mut medians)?;
    Ok(medians)
}

fn walk(root: &Path, dir: &Path, baseline: &str, medians: &mut Medians) -> Result<(), BenchError> {
    let entries = fs::read_dir(dir).map_err(|source| BenchError::Io {
        path: dir.to_path_buf(),
        source,
    })?;
    for entry in entries.flatten() {
        let path = entry.path();
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
    let Ok(text) = fs::read_to_string(&file) else {
        return Ok(());
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
        .replace('\\', "/");
    medians.insert(id, parsed.median.point_estimate);
    Ok(())
}
