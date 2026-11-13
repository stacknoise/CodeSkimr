//! Text report of a comparison.

use crate::compare::{Row, Verdict};
use crate::consts::{NS_PER_US, text};

/// Renders one line per benchmark.
#[must_use]
pub fn render(rows: &[Row]) -> String {
    rows.iter().map(line).collect::<Vec<_>>().join("\n")
}

fn line(row: &Row) -> String {
    let base = row
        .base_ns
        .map_or_else(|| "-".to_owned(), |ns| format!("{:.2} us", ns / NS_PER_US));
    let marker = match &row.verdict {
        Verdict::Ok => text::OK.to_owned(),
        Verdict::Regression(change) => format!("{} (+{change:.1} %)", text::REGRESSION),
        Verdict::OverBudget(budget) => format!("{} (budget {budget} ms)", text::OVER_BUDGET),
        Verdict::NewBench => text::NEW_BENCH.to_owned(),
    };
    format!(
        "{:<40} new {:.2} us  base {base}  {marker}",
        row.id,
        row.new_ns / NS_PER_US
    )
}

/// True when any row fails the check.
#[must_use]
pub fn has_failure(rows: &[Row]) -> bool {
    rows.iter().any(|row| row.verdict.is_failure())
}
