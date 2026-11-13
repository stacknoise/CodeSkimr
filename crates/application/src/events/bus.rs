//! Fan-out of events to every subscriber.

use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex, PoisonError};

use super::Event;

/// The receiving end of a subscription. Dropping it unsubscribes.
pub type Subscription = Receiver<Event>;

/// Delivers every published event to every live subscription. Clones share the subscribers.
///
/// Queues are unbounded: progress events are small and a subscriber that never reads is its own
/// problem, which a dropped receiver cleans up on the next publish.
#[derive(Debug, Clone, Default)]
pub struct EventBus {
    subscribers: Arc<Mutex<Vec<Sender<Event>>>>,
}

impl EventBus {
    /// Creates a bus without subscribers.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Starts receiving all events published from now on.
    #[must_use]
    pub fn subscribe(&self) -> Subscription {
        let (sender, receiver) = channel();
        self.lock().push(sender);
        receiver
    }

    /// Sends `event` to every subscriber and forgets those that have gone away.
    pub fn publish(&self, event: &Event) {
        self.lock()
            .retain(|subscriber| subscriber.send(event.clone()).is_ok());
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Vec<Sender<Event>>> {
        // A panic in another thread cannot leave the list inconsistent, so poisoning is ignored.
        self.subscribers
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }
}
