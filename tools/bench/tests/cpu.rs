//! Tests of the CPU time that a benchmark writes next to the criterion results.

use std::error::Error;
use std::fs;
use std::path::Path;
use std::time::Duration;

use codeskimr_bench::cpu::{collect, write};
use codeskimr_bench::report::render_cpu;

type TestResult = Result<(), Box<dyn Error>>;

fn fresh_dir(name: &str) -> Result<std::path::PathBuf, Box<dyn Error>> {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn millis(values: &[u64]) -> Vec<Duration> {
    values.iter().map(|ms| Duration::from_millis(*ms)).collect()
}

#[test]
fn the_median_of_the_samples_is_written_and_read_back() -> TestResult {
    let dir = fresh_dir("cpu-roundtrip")?;
    write(&dir, ("scan", "full"), &millis(&[900, 100, 500]))?;
    let ids = ["scan/full".to_owned(), "scan/other".to_owned()];
    let times = collect(&dir, ids.iter());
    assert_eq!(times.len(), 1);
    assert!((times["scan/full"] - 500.0).abs() < 1e-6);
    Ok(())
}

#[test]
fn the_report_lists_the_cpu_time_and_stays_silent_without() -> TestResult {
    let dir = fresh_dir("cpu-report")?;
    write(&dir, ("scan", "full"), &millis(&[2000]))?;
    let ids = ["scan/full".to_owned()];
    let text = render_cpu(&collect(&dir, ids.iter()));
    assert!(text.contains("scan/full"), "{text}");
    assert!(text.contains("2000 ms"), "{text}");
    assert_eq!(render_cpu(&collect(&dir, [].iter())), "");
    Ok(())
}
