//! The system clock behind the `Clock` port.

use std::time::{SystemTime, UNIX_EPOCH};

use codeskimr_application::ports::Clock;
use codeskimr_domain::Timestamp;

/// Seconds since the Unix epoch from the operating system.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Timestamp {
        let seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_secs());
        Timestamp(i64::try_from(seconds).unwrap_or(i64::MAX))
    }
}
