//! Stand-ins for the two ports whose adapters do not exist yet.
//!
//! The scan runs end to end with them: files and symbols are found and stored, but no references
//! and no metrics. They are replaced in this file only, when the reference resolver (blueprint 9)
//! and the metrics calculator (ADR 0015, issues #39 and #44) arrive.

use codeskimr_application::error::CodedError;
use codeskimr_application::ids::IdSequence;
use codeskimr_application::ports::{MetricsCalculator, PendingReference, ReferenceResolver};
use codeskimr_domain::{Metric, Reference, Symbol};

/// A resolver that finds no references.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoReferences;

impl ReferenceResolver for NoReferences {
    fn resolve(
        &self,
        _symbols: &[Symbol],
        _pending: &[PendingReference],
        _ids: &IdSequence,
    ) -> Result<Vec<Reference>, CodedError> {
        Ok(Vec::new())
    }
}

/// A calculator that measures nothing.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoMetrics;

impl MetricsCalculator for NoMetrics {
    fn measure(
        &self,
        _symbols: &[Symbol],
        _references: &[Reference],
    ) -> Result<Vec<Metric>, CodedError> {
        Ok(Vec::new())
    }
}
