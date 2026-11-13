//! The standard metrics strategy: lines of code, cyclomatic complexity and maintainability index
//! per symbol, rolled up to types, namespaces and modules (ADR 0015).

use codeskimr_domain::{Metric, Reference, Symbol, SymbolCounts, measure};

use crate::error::CodedError;
use crate::ports::MetricsCalculator;

/// Measures from the counts of the language packs with the pure functions of the domain. Coupling
/// metrics (Ce, Ca, instability) are not part of it yet.
#[derive(Debug, Clone, Copy, Default)]
pub struct StandardMetrics;

impl MetricsCalculator for StandardMetrics {
    fn measure(
        &self,
        symbols: &[Symbol],
        _references: &[Reference],
        counts: &[SymbolCounts],
    ) -> Result<Vec<Metric>, CodedError> {
        measure(symbols, counts).map_err(CodedError::caused_by)
    }
}
