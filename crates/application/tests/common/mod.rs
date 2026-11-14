//! Fakes shared by the tests of the application crate.

use std::sync::{Mutex, PoisonError};

use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{ConsentRecord, ConsentStore};
use codeskimr_domain::ProjectId;

/// A consent store in memory with one record for every project.
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
