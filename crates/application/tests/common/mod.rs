//! Fakes shared by the tests of the application crate.

use std::sync::{Mutex, PoisonError};

use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{ConsentGrant, ConsentRecord, ConsentStore};
use codeskimr_domain::ProjectId;

/// A consent store in memory with one record for every project.
#[derive(Debug, Default)]
pub struct InMemoryConsent(Mutex<ConsentRecord>);

impl ConsentStore for InMemoryConsent {
    fn consent(&self, _project: ProjectId) -> Result<ConsentRecord, CodedError> {
        Ok(*self.0.lock().unwrap_or_else(PoisonError::into_inner))
    }

    fn store_consent(&self, grant: &ConsentGrant) -> Result<(), CodedError> {
        self.set(grant.record());
        Ok(())
    }
}

impl InMemoryConsent {
    /// Sets the record directly, for a test that needs a state `confirm` never produces.
    pub fn set(&self, record: ConsentRecord) {
        *self.0.lock().unwrap_or_else(PoisonError::into_inner) = record;
    }
}
