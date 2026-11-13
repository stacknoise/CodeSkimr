//! Tests of the limits from the configuration and the budget mapping.

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use codeskimr_bench::BenchError;
use codeskimr_bench::config::load;

type TestResult = Result<(), Box<dyn Error>>;

fn root(name: &str, config: &str, mapping: &str) -> Result<PathBuf, Box<dyn Error>> {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("config"))?;
    fs::create_dir_all(root.join("tools/bench"))?;
    fs::write(root.join("config/default.toml"), config)?;
    fs::write(root.join("tools/bench/budgets.toml"), mapping)?;
    Ok(root)
}

const CONFIG: &str =
    "[budgets]\nusages_query_ms = 20\nratio = 1.5\nbad = -3\nregression_tolerance_percent = 10\n";

#[test]
fn reads_tolerance_and_budgets() -> TestResult {
    let mapping = "[benchmarks]\n\"g/b\" = \"usages_query_ms\"\n\"g/c\" = \"ratio\"\n";
    let limits = load(&root("cfg_ok", CONFIG, mapping)?)?;
    assert!((limits.tolerance_percent - 10.0).abs() < f64::EPSILON);
    assert_eq!(limits.budgets_ms.get("g/b"), Some(&20.0));
    assert_eq!(limits.budgets_ms.get("g/c"), Some(&1.5));
    Ok(())
}

#[test]
fn unknown_budget_key_is_reported() -> TestResult {
    let mapping = "[benchmarks]\n\"g/b\" = \"nope\"\n";
    let result = load(&root("cfg_unknown", CONFIG, mapping)?);
    assert!(
        matches!(result, Err(BenchError::UnknownBudget(_))),
        "{result:?}"
    );
    Ok(())
}

#[test]
fn negative_budget_is_invalid_not_unknown() -> TestResult {
    let mapping = "[benchmarks]\n\"g/b\" = \"bad\"\n";
    let result = load(&root("cfg_negative", CONFIG, mapping)?);
    assert!(
        matches!(result, Err(BenchError::InvalidBudget(_))),
        "{result:?}"
    );
    Ok(())
}

#[test]
fn missing_tolerance_is_reported() -> TestResult {
    let config = "[budgets]\nusages_query_ms = 20\n";
    let result = load(&root("cfg_missing", config, "[benchmarks]\n")?);
    assert!(
        matches!(result, Err(BenchError::MissingConfig(_))),
        "{result:?}"
    );
    Ok(())
}
