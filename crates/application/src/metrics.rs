//! The standard metrics strategy: lines of code, cyclomatic complexity and maintainability index
//! per symbol, rolled up to types, namespaces and modules, and the coupling of namespaces and
//! modules (ADR 0015, ADR 0037).

use codeskimr_domain::{Metric, Reference, Symbol, SymbolCounts, coupling, measure};

use crate::error::CodedError;
use crate::ports::MetricsCalculator;

/// Measures from the counts of the language packs with the pure functions of the domain, and
/// computes Ce, Ca and instability from the references of the whole snapshot.
#[derive(Debug, Clone, Copy, Default)]
pub struct StandardMetrics;

impl MetricsCalculator for StandardMetrics {
    fn measure(
        &self,
        symbols: &[Symbol],
        references: &[Reference],
        counts: &[SymbolCounts],
    ) -> Result<Vec<Metric>, CodedError> {
        let mut metrics = measure(symbols, counts).map_err(CodedError::caused_by)?;
        metrics.extend(coupling(symbols, references).map_err(CodedError::caused_by)?);
        Ok(metrics)
    }
}
