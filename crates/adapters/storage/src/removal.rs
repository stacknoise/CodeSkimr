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
    BULK_REFERENCES_MIN, BULK_REFERENCES_SHARE_DIVISOR, BULK_REMOVAL_REFERENCES_MAX,
    REMOVAL_FILES_PER_SLICE, REMOVAL_PAUSE_MIN_ROWS, REMOVAL_PAUSE_MS, REMOVAL_ROWS_PER_PART,
};
use crate::error::StorageError;
use crate::snapshots::{delete_shared_rows, delete_snapshot};
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
            let snapshots = tx
                .prepare_cached(sql::SNAPSHOTS_OF_PROJECT)?
                .query_map(params![project], |row| row.get::<_, i64>(0))?
                .collect::<Result<Vec<_>, _>>()?;
            if bulk_removal_starts(tx, project)? {
                remove_snapshots_at_once(tx, &snapshots)?;
                return Ok(Vec::new());
            }
            Ok(snapshots)
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
        for delete in sql::SNAPSHOT_PART_DELETE {
            self.delete_in_parts(snapshot, delete)?;
        }
        for delete in sql::SNAPSHOT_CONTENT_DELETE {
            let deleted =
                self.write(|tx| Ok(tx.prepare_cached(delete)?.execute(params![snapshot])?))?;
            // A table with few rows was a short write; nobody waited for it.
            if deleted > REMOVAL_PAUSE_MIN_ROWS {
                pause();
            }
        }
        self.write(|tx| {
            tx.execute(sql::SNAPSHOT_PRUNE, params![snapshot])?;
            Ok(())
        })
    }
}

impl SqliteStore {
    /// Deletes the rows of a snapshot that `delete` names, a part of them per write, until none is
    /// left, with a pause after each part so that other writers get their turn.
    fn delete_in_parts(&self, snapshot: i64, delete: &str) -> Result<(), StorageError> {
        let last = i64::try_from(REMOVAL_ROWS_PER_PART - 1).unwrap_or(i64::MAX);
        loop {
            let deleted = self.write(|tx| {
                Ok(tx
                    .prepare_cached(delete)?
                    .execute(params![snapshot, last])?)
            })?;
            if deleted == 0 {
                return Ok(());
            }
            pause();
        }
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
    // A larger project would hold the store for longer than another process waits for it.
    if !(BULK_REFERENCES_MIN..=BULK_REMOVAL_REFERENCES_MAX).contains(&own) {
        return Ok(false);
    }
    let all: i64 = conn.query_row(sql::REFERENCE_COUNT, [], |row| row.get(0))?;
    let rest = usize::try_from(all)
        .unwrap_or(usize::MAX)
        .saturating_sub(own);
    Ok(own >= rest / BULK_REFERENCES_SHARE_DIVISOR)
}

/// Deletes the snapshots of the project one after the other, each with everything it owns, between
/// dropping and building again the indexes of the references and of the fingerprints. A snapshot
/// goes whole before the next, so that the rows the two shared go with the second.
fn remove_snapshots_at_once(
    conn: &rusqlite::Connection,
    snapshots: &[i64],
) -> Result<(), StorageError> {
    for drop in sql::REFERENCE_INDEXES_DROP {
        conn.execute(drop, [])?;
    }
    conn.execute(sql::FINGERPRINT_INDEX_DROP, [])?;
    for snapshot in snapshots {
        delete_snapshot(conn, *snapshot)?;
    }
    for create in sql::REFERENCE_INDEXES_CREATE {
        conn.execute(create, [])?;
    }
    conn.execute(sql::FINGERPRINT_INDEX_CREATE, [])?;
    Ok(())
}
