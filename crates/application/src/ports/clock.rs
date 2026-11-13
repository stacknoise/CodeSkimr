//! Access to the current time.

use codeskimr_domain::Timestamp;

/// The source of "now", replaceable in tests.
pub trait Clock: Send + Sync {
    /// The current time.
    fn now(&self) -> Timestamp;
}
