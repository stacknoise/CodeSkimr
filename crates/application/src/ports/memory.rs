//! The memory a scan may use (ADR 0087).

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::thread::JoinHandle;
use std::time::Duration;

use crate::cancel::CancelToken;
use crate::consts::{
    LOG_MEMORY_LIMIT_PASSED, LOG_MEMORY_READING_MISSING, LOG_MEMORY_WATCH_NOT_STARTED,
    MEMORY_WATCH_THREAD,
};

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

    /// The limit in megabytes that counts for the scan.
    #[must_use]
    pub fn max_mb(&self) -> u64 {
        self.max_mb
    }

    /// Starts watching: when the limit is exceeded, the watch raises `cancel`, which every stage
    /// polls, and remembers why. The watch ends when the returned handle is dropped, without
    /// waiting for the end of the check interval. A system that cannot say how much memory the
    /// program holds, or a thread that cannot be started, leaves the scan unwatched; both are
    /// logged, because the limit does not protect then.
    #[must_use]
    pub fn watch(&self, cancel: CancelToken) -> MemoryWatch {
        let tripped = Arc::new(AtomicBool::new(false));
        if self.gauge.used_mb().is_none() {
            tracing::warn!(
                max_mb = self.max_mb,
                "memory reading unavailable, scan runs without memory limit"
            );
        }
        let (stop, wake) = mpsc::channel::<()>();
        let limit = self.clone();
        let flag = Arc::clone(&tripped);
        let spawned = std::thread::Builder::new()
            .name(MEMORY_WATCH_THREAD.to_owned())
            .spawn(move || {
                while let Err(RecvTimeoutError::Timeout) = wake.recv_timeout(limit.interval) {
                    if limit.exceeded() {
                        tracing::error!(event = LOG_MEMORY_LIMIT_PASSED, max_mb = limit.max_mb, used_mb = ?limit.gauge.used_mb());
                        flag.store(true, Ordering::Release);
                        cancel.cancel();
                        return;
                    }
                }
            });
        let thread = spawned
            .map_err(|cause| tracing::warn!(event = LOG_MEMORY_WATCH_NOT_STARTED, %cause))
            .ok();
        MemoryWatch {
            tripped,
            stop: Some(stop),
            thread,
        }
    }
}

impl std::fmt::Debug for MemoryLimit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple(std::any::type_name::<Self>())
            .field(&self.max_mb)
            .finish()
    }
}

/// A running memory watch of one scan.
pub struct MemoryWatch {
    tripped: Arc<AtomicBool>,
    stop: Option<Sender<()>>,
    thread: Option<JoinHandle<()>>,
}

impl MemoryWatch {
    /// Whether the watch stopped the scan because the limit was exceeded.
    #[must_use]
    pub fn tripped(&self) -> bool {
        self.tripped.load(Ordering::Acquire)
    }
}

impl Drop for MemoryWatch {
    fn drop(&mut self) {
        // Dropping the sender wakes the thread at once.
        self.stop.take();
        if let Some(thread) = self.thread.take() {
            // A panic of the watch thread has nothing left to report.
            let _ = thread.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    use super::{MemoryGauge, MemoryLimit};
    use crate::cancel::CancelToken;

    struct Fixed(Option<u64>);

    impl MemoryGauge for Fixed {
        fn used_mb(&self) -> Option<u64> {
            self.0
        }
    }

    #[test]
    fn dropping_the_watch_does_not_wait_for_the_interval() {
        let limit = MemoryLimit::new(Arc::new(Fixed(Some(1))), 100, Duration::from_secs(10));
        let watch = limit.watch(CancelToken::default());
        let start = Instant::now();
        drop(watch);
        assert!(start.elapsed() < Duration::from_secs(1));
    }

    #[test]
    fn a_missing_reading_never_trips_the_watch() {
        let limit = MemoryLimit::new(Arc::new(Fixed(None)), 1, Duration::from_millis(5));
        let watch = limit.watch(CancelToken::default());
        std::thread::sleep(Duration::from_millis(30));
        assert!(!watch.tripped());
    }

    #[test]
    fn a_reading_above_the_limit_trips_the_watch() {
        let cancel = CancelToken::default();
        let limit = MemoryLimit::new(Arc::new(Fixed(Some(10))), 1, Duration::from_millis(5));
        let watch = limit.watch(cancel.clone());
        std::thread::sleep(Duration::from_millis(100));
        assert!(watch.tripped());
        assert!(cancel.is_cancelled());
    }
}
