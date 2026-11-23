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

/// The most memory a scan may add to what the program held when it started, with an optional
/// ceiling for the whole program, and the gauge that measures it.
///
/// The growth is what counts for `max_mb`: the program may already hold the index of another
/// project when a scan starts, and that is not the scan's own memory. The ceiling protects the
/// machine and counts everything.
#[derive(Clone)]
pub struct MemoryLimit {
    gauge: Arc<dyn MemoryGauge>,
    max_mb: u64,
    ceiling_mb: Option<u64>,
    interval: Duration,
}

impl MemoryLimit {
    /// A limit of `max_mb` megabytes of growth, checked every `interval` while a scan runs.
    #[must_use]
    pub fn new(gauge: Arc<dyn MemoryGauge>, max_mb: u64, interval: Duration) -> Self {
        Self {
            gauge,
            max_mb,
            ceiling_mb: None,
            interval,
        }
    }

    /// Adds a ceiling of `ceiling_mb` megabytes for the whole program, whatever it held before.
    #[must_use]
    pub fn with_ceiling(mut self, ceiling_mb: u64) -> Self {
        self.ceiling_mb = Some(ceiling_mb);
        self
    }

    /// The most the program may hold in total, given what it held when the scan started.
    fn allowed_mb(&self, baseline_mb: u64) -> u64 {
        let by_growth = baseline_mb.saturating_add(self.max_mb);
        self.ceiling_mb
            .map_or(by_growth, |ceiling| by_growth.min(ceiling))
    }

    /// Whether the program holds more than the scan may, `baseline_mb` being what it held when
    /// the scan started.
    #[must_use]
    pub fn exceeded(&self, baseline_mb: u64) -> bool {
        let allowed = self.allowed_mb(baseline_mb);
        self.gauge.used_mb().is_some_and(|used| used > allowed)
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
        let first = self.gauge.used_mb();
        if first.is_none() {
            tracing::warn!(event = LOG_MEMORY_READING_MISSING, max_mb = self.max_mb);
        }
        let baseline = first.unwrap_or_default();
        let (stop, wake) = mpsc::channel::<()>();
        let limit = self.clone();
        let flag = Arc::clone(&tripped);
        let spawned = std::thread::Builder::new()
            .name(MEMORY_WATCH_THREAD.to_owned())
            .spawn(move || {
                while let Err(RecvTimeoutError::Timeout) = wake.recv_timeout(limit.interval) {
                    if limit.exceeded(baseline) {
                        tracing::error!(
                            event = LOG_MEMORY_LIMIT_PASSED,
                            max_mb = limit.max_mb,
                            baseline_mb = baseline,
                            allowed_mb = limit.allowed_mb(baseline),
                            ceiling_mb = ?limit.ceiling_mb,
                            used_mb = ?limit.gauge.used_mb()
                        );
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
    fn memory_held_before_the_scan_does_not_count_against_the_growth() {
        let cancel = CancelToken::default();
        let limit = MemoryLimit::new(Arc::new(Fixed(Some(500))), 100, Duration::from_millis(5));
        let watch = limit.watch(cancel.clone());
        std::thread::sleep(Duration::from_millis(50));
        assert!(!watch.tripped());
        assert!(limit.exceeded(300));
        assert!(!limit.exceeded(450));
    }

    /// Reports the first value once and the second from then on.
    struct Rising(std::sync::atomic::AtomicU32);

    impl MemoryGauge for Rising {
        fn used_mb(&self) -> Option<u64> {
            let calls = self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Some(if calls == 0 { 300 } else { 500 })
        }
    }

    #[test]
    fn growth_alone_trips_the_watch_without_a_ceiling() {
        let cancel = CancelToken::default();
        let limit = MemoryLimit::new(
            Arc::new(Rising(std::sync::atomic::AtomicU32::new(0))),
            100,
            Duration::from_millis(5),
        );
        let watch = limit.watch(cancel.clone());
        std::thread::sleep(Duration::from_millis(100));
        assert!(watch.tripped());
        assert!(cancel.is_cancelled());
    }

    #[test]
    fn growth_within_the_limit_does_not_trip_the_watch_below_the_ceiling() {
        let limit = MemoryLimit::new(
            Arc::new(Rising(std::sync::atomic::AtomicU32::new(0))),
            1000,
            Duration::from_millis(5),
        )
        .with_ceiling(2000);
        let watch = limit.watch(CancelToken::default());
        std::thread::sleep(Duration::from_millis(100));
        assert!(!watch.tripped());
    }

    #[test]
    fn the_ceiling_counts_the_whole_program() {
        let limit = MemoryLimit::new(Arc::new(Fixed(Some(500))), 1000, Duration::from_secs(1))
            .with_ceiling(400);
        assert!(limit.exceeded(450));
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
        let limit = MemoryLimit::new(Arc::new(Fixed(Some(10))), 1, Duration::from_millis(5))
            .with_ceiling(1);
        let watch = limit.watch(cancel.clone());
        std::thread::sleep(Duration::from_millis(100));
        assert!(watch.tripped());
        assert!(cancel.is_cancelled());
    }
}
