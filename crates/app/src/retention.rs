//! The retention of dismissed alerts (ADR 0068): they are deleted once they are older than
//! `annotations.dismissed_retention_days`, at the start of the server and once a day after that.

use std::sync::Arc;
use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::time::Duration;

use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{AlertPurge, AnnotationStore, Clock};
use codeskimr_codes::Coded;

use crate::clock::SystemClock;
use crate::consts::{PURGE_INTERVAL, PURGE_THREAD_NAME};

/// Deletes the dismissed alerts that are older than `retention_days`; returns how many it deleted.
///
/// # Errors
/// `STO` failures of the store.
pub(crate) fn purge_expired(
    store: &dyn AnnotationStore,
    clock: &dyn Clock,
    retention_days: u32,
) -> Result<u64, CodedError> {
    store.purge_dismissed(&AlertPurge {
        now: clock.now(),
        retention_days,
    })
}

/// The timer behind the daily purge. The thread ends when this value is dropped.
#[derive(Debug)]
pub(crate) struct DailyPurge {
    _stop: Sender<()>,
}

impl DailyPurge {
    /// Purges now and then after every `every`, until the value is dropped. A thread that cannot
    /// start is logged and leaves the alerts to the next start of the program.
    pub(crate) fn start(
        store: Arc<dyn AnnotationStore>,
        retention_days: u32,
        every: Duration,
    ) -> Self {
        let (stop, stopped) = mpsc::channel::<()>();
        let started = std::thread::Builder::new()
            .name(PURGE_THREAD_NAME.to_owned())
            .spawn(move || {
                loop {
                    log(purge_expired(&*store, &SystemClock, retention_days));
                    if !matches!(stopped.recv_timeout(every), Err(RecvTimeoutError::Timeout)) {
                        break;
                    }
                }
            });
        if started.is_err() {
            tracing::warn!("the thread that purges dismissed alerts could not start");
        }
        Self { _stop: stop }
    }

    /// The timer of the running server: the interval of one day.
    pub(crate) fn daily(store: Arc<dyn AnnotationStore>, retention_days: u32) -> Self {
        Self::start(store, retention_days, PURGE_INTERVAL)
    }
}

/// Writes the outcome of a purge to the log; a failure never stops the server.
fn log(outcome: Result<u64, CodedError>) {
    match outcome {
        Ok(removed) => tracing::debug!(removed, "purged dismissed alerts"),
        Err(error) => tracing::warn!(code = error.code().id(), "purging dismissed alerts failed"),
    }
}
