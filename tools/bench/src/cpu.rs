//! CPU time per benchmark, the energy proxy of blueprint 16.2. criterion measures wall time only,
//! so a benchmark that runs a whole scan writes the CPU time it measured next to the criterion
//! results, and `bench-check` prints it beside the wall time. It is a report, not a budget.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::consts::{CPU_FILE, MEDIAN_DIVISOR, MS_PER_SECOND};

#[derive(Serialize, Deserialize)]
struct CpuTime {
    median_ms: f64,
}

/// CPU time in milliseconds per benchmark id (`group/name`).
pub type CpuTimes = BTreeMap<String, f64>;

fn file(criterion_dir: &Path, (group, name): (&str, &str)) -> PathBuf {
    criterion_dir.join(group).join(name).join(CPU_FILE)
}

/// Writes the median of `samples` as the CPU time of benchmark `group/name`.
///
/// # Errors
/// If the file cannot be written.
pub fn write(criterion_dir: &Path, id: (&str, &str), samples: &[Duration]) -> io::Result<()> {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let median = sorted
        .get(sorted.len() / MEDIAN_DIVISOR)
        .copied()
        .unwrap_or_default();
    let path = file(criterion_dir, id);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string(&CpuTime {
        median_ms: median.as_secs_f64() * MS_PER_SECOND,
    })?;
    fs::write(path, content)
}

/// Reads the CPU time of every id in `ids` that has one; a benchmark without a file has none.
#[must_use]
pub fn collect<'a>(criterion_dir: &Path, ids: impl Iterator<Item = &'a String>) -> CpuTimes {
    ids.filter_map(|id| {
        let (group, name) = id.split_once('/')?;
        let text = fs::read_to_string(file(criterion_dir, (group, name))).ok()?;
        let parsed: CpuTime = serde_json::from_str(&text).ok()?;
        Some((id.clone(), parsed.median_ms))
    })
    .collect()
}
