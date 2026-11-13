//! Comparing medians against a base measurement and against the budgets.

use std::collections::BTreeMap;

use crate::consts::{NS_PER_MS, PERCENT};
use crate::estimates::Medians;

/// Outcome for one benchmark.
#[derive(Debug, Clone, PartialEq)]
pub enum Verdict {
    /// Within tolerance and budget.
    Ok,
    /// Slower than the base by more than the tolerance (change in percent).
    Regression(f64),
    /// Above the absolute budget (budget in milliseconds).
    OverBudget(f64),
    /// No base measurement exists for this benchmark.
    NewBench,
}

impl Verdict {
    /// True when the verdict must fail the check.
    #[must_use]
    pub const fn is_failure(&self) -> bool {
        matches!(self, Self::Regression(_) | Self::OverBudget(_))
    }
}

/// One line of the report.
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    /// Benchmark id.
    pub id: String,
    /// Median of the new measurement in nanoseconds.
    pub new_ns: f64,
    /// Median of the base measurement in nanoseconds, when there is one.
    pub base_ns: Option<f64>,
    /// Outcome.
    pub verdict: Verdict,
}

/// Compares every new median with its base and with its budget in milliseconds.
#[must_use]
pub fn compare(
    new: &Medians,
    base: &Medians,
    budgets_ms: &BTreeMap<String, f64>,
    tolerance_percent: f64,
) -> Vec<Row> {
    new.iter()
        .map(|(id, &new_ns)| {
            let base_ns = base.get(id).copied();
            let verdict = judge(
                new_ns,
                base_ns,
                budgets_ms.get(id).copied(),
                tolerance_percent,
            );
            Row {
                id: id.clone(),
                new_ns,
                base_ns,
                verdict,
            }
        })
        .collect()
}

fn judge(new_ns: f64, base_ns: Option<f64>, budget_ms: Option<f64>, tolerance: f64) -> Verdict {
    if let Some(budget) = budget_ms.filter(|budget| new_ns / NS_PER_MS > *budget) {
        return Verdict::OverBudget(budget);
    }
    match base_ns {
        None => Verdict::NewBench,
        Some(base) if base > 0.0 => {
            let change = (new_ns / base - 1.0) * PERCENT;
            if change > tolerance {
                Verdict::Regression(change)
            } else {
                Verdict::Ok
            }
        }
        Some(_) => Verdict::Ok,
    }
}
