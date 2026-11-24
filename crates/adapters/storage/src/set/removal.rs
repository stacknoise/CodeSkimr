//! Removing a project: closing its file and deleting it (ADR 0095).

use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use codeskimr_application::error::CodedError;
use codeskimr_application::ports::ProjectRemoval;
use codeskimr_domain::ProjectId;

use super::SqliteStore;
use super::consts::{REMOVAL_PAUSE_MS, REMOVAL_STILL_USED, REMOVAL_WAIT_MS};
use super::files::delete_with_log;
use crate::error::{StorageError, StorageResult};
use crate::store::ProjectDb;

impl ProjectRemoval for SqliteStore {
    fn remove_project(&self, id: ProjectId) -> Result<(), CodedError> {
        self.removed(id).map_err(CodedError::from)
    }
}

impl SqliteStore {
    fn removed(&self, id: ProjectId) -> StorageResult<()> {
        if self.registry.removing(id)?.is_none() {
            return Err(StorageError::ProjectNotFound);
        }
        // From here on the project is no project for anyone else; a removal that stops is
        // finished by the same call, or at the next start.
        self.registry.mark_removing(id)?;
        if let Ok(db) = self.db_any(id) {
            db.mark_removing(id)?;
        }
        self.remove_marked(id)
    }

    /// Closes the file of a marked project once no call uses it, deletes it and then the row of
    /// the catalog. A file that the system still holds stays marked and is removed at the next
    /// start.
    pub(super) fn remove_marked(&self, id: ProjectId) -> StorageResult<()> {
        let held = self.forget_file(id);
        let deadline = Instant::now() + Duration::from_millis(REMOVAL_WAIT_MS);
        if let Some(db) = held {
            wait_until_alone(db, deadline)?;
        }
        if let Some(path) = self.path_of(id) {
            retry_until(deadline, || delete_with_log(&path))?;
        }
        self.registry.delete(id)
    }
}

/// Waits until `db` is used by no one else, then closes it (a closed `SQLite` file can be deleted
/// on Windows).
fn wait_until_alone(mut db: Arc<ProjectDb>, deadline: Instant) -> StorageResult<()> {
    loop {
        match Arc::try_unwrap(db) {
            Ok(alone) => {
                drop(alone);
                return Ok(());
            }
            Err(shared) => db = shared,
        }
        if Instant::now() >= deadline {
            return Err(StorageError::Io(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                REMOVAL_STILL_USED,
            )));
        }
        thread::sleep(Duration::from_millis(REMOVAL_PAUSE_MS));
    }
}

/// Runs `attempt` until it succeeds or the deadline passes; the last failure is returned.
fn retry_until(
    deadline: Instant,
    mut attempt: impl FnMut() -> std::io::Result<()>,
) -> StorageResult<()> {
    loop {
        match attempt() {
            Ok(()) => return Ok(()),
            Err(error) if Instant::now() >= deadline => return Err(StorageError::Io(error)),
            Err(_) => thread::sleep(Duration::from_millis(REMOVAL_PAUSE_MS)),
        }
    }
}
