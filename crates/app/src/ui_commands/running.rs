//! Which projects are being scanned right now.

use std::collections::BTreeSet;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use codeskimr_application::error::CodedError;
use codeskimr_codes::ErrorCode;
use codeskimr_domain::ProjectId;

/// The projects with a running scan; a second scan of the same project is refused (`PRJ-302`).
#[derive(Debug, Clone, Default)]
pub struct RunningScans(Arc<Mutex<BTreeSet<u64>>>);

/// Marks a project as scanning until it is dropped.
#[derive(Debug)]
pub struct ScanGuard {
    scans: RunningScans,
    project: ProjectId,
}

impl RunningScans {
    fn lock(&self) -> MutexGuard<'_, BTreeSet<u64>> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Marks `project` as scanning.
    ///
    /// # Errors
    /// `PRJ-302` if a scan of the project is already running.
    pub fn start(&self, project: ProjectId) -> Result<ScanGuard, CodedError> {
        if self.lock().insert(project.get()) {
            Ok(ScanGuard {
                scans: self.clone(),
                project,
            })
        } else {
            Err(CodedError::new(ErrorCode::SnapshotInProgress))
        }
    }

    /// Whether a scan of `project` is running.
    #[must_use]
    pub fn is_running(&self, project: ProjectId) -> bool {
        self.lock().contains(&project.get())
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
}
