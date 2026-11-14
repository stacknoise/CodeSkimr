//! A consent store for tests, compiled only with the feature `test-support`.

use std::sync::{Mutex, PoisonError};

use codeskimr_domain::ProjectId;

use super::{ConsentRecord, ConsentStore};
use crate::error::CodedError;

/// A consent store that lives in memory only. One record for every project.
#[derive(Debug, Default)]
pub struct InMemoryConsent(Mutex<ConsentRecord>);

impl ConsentStore for InMemoryConsent {
    fn consent(&self, _project: ProjectId) -> Result<ConsentRecord, CodedError> {
        Ok(*self.0.lock().unwrap_or_else(PoisonError::into_inner))
    }

    fn store_consent(&self, _project: ProjectId, record: ConsentRecord) -> Result<(), CodedError> {
        *self.0.lock().unwrap_or_else(PoisonError::into_inner) = record;
        Ok(())
    }
}
