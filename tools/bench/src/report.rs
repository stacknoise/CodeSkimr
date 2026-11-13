//! Text report of a comparison.

use crate::compare::{Row, Verdict};
use crate::consts::{MISSING, NS_PER_US, text};

/// Renders one line per benchmark.
#[must_use]
pub fn render(rows: &[Row]) -> String {
    rows.iter().map(line).collect::<Vec<_>>().join("\n")
}

fn micros(ns: Option<f64>) -> String {
    ns.map_or_else(
        || MISSING.to_owned(),
        |ns| format!("{:.2} us", ns / NS_PER_US),
    )
}

fn marker(verdict: &Verdict) -> String {
    match verdict {
        Verdict::Ok => text::OK.to_owned(),
        Verdict::Regression(change) => format!("{} (+{change:.1} %)", text::REGRESSION),
        Verdict::OverBudget(budget) => format!("{} (budget {budget} ms)", text::OVER_BUDGET),
        Verdict::NewBench => text::NEW_BENCH.to_owned(),
        Verdict::Removed => text::REMOVED.to_owned(),
    }
}

fn line(row: &Row) -> String {
    format!(
        "{:<40} new {}  base {}  {}",
        row.id,
        micros(row.new_ns),
        micros(row.base_ns),
        marker(&row.verdict)
    )
}

/// True when any row fails the check.
#[must_use]
pub fn has_failure(rows: &[Row]) -> bool {
    rows.iter().any(|row| row.verdict.is_failure())
}
