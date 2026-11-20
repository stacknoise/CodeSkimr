//! Which level 2 runs are going on, and the two limits of ADR 0072: at most
//! `level2.max_parallel_runs` runs at the same time over all projects, and a pause after a run
//! that the browser started has ended, before the same project may start another one.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use codeskimr_domain::{ProjectId, SkipReason};

/// The two limits, from the user configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunLimits {
    /// Runs that may go on at the same time.
    pub max_parallel: u32,
    /// Pause after a run that the browser started has ended.
    pub cooldown: Duration,
}

#[derive(Debug, Default)]
struct State {
    active: u32,
    ended: BTreeMap<u64, Instant>,
}

/// The registry of level 2 runs, shared by every command of the UI.
#[derive(Debug, Clone, Default)]
pub struct Level2Runs(Arc<Mutex<State>>);

/// Holds a place for one run until it is dropped; dropping it, whatever the run did, starts the
/// pause of the project.
#[derive(Debug)]
pub struct RunSlot {
    runs: Level2Runs,
    project: ProjectId,
}

impl Level2Runs {
    fn lock(&self) -> MutexGuard<'_, State> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Takes a place for a run of `project` at the time `now`.
    ///
    /// # Errors
    /// `paused` if a run of the project ended less than the pause ago, `busy` if the places are
    /// taken. Neither is an error of the scan: it stores level 1 and says why.
    pub fn begin(
        &self,
        project: ProjectId,
        limits: RunLimits,
        now: Instant,
    ) -> Result<RunSlot, SkipReason> {
        let mut state = self.lock();
        let paused = state
            .ended
            .get(&project.get())
            .is_some_and(|ended| now.saturating_duration_since(*ended) < limits.cooldown);
        if paused {
            return Err(SkipReason::Paused);
        }
        if state.active >= limits.max_parallel {
            return Err(SkipReason::Busy);
        }
        state.active += 1;
        Ok(RunSlot {
            runs: self.clone(),
            project,
        })
    }
}

impl Drop for RunSlot {
    fn drop(&mut self) {
        let mut state = self.runs.lock();
        state.active = state.active.saturating_sub(1);
        state.ended.insert(self.project.get(), Instant::now());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn limits(cooldown: u64) -> RunLimits {
        RunLimits {
            max_parallel: 1,
            cooldown: Duration::from_secs(cooldown),
        }
    }

    #[test]
    fn a_second_run_finds_the_place_taken_whatever_the_project() {
        let runs = Level2Runs::default();
        let first = runs
            .begin(ProjectId::new(1), limits(0), Instant::now())
            .unwrap();
        assert_eq!(
            runs.begin(ProjectId::new(2), limits(0), Instant::now())
                .unwrap_err(),
            SkipReason::Busy
        );
        drop(first);
        assert!(
            runs.begin(ProjectId::new(2), limits(0), Instant::now())
                .is_ok()
        );
    }

    #[test]
    fn the_same_project_waits_after_a_run_but_another_one_does_not() {
        let runs = Level2Runs::default();
        drop(
            runs.begin(ProjectId::new(1), limits(60), Instant::now())
                .unwrap(),
        );
        assert_eq!(
            runs.begin(ProjectId::new(1), limits(60), Instant::now())
                .unwrap_err(),
            SkipReason::Paused
        );
        assert!(
            runs.begin(ProjectId::new(2), limits(60), Instant::now())
                .is_ok()
        );
    }

    #[test]
    fn the_pause_is_over_after_the_cooldown() {
        let runs = Level2Runs::default();
        drop(
            runs.begin(ProjectId::new(1), limits(60), Instant::now())
                .unwrap(),
        );
        let later = Instant::now() + Duration::from_secs(61);
        assert!(runs.begin(ProjectId::new(1), limits(60), later).is_ok());
    }

    #[test]
    fn a_run_that_ended_in_any_way_starts_the_pause() {
        let runs = Level2Runs::default();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _slot = runs
                .begin(ProjectId::new(1), limits(60), Instant::now())
                .unwrap();
            panic!("the run broke");
        }));
        assert!(result.is_err());
        assert_eq!(
            runs.begin(ProjectId::new(1), limits(60), Instant::now())
                .unwrap_err(),
            SkipReason::Paused
        );
    }
}
