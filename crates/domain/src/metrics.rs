//! Pure metric functions: lines of code, cyclomatic complexity and maintainability index.
//!
//! The functions take plain counts that a language pack extracts from its syntax tree, so the
//! domain stays free of parsers (blueprint 4.2). They are distinct from the entity in `metric`,
//! which only stores the results.

use crate::ModelError;
use crate::consts::{
    CC_BASE, MI_BASE, MI_COMPLEXITY_WEIGHT, MI_LOC_WEIGHT, MI_LOG_FLOOR, MI_MAX, MI_MIN,
    MI_VOLUME_WEIGHT,
};
use crate::file::Loc;
use crate::metric::MetricValue;

/// How a physical source line counts for the lines-of-code metric.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineKind {
    /// Only whitespace.
    Blank,
    /// Only a comment, no code.
    Comment,
    /// Contains code, with or without a trailing comment.
    Code,
}

/// Counts of operators and operands of one body, the input of the Halstead volume.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HalsteadCounts {
    /// Number of distinct operators.
    pub distinct_operators: u32,
    /// Number of distinct operands.
    pub distinct_operands: u32,
    /// Total number of operators.
    pub total_operators: u32,
    /// Total number of operands.
    pub total_operands: u32,
}

impl HalsteadCounts {
    /// Halstead volume `N * log2(n)`; zero when there is no vocabulary.
    #[must_use]
    pub fn volume(self) -> f64 {
        let length = f64::from(self.total_operators) + f64::from(self.total_operands);
        let vocabulary = f64::from(self.distinct_operators) + f64::from(self.distinct_operands);
        if vocabulary < MI_LOG_FLOOR {
            MI_MIN
        } else {
            length * vocabulary.log2()
        }
    }
}

/// Lines of code: all lines except blank and comment-only lines.
#[must_use]
pub fn lines_of_code<I: IntoIterator<Item = LineKind>>(lines: I) -> Loc {
    let code = lines
        .into_iter()
        .filter(|kind| *kind == LineKind::Code)
        .count();
    Loc(u32::try_from(code).unwrap_or(u32::MAX))
}

/// Cyclomatic complexity: one path plus one per decision point (branch, loop, case, catch,
/// short-circuit operator). Which syntax nodes count is decided by the language pack.
#[must_use]
pub fn cyclomatic_complexity(decision_points: u32) -> u32 {
    CC_BASE.saturating_add(decision_points)
}

/// Maintainability index on a 0 to 100 scale (higher is better), using the Visual Studio form
/// `max(0, (171 - 5.2 ln V - 0.23 CC - 16.2 ln LOC) * 100 / 171)`.
///
/// A body without code lines has nothing to maintain and scores the maximum.
///
/// # Errors
///
/// Returns [`ModelError::NotFinite`] only if the arithmetic overflows, which the `u32` inputs
/// rule out in practice.
pub fn maintainability_index(
    halstead: HalsteadCounts,
    complexity: u32,
    loc: Loc,
) -> Result<MetricValue, ModelError> {
    if loc.0 == 0 {
        return MetricValue::new(MI_MAX);
    }
    let volume = halstead.volume().max(MI_LOG_FLOOR);
    let lines = f64::from(loc.0).max(MI_LOG_FLOOR);
    let raw = MI_BASE
        - MI_VOLUME_WEIGHT * volume.ln()
        - MI_COMPLEXITY_WEIGHT * f64::from(complexity)
        - MI_LOC_WEIGHT * lines.ln();
    MetricValue::new((raw * MI_MAX / MI_BASE).clamp(MI_MIN, MI_MAX))
}
