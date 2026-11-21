//! Fan-out of events to every subscriber.

use std::collections::BTreeMap;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use crate::stage::StageKind;

use super::{Event, ScanProgressed};

/// The receiving end of a subscription. Dropping it unsubscribes.
pub type Subscription = Receiver<Event>;

/// Delivers every published event to every live subscription. Clones share the subscribers.
///
/// Queues are unbounded: progress events are small and a subscriber that never reads is its own
/// problem, which a dropped receiver cleans up on the next publish.
///
/// The bus also remembers the last report of every stage of each running scan, so that a late
/// subscriber (a browser that reloaded its page) can learn how far a scan has come.
#[derive(Debug, Clone, Default)]
pub struct EventBus {
    subscribers: Arc<Mutex<Vec<Sender<Event>>>>,
    running: Arc<Mutex<Running>>,
}

/// The last report of each stage of each scan that has not ended, by project and stage.
type Running = BTreeMap<(u64, StageKind), ScanProgressed>;

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
        self.remember(event);
        self.lock()
            .retain(|subscriber| subscriber.send(event.clone()).is_ok());
    }

    /// The last report of every stage of every scan that is running now, in the order the
    /// stages run. A scan that ended is not in it.
    #[must_use]
    pub fn running_scans(&self) -> Vec<ScanProgressed> {
        self.running_lock().values().copied().collect()
    }

    /// Keeps the last report of each stage of a running scan, and forgets a scan that ended.
    fn remember(&self, event: &Event) {
        let mut running = self.running_lock();
        match event {
            Event::ScanProgressed(report) => {
                running.insert((report.project.get(), report.stage), *report);
            }
            Event::SnapshotCreated(created) => {
                running.retain(|(project, _), _| *project != created.snapshot.project.get());
            }
            Event::ScanStopped(stopped) => {
                running.retain(|(project, _), _| *project != stopped.project.get());
            }
            _ => {}
        }
    }

    fn running_lock(&self) -> MutexGuard<'_, Running> {
        self.running.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn lock(&self) -> MutexGuard<'_, Vec<Sender<Event>>> {
        // A panic in another thread cannot leave the list inconsistent, so poisoning is ignored.
        self.subscribers
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }
}

#[cfg(test)]
mod tests {
    use codeskimr_domain::ProjectId;

    use super::super::{ScanStopped, StopReason};
    use super::*;

    fn report(project: u64, stage: StageKind, done: u32, total: u32) -> Event {
        Event::ScanProgressed(ScanProgressed {
            project: ProjectId::new(project),
            stage,
            done,
            total,
            elapsed_ms: u64::from(done),
        })
    }

    #[test]
    fn a_late_subscriber_can_ask_for_the_last_report_of_each_stage_of_a_running_scan() {
        let bus = EventBus::new();
        bus.publish(&report(1, StageKind::Fetch, 1, 1));
        bus.publish(&report(1, StageKind::Analyze, 3, 10));
        bus.publish(&report(1, StageKind::Analyze, 4, 10));
        bus.publish(&report(2, StageKind::Discover, 0, 0));
        let seen: Vec<_> = bus
            .running_scans()
            .iter()
            .map(|r| (r.project.get(), r.stage, r.done))
            .collect();
        assert_eq!(
            seen,
            [
                (1, StageKind::Fetch, 1),
                (1, StageKind::Analyze, 4),
                (2, StageKind::Discover, 0)
            ]
        );
    }

    #[test]
    fn a_scan_that_ended_is_forgotten() {
        let bus = EventBus::new();
        bus.publish(&report(1, StageKind::Analyze, 4, 10));
        bus.publish(&report(2, StageKind::Analyze, 5, 10));
        bus.publish(&Event::ScanStopped(ScanStopped {
            project: ProjectId::new(1),
            reason: StopReason::Cancelled,
        }));
        let projects: Vec<_> = bus
            .running_scans()
            .iter()
            .map(|r| r.project.get())
            .collect();
        assert_eq!(projects, [2]);
    }
}
