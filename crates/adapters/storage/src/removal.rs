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
use crate::consts::{
    BULK_REFERENCES_MIN, BULK_REFERENCES_SHARE_DIVISOR, REMOVAL_FILES_PER_SLICE, REMOVAL_PAUSE_MS,
};
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
        let (snapshots, bulk) = self.write(|tx| {
            let known: i64 =
                tx.query_row(sql::PROJECT_EXISTS, params![project], |row| row.get(0))?;
            if known == 0 {
                return Err(StorageError::ProjectNotFound);
            }
            // From here on the project is no project for anyone else; a removal that stops is
            // started again by the same call.
            tx.execute(sql::PROJECT_REMOVING_SET, params![project])?;
            let snapshots = tx
                .prepare_cached(sql::SNAPSHOTS_OF_PROJECT)?
                .query_map(params![project], |row| row.get::<_, i64>(0))?
                .collect::<Result<Vec<_>, _>>()?;
            let bulk = bulk_removal_starts(tx, project)?;
            if bulk {
                remove_shared_rows_at_once(tx, &snapshots)?;
            }
            Ok((snapshots, bulk))
        })?;
        for snapshot in snapshots {
            self.remove_snapshot_in_slices(snapshot, bulk)?;
        }
        self.write(|tx| {
            tx.execute(sql::PROJECT_DELETE, params![project])?;
            Ok(())
        })
    }

    /// Deletes one snapshot: the shared rows by slices of files, then table by table.
    /// `shared_gone` says that the shared rows went already with the first write.
    fn remove_snapshot_in_slices(
        &self,
        snapshot: i64,
        shared_gone: bool,
    ) -> Result<(), StorageError> {
        let files = if shared_gone {
            Vec::new()
        } else {
            self.write(|tx| {
                Ok(tx
                    .prepare_cached(sql::SNAPSHOT_FILE_IDS)?
                    .query_map(params![snapshot], |row| row.get::<_, i64>(0))?
                    .collect::<Result<Vec<_>, _>>()?)
            })?
        };
        for slice in files.chunks(REMOVAL_FILES_PER_SLICE) {
            if let (Some(first), Some(last)) = (slice.first(), slice.last()) {
                self.write(|tx| delete_shared_rows(tx, snapshot, *first, *last))?;
                pause();
            }
        }
        for delete in sql::SNAPSHOT_CONTENT_DELETE {
            let deleted =
                self.write(|tx| Ok(tx.prepare_cached(delete)?.execute(params![snapshot])?))?;
            // A table with few rows was a short write; nobody waited for it.
            if deleted > REMOVAL_FILES_PER_SLICE {
                pause();
            }
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

/// Whether the project owns so many of the stored references that its shared rows are best
/// deleted in one write with the indexes dropped. With live indexes every deleted reference costs
/// several index updates (18 s for 2.5 million references against 1.3 s without them); building
/// the indexes again over what is left costs less than that as long as the project owns about as
/// many references as all other projects together, the same trade as when a large scan is loaded.
/// The write holds the store for a few seconds then; otherwise the removal goes in slices.
fn bulk_removal_starts(conn: &rusqlite::Connection, project: i64) -> Result<bool, StorageError> {
    let own: i64 = conn.query_row(sql::PROJECT_REFERENCE_COUNT, params![project], |row| {
        row.get(0)
    })?;
    let own = usize::try_from(own).unwrap_or(usize::MAX);
    if own < BULK_REFERENCES_MIN {
        return Ok(false);
    }
    let all: i64 = conn.query_row(sql::REFERENCE_COUNT, [], |row| row.get(0))?;
    let rest = usize::try_from(all)
        .unwrap_or(usize::MAX)
        .saturating_sub(own);
    Ok(own >= rest / BULK_REFERENCES_SHARE_DIVISOR)
}

/// Deletes the shared rows of all snapshots of the project in one go, between dropping and
/// building again the indexes of the references and of the fingerprints.
fn remove_shared_rows_at_once(
    conn: &rusqlite::Connection,
    snapshots: &[i64],
) -> Result<(), StorageError> {
    for drop in sql::REFERENCE_INDEXES_DROP {
        conn.execute(drop, [])?;
    }
    conn.execute(sql::FINGERPRINT_INDEX_DROP, [])?;
    for snapshot in snapshots {
        delete_shared_rows(conn, *snapshot, i64::MIN, i64::MAX)?;
    }
    for create in sql::REFERENCE_INDEXES_CREATE {
        conn.execute(create, [])?;
    }
    conn.execute(sql::FINGERPRINT_INDEX_CREATE, [])?;
    Ok(())
}
