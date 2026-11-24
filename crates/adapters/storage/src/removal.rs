//! Removing a project: the `ProjectRemoval` port.
//!
//! A large project holds millions of rows, so deleting them in one write would keep the store from
//! every other write (a rename of another project, a scan) for minutes. The removal therefore goes
//! in many short writes: the shared rows of a few hundred files, then the other tables of one
//! snapshot, then the snapshot, and at last the project. A removal that stops half way leaves a
//! project with fewer rows, which is removed by the same call again.

use std::time::Duration;

use codeskimr_application::error::CodedError;
use codeskimr_application::ports::ProjectRemoval;
use codeskimr_domain::ProjectId;
use rusqlite::params;

use crate::codec::column_int;
use crate::consts::{REMOVAL_FILES_PER_SLICE, REMOVAL_PAUSE_MS};
use crate::error::StorageError;
use crate::snapshots::delete_shared_rows;
use crate::sql;
use crate::store::SqliteStore;

impl ProjectRemoval for SqliteStore {
    fn remove_project(&self, id: ProjectId) -> Result<(), CodedError> {
        self.remove_in_slices(column_int(id.get()).map_err(CodedError::from)?)
            .map_err(CodedError::from)
    }
}

impl SqliteStore {
    fn remove_in_slices(&self, project: i64) -> Result<(), StorageError> {
        let snapshots = self.write(|tx| {
            let known: i64 =
                tx.query_row(sql::PROJECT_EXISTS, params![project], |row| row.get(0))?;
            if known == 0 {
                return Err(StorageError::ProjectNotFound);
            }
            // From here on the project is no project for anyone else; a removal that stops is
            // started again by the same call.
            tx.execute(sql::PROJECT_REMOVING_SET, params![project])?;
            Ok(tx
                .prepare_cached(sql::SNAPSHOTS_OF_PROJECT)?
                .query_map(params![project], |row| row.get::<_, i64>(0))?
                .collect::<Result<Vec<_>, _>>()?)
        })?;
        for snapshot in snapshots {
            self.remove_snapshot_in_slices(snapshot)?;
        }
        self.write(|tx| {
            tx.execute(sql::PROJECT_DELETE, params![project])?;
            Ok(())
        })
    }

    /// Deletes one snapshot: the shared rows by slices of files, then table by table.
    fn remove_snapshot_in_slices(&self, snapshot: i64) -> Result<(), StorageError> {
        let files = self.write(|tx| {
            Ok(tx
                .prepare_cached(sql::SNAPSHOT_FILE_IDS)?
                .query_map(params![snapshot], |row| row.get::<_, i64>(0))?
                .collect::<Result<Vec<_>, _>>()?)
        })?;
        for slice in files.chunks(REMOVAL_FILES_PER_SLICE) {
            if let (Some(first), Some(last)) = (slice.first(), slice.last()) {
                self.write(|tx| delete_shared_rows(tx, snapshot, *first, *last))?;
                pause();
            }
        }
        for delete in sql::SNAPSHOT_CONTENT_DELETE {
            self.write(|tx| {
                tx.prepare_cached(delete)?.execute(params![snapshot])?;
                Ok(())
            })?;
            pause();
        }
        self.write(|tx| {
            tx.execute(sql::SNAPSHOT_PRUNE, params![snapshot])?;
            Ok(())
        })
    }
}

/// Gives a write that waits for the connection its turn.
fn pause() {
    std::thread::sleep(Duration::from_millis(REMOVAL_PAUSE_MS));
}
