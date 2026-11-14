//! Which projects are being scanned right now.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use codeskimr_application::cancel::CancelToken;
use codeskimr_application::error::CodedError;
use codeskimr_codes::ErrorCode;
use codeskimr_domain::ProjectId;

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
    /// `PRJ-302` if a scan of the project is already running.
    pub fn start(&self, project: ProjectId) -> Result<ScanGuard, CodedError> {
        let mut running = self.lock();
        if running.contains_key(&project.get()) {
            return Err(CodedError::new(ErrorCode::SnapshotInProgress));
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
