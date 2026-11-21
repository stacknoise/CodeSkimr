//! The memory a scan may use (ADR 0087).

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::Duration;

use crate::cancel::CancelToken;

/// Reads how much memory the program holds, replaceable in tests.
pub trait MemoryGauge: Send + Sync {
    /// The memory the program holds now, in megabytes; `None` if the system does not say.
    fn used_mb(&self) -> Option<u64>;
}

/// The most memory a scan may hold, and the gauge that measures it.
#[derive(Clone)]
pub struct MemoryLimit {
    gauge: Arc<dyn MemoryGauge>,
    max_mb: u64,
    interval: Duration,
}

impl MemoryLimit {
    /// A limit of `max_mb` megabytes, checked every `interval` while a scan runs.
    #[must_use]
    pub fn new(gauge: Arc<dyn MemoryGauge>, max_mb: u64, interval: Duration) -> Self {
        Self {
            gauge,
            max_mb,
            interval,
        }
    }

    /// Whether the program holds more than the limit.
    #[must_use]
    pub fn exceeded(&self) -> bool {
        self.gauge.used_mb().is_some_and(|used| used > self.max_mb)
    }

    /// Starts watching: when the limit is exceeded, the watch raises `cancel`, which every stage
    /// polls, and remembers why. The watch ends when the returned handle is dropped.
    #[must_use]
    pub fn watch(&self, cancel: CancelToken) -> MemoryWatch {
        let state = Arc::new(WatchState::default());
        let limit = self.clone();
        let shared = Arc::clone(&state);
        let thread = std::thread::spawn(move || {
            while !shared.stop.load(Ordering::Acquire) {
                if limit.exceeded() {
                    shared.tripped.store(true, Ordering::Release);
                    cancel.cancel();
                    return;
                }
                std::thread::sleep(limit.interval);
            }
        });
        MemoryWatch {
            state,
            thread: Some(thread),
        }
    }
}

impl std::fmt::Debug for MemoryLimit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemoryLimit")
            .field("max_mb", &self.max_mb)
            .finish_non_exhaustive()
    }
}

#[derive(Default)]
struct WatchState {
    stop: AtomicBool,
    tripped: AtomicBool,
}

/// A running memory watch of one scan.
pub struct MemoryWatch {
    state: Arc<WatchState>,
    thread: Option<JoinHandle<()>>,
}

impl MemoryWatch {
    /// Whether the watch stopped the scan because the limit was exceeded.
    #[must_use]
    pub fn tripped(&self) -> bool {
        self.state.tripped.load(Ordering::Acquire)
    }
}

impl Drop for MemoryWatch {
    fn drop(&mut self) {
        self.state.stop.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            // A panic of the watch thread has nothing left to report.
            let _ = thread.join();
        }
    }
}
