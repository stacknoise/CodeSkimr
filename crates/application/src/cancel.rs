//! Cooperative cancellation of a scan (blueprint 7: cancellable at every stage boundary).

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// A flag that one side raises and the other side polls. Clones share the same flag.
#[derive(Debug, Clone, Default)]
pub struct CancelToken(Arc<AtomicBool>);

impl CancelToken {
    /// Creates a token that is not cancelled.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Raises the flag for every clone of this token.
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }

    /// Whether `cancel` has been called on any clone.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}
