//! The pause between two confirmation windows (ADR 0071, decision 4a).

use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

/// When the last window was opened. One pause for the whole server, for every project and every
/// client: the answer to a click does not depend on which tab or program sent it.
#[derive(Debug)]
pub(crate) struct Pause {
    length: Duration,
    last: Mutex<Option<Instant>>,
}

impl Pause {
    pub(crate) const fn new(length: Duration) -> Self {
        Self {
            length,
            last: Mutex::new(None),
        }
    }

    /// Starts the pause at `now`, so that a second request that comes right after is refused even
    /// before the first one has finished. Gives the earlier start back for [`Pause::undo`], or,
    /// inside a pause, the time that is left.
    pub(crate) fn begin(&self, now: Instant) -> Result<Option<Instant>, Duration> {
        let mut last = self.last.lock().unwrap_or_else(PoisonError::into_inner);
        let left = Self::left(*last, self.length, now);
        if left.is_zero() {
            Ok(last.replace(now))
        } else {
            Err(left)
        }
    }

    /// Puts the start back that `begin` returned: nothing was opened after all.
    pub(crate) fn undo(&self, before: Option<Instant>) {
        *self.last.lock().unwrap_or_else(PoisonError::into_inner) = before;
    }

    /// How long the pause lasts from `now` on; zero outside of it.
    pub(crate) fn remaining(&self, now: Instant) -> Duration {
        let last = self.last.lock().unwrap_or_else(PoisonError::into_inner);
        Self::left(*last, self.length, now)
    }

    fn left(last: Option<Instant>, length: Duration, now: Instant) -> Duration {
        last.map_or(Duration::ZERO, |start| {
            length.saturating_sub(now.saturating_duration_since(start))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LENGTH: Duration = Duration::from_secs(30);

    #[test]
    fn a_second_request_inside_the_pause_is_refused_with_the_time_left() {
        let pause = Pause::new(LENGTH);
        let start = Instant::now();
        assert_eq!(pause.begin(start), Ok(None));
        let later = start + Duration::from_secs(10);
        assert_eq!(pause.begin(later), Err(Duration::from_secs(20)));
        assert_eq!(pause.remaining(later), Duration::from_secs(20));
    }

    #[test]
    fn after_the_pause_a_window_may_open_again() {
        let pause = Pause::new(LENGTH);
        let start = Instant::now();
        pause.begin(start).unwrap();
        assert_eq!(pause.begin(start + LENGTH), Ok(Some(start)));
    }

    #[test]
    fn a_window_that_did_not_open_gives_the_pause_back() {
        let pause = Pause::new(LENGTH);
        let start = Instant::now();
        let before = pause.begin(start).unwrap();
        pause.undo(before);
        assert_eq!(pause.remaining(start), Duration::ZERO);
        assert!(pause.begin(start).is_ok());
    }
}
