//! Stand-in for the port whose adapter does not exist yet.
//!
//! The scan runs end to end with it: files, symbols and metrics are found and stored, but no
//! references. It is replaced in this file only, when the reference resolver (blueprint 9) arrives.

use codeskimr_application::error::CodedError;
use codeskimr_application::ids::IdSequence;
use codeskimr_application::ports::{PendingReference, ReferenceResolver};
use codeskimr_domain::{Reference, Symbol};

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
