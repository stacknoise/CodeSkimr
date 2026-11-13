//! Budgets and tolerance from `config/default.toml` and the benchmark mapping.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use toml::Table;

use crate::consts::{BENCHMARKS_KEY, BUDGET_MAP_PATH, BUDGETS_TABLE, CONFIG_PATH, TOLERANCE_KEY};
use crate::error::BenchError;

/// The tolerance and the budget per benchmark id, in milliseconds.
#[derive(Debug, Clone, PartialEq)]
pub struct Limits {
    /// Allowed slowdown against the base, in percent.
    pub tolerance_percent: f64,
    /// Budget in milliseconds per benchmark id.
    pub budgets_ms: BTreeMap<String, f64>,
}

fn read_table(path: &Path) -> Result<Table, BenchError> {
    let text = fs::read_to_string(path).map_err(|source| BenchError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    text.parse::<Table>().map_err(|source| BenchError::Toml {
        path: path.to_path_buf(),
        source,
    })
}

/// Reads the limits below the repository `root`.
///
/// # Errors
/// Returns an error for unreadable files or a budget key that does not exist.
pub fn load(root: &Path) -> Result<Limits, BenchError> {
    let config = read_table(&root.join(CONFIG_PATH))?;
    let mapping = read_table(&root.join(BUDGET_MAP_PATH))?;
    let budgets = config
        .get(BUDGETS_TABLE)
        .and_then(toml::Value::as_table)
        .ok_or_else(|| BenchError::MissingConfig(BUDGETS_TABLE.to_owned()))?;
    let tolerance_percent = number(budgets, TOLERANCE_KEY)
        .ok_or_else(|| BenchError::MissingConfig(TOLERANCE_KEY.to_owned()))?;
    Ok(Limits {
        tolerance_percent,
        budgets_ms: map_budgets(&mapping, budgets)?,
    })
}

fn map_budgets(mapping: &Table, budgets: &Table) -> Result<BTreeMap<String, f64>, BenchError> {
    let mut out = BTreeMap::new();
    let Some(benchmarks) = mapping.get(BENCHMARKS_KEY).and_then(toml::Value::as_table) else {
        return Ok(out);
    };
    for (id, key) in benchmarks {
        let key = key.as_str().unwrap_or_default();
        if !budgets.contains_key(key) {
            return Err(BenchError::UnknownBudget(key.to_owned()));
        }
        let value =
            number(budgets, key).ok_or_else(|| BenchError::InvalidBudget(key.to_owned()))?;
        out.insert(id.clone(), value);
    }
    Ok(out)
}

fn number(table: &Table, key: &str) -> Option<f64> {
    match table.get(key)? {
        toml::Value::Integer(value) => u32::try_from(*value).ok().map(f64::from),
        toml::Value::Float(value) => Some(*value),
        _ => None,
    }
}
