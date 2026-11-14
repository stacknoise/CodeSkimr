//! Which projects are being scanned right now.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use codeskimr_application::cancel::CancelToken;
use codeskimr_application::error::CodedError;
use codeskimr_application::ports::ProjectScanner;
use codeskimr_codes::ErrorCode;
use codeskimr_domain::{Principal, ProjectId, Snapshot};

use crate::consts::MAX_PARALLEL_SCANS;

/// The projects with a running scan; a second scan of the same project is refused (`PRJ-302`).
#[derive(Debug, Clone, Default)]
pub struct RunningScans(Arc<Mutex<BTreeMap<u64, Option<CancelToken>>>>);

/// Marks a project as scanning until it is dropped.
#[derive(Debug)]
pub struct ScanGuard {
    scans: RunningScans,
    project: ProjectId,
}

impl RunningScans {
    fn lock(&self) -> MutexGuard<'_, BTreeMap<u64, Option<CancelToken>>> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Marks `project` as scanning.
    ///
    /// # Errors
    /// `PRJ-302` if a scan of the project is already running, `PRJ-501` if `MAX_PARALLEL_SCANS`
    /// scans of other projects run already.
    pub fn start(&self, project: ProjectId) -> Result<ScanGuard, CodedError> {
        let mut running = self.lock();
        if running.contains_key(&project.get()) {
            return Err(CodedError::new(ErrorCode::SnapshotInProgress));
        }
        if running.len() >= MAX_PARALLEL_SCANS {
            return Err(CodedError::new(ErrorCode::TooManyScans));
        }
        running.insert(project.get(), None);
        Ok(ScanGuard {
            scans: self.clone(),
            project,
        })
    }

    /// Remembers the token that cancels the scan of `project`, once the scan has one.
    pub fn attach(&self, project: ProjectId, token: CancelToken) {
        if let Some(slot) = self.lock().get_mut(&project.get()) {
            *slot = Some(token);
        }
    }

    /// Cancels the running scan of `project`; `false` if none is running.
    #[must_use]
    pub fn cancel(&self, project: ProjectId) -> bool {
        self.lock().get(&project.get()).is_some_and(|slot| {
            slot.as_ref().is_some_and(|token| {
                token.cancel();
                true
            })
        })
    }

    /// Whether a scan of `project` is running.
    #[must_use]
    pub fn is_running(&self, project: ProjectId) -> bool {
        self.lock().contains_key(&project.get())
    }
}

/// A scanner that lets one scan per project run at a time, sharing the lock with the UI: a
/// second `rescan` of a project that is being scanned is `PRJ-302`.
pub struct ExclusiveScans {
    inner: Box<dyn ProjectScanner>,
    running: RunningScans,
}

impl ExclusiveScans {
    /// Wraps `inner`; `running` is the lock the UI commands use too.
    pub fn new(inner: impl ProjectScanner + 'static, running: RunningScans) -> Self {
        Self {
            inner: Box::new(inner),
            running,
        }
    }
}

impl ProjectScanner for ExclusiveScans {
    fn rescan(&self, principal: &Principal, project: ProjectId) -> Result<Snapshot, CodedError> {
        let _scanning = self.running.start(project)?;
        self.inner.rescan(principal, project)
    }
}

impl Drop for ScanGuard {
    fn drop(&mut self) {
        self.scans.lock().remove(&self.project.get());
    }
}

#[cfg(test)]
mod tests {
    use codeskimr_codes::Coded;

    use super::*;

    #[test]
    fn a_project_scans_once_at_a_time() {
        let scans = RunningScans::default();
        let id = ProjectId::new(7);
        let guard = scans.start(id).unwrap();
        assert!(scans.is_running(id));
        assert_eq!(
            scans.start(id).unwrap_err().code(),
            ErrorCode::SnapshotInProgress
        );
        drop(guard);
        assert!(!scans.is_running(id));
        assert!(scans.start(id).is_ok());
    }

    struct Scanner;

    impl ProjectScanner for Scanner {
        fn rescan(&self, _: &Principal, _: ProjectId) -> Result<Snapshot, CodedError> {
            Err(CodedError::new(ErrorCode::ScanCancelled))
        }
    }

    #[test]
    fn a_rescan_is_refused_while_the_project_is_scanned_elsewhere() {
        let running = RunningScans::default();
        let exclusive = ExclusiveScans::new(Scanner, running.clone());
        let id = ProjectId::new(7);
        let principal = crate::principal::remote_client();
        let scanning = running.start(id).unwrap();
        assert_eq!(
            exclusive.rescan(&principal, id).unwrap_err().code(),
            ErrorCode::SnapshotInProgress
        );
        drop(scanning);
        // The scan itself runs (here: ends cancelled) and releases the project afterwards.
        assert_eq!(
            exclusive.rescan(&principal, id).unwrap_err().code(),
            ErrorCode::ScanCancelled
        );
        assert!(!running.is_running(id));
    }

    #[test]
    fn only_so_many_scans_run_at_the_same_time() {
        let scans = RunningScans::default();
        let guards: Vec<_> = (0..MAX_PARALLEL_SCANS as u64)
            .map(|id| scans.start(ProjectId::new(id)).unwrap())
            .collect();
        let extra = ProjectId::new(MAX_PARALLEL_SCANS as u64);
        assert_eq!(
            scans.start(extra).unwrap_err().code(),
            ErrorCode::TooManyScans
        );
        drop(guards);
        assert!(scans.start(extra).is_ok());
    }

    #[test]
    fn a_running_scan_is_cancelled_through_its_token() {
        let scans = RunningScans::default();
        let id = ProjectId::new(7);
        assert!(!scans.cancel(id));
        let _guard = scans.start(id).unwrap();
        assert!(!scans.cancel(id), "no token yet: the scan has not started");
        let token = CancelToken::new();
        scans.attach(id, token.clone());
        assert!(scans.cancel(id));
        assert!(token.is_cancelled());
    }
}
