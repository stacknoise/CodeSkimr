//! Level 2: replaces the references of some files of the newest snapshot (ADR 0033).

use std::collections::{BTreeMap, BTreeSet, HashSet};

use codeskimr_application::ports::ReferenceReplacement;
use codeskimr_domain::{FileId, ProjectId, Snapshot, SnapshotId};
use rusqlite::{Connection, params};

use crate::codec::{column_int, id_from};
use crate::error::{StorageError, StorageResult};
use crate::headers::{insert_reference, latest_snapshot, set_totals, totals};
use crate::projects::load_layers;
use crate::read_models;
use crate::sql;

/// Whether `query` finds a row for the snapshot and the id.
fn holds(conn: &Connection, query: &str, snapshot: i64, id: u64) -> StorageResult<bool> {
    Ok(conn
        .prepare_cached(query)?
        .query_row(params![snapshot, column_int(id)?], |row| row.get(0))?)
}

/// Fails with `DanglingReference` unless every file is in the snapshot and every reference stands
/// in one of those files and joins two symbols of the snapshot.
fn check(conn: &Connection, snapshot: i64, change: &ReferenceReplacement) -> StorageResult<()> {
    let files: HashSet<u64> = change.files.iter().map(|file| file.get()).collect();
    let mut ok = true;
    for file in &files {
        ok &= holds(conn, sql::FILE_IN_SNAPSHOT, snapshot, *file)?;
    }
    for reference in &change.references {
        ok &= files.contains(&reference.file.get())
            && holds(
                conn,
                sql::SYMBOL_IN_SNAPSHOT,
                snapshot,
                reference.from.get(),
            )?
            && holds(conn, sql::SYMBOL_IN_SNAPSHOT, snapshot, reference.to.get())?;
    }
    if ok {
        Ok(())
    } else {
        Err(StorageError::DanglingReference)
    }
}

fn project_of(conn: &Connection, snapshot: i64) -> StorageResult<ProjectId> {
    let id: i64 = conn
        .query_row(sql::SNAPSHOT_PROJECT, params![snapshot], |row| row.get(0))
        .map_err(|_| StorageError::DanglingReference)?;
    Ok(ProjectId::new(id_from(id)?))
}

/// Replaces the references of the files in the newest snapshot of its project; any other snapshot
/// is refused, because its files may be shared with newer ones.
pub(crate) fn replace_references(
    conn: &Connection,
    change: &ReferenceReplacement,
) -> StorageResult<Snapshot> {
    let snapshot = column_int(change.snapshot.get())?;
    let project = project_of(conn, snapshot)?;
    let latest = latest_snapshot(conn, project)?.ok_or(StorageError::DanglingReference)?;
    if latest.id != change.snapshot {
        return Err(StorageError::DanglingReference);
    }
    check(conn, snapshot, change)?;
    for file in &change.files {
        conn.execute(
            sql::REFERENCES_OF_FILE_DELETE,
            params![column_int(file.get())?],
        )?;
    }
    let holders = holders(conn, change)?;
    let mut insert = conn.prepare_cached(sql::REFERENCE_INSERT)?;
    for reference in &change.references {
        // A reference is visible from the oldest snapshot that holds its file, so every snapshot
        // that shares the file shows the same references.
        let first = holders
            .oldest
            .get(&reference.file)
            .copied()
            .unwrap_or(snapshot);
        insert_reference(&mut insert, first, reference)?;
    }
    for affected in &holders.all {
        refresh(conn, *affected, project)?;
    }
    let stats = totals(conn, change.snapshot)?;
    Ok(Snapshot { stats, ..latest })
}

/// The snapshots that hold the replaced files. They share the rows of a file (ADR 0032), so their
/// totals and read models change with it.
struct Holders {
    all: BTreeSet<SnapshotId>,
    /// The oldest snapshot of each file, as a raw id.
    oldest: BTreeMap<FileId, i64>,
}

fn holders(conn: &Connection, change: &ReferenceReplacement) -> StorageResult<Holders> {
    let mut found = Holders {
        all: BTreeSet::new(),
        oldest: BTreeMap::new(),
    };
    let mut statement = conn.prepare_cached(sql::SNAPSHOTS_WITH_FILE)?;
    for file in &change.files {
        let rows =
            statement.query_map(params![column_int(file.get())?], |row| row.get::<_, i64>(0))?;
        for row in rows {
            let id = row?;
            found.all.insert(SnapshotId::new(id_from(id)?));
            let oldest = found.oldest.entry(*file).or_insert(id);
            *oldest = (*oldest).min(id);
        }
    }
    Ok(found)
}

/// Writes the totals of `snapshot` and builds its read models again.
fn refresh(conn: &Connection, snapshot: SnapshotId, project: ProjectId) -> StorageResult<()> {
    let id = column_int(snapshot.get())?;
    set_totals(conn, snapshot, totals(conn, snapshot)?)?;
    for delete in sql::READ_MODELS_DELETE {
        conn.execute(delete, params![id])?;
    }
    read_models::rebuild(conn, snapshot, &load_layers(conn, project)?)
}
