//! Tests of reading criterion results.

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use codeskimr_bench::BenchError;
use codeskimr_bench::estimates::{collect, collect_best};

type TestResult = Result<(), Box<dyn Error>>;

fn fresh_dir(name: &str) -> Result<PathBuf, Box<dyn Error>> {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn write_estimates(dir: &Path, bench: &str, baseline: &str, content: &str) -> TestResult {
    let target = dir.join(bench).join(baseline);
    fs::create_dir_all(&target)?;
    fs::write(target.join("estimates.json"), content)?;
    Ok(())
}

#[test]
fn reads_medians_of_the_named_baseline_only() -> TestResult {
    let dir = fresh_dir("estimates_ok")?;
    write_estimates(
        &dir,
        "domain/a",
        "ci-head",
        r#"{"median":{"point_estimate":12.5}}"#,
    )?;
    write_estimates(
        &dir,
        "domain/a",
        "ci-base",
        r#"{"median":{"point_estimate":99.0}}"#,
    )?;
    let medians = collect(&dir, "ci-head")?;
    assert_eq!(medians.get("domain/a"), Some(&12.5));
    assert_eq!(medians.len(), 1);
    Ok(())
}

#[test]
fn broken_file_is_an_error_not_a_missing_base() -> TestResult {
    let dir = fresh_dir("estimates_broken")?;
    write_estimates(&dir, "domain/a", "ci-base", "{ not json")?;
    assert!(matches!(
        collect(&dir, "ci-base"),
        Err(BenchError::Estimates { .. })
    ));
    Ok(())
}

#[test]
fn missing_directory_is_an_error() -> TestResult {
    let dir = fresh_dir("estimates_none")?.join("does-not-exist");
    assert!(matches!(
        collect(&dir, "ci-base"),
        Err(BenchError::Io { .. })
    ));
    Ok(())
}

#[test]
fn baseline_without_estimates_file_is_skipped() -> TestResult {
    let dir = fresh_dir("estimates_empty")?;
    fs::create_dir_all(dir.join("domain/a/ci-base"))?;
    assert!(collect(&dir, "ci-base")?.is_empty());
    Ok(())
}

#[test]
fn the_best_median_over_several_runs_counts() -> TestResult {
    let first = fresh_dir("estimates_best_first")?;
    let second = fresh_dir("estimates_best_second")?;
    let write = |dir: &Path, bench: &str, median: f64| {
        write_estimates(
            dir,
            bench,
            "ci-head",
            &format!(r#"{{"median":{{"point_estimate":{median}}}}}"#),
        )
    };
    write(&first, "domain/a", 30.0)?;
    write(&second, "domain/a", 20.0)?;
    write(&first, "domain/only_first", 5.0)?;
    let best = collect_best(&[first, second], "ci-head")?;
    assert_eq!(best.get("domain/a"), Some(&20.0));
    assert_eq!(best.get("domain/only_first"), Some(&5.0));
    Ok(())
}
