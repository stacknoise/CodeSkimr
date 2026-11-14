//! Level 2: replaces the references of some files of the newest snapshot (ADR 0033).

use std::collections::HashSet;

use codeskimr_application::ports::ReferenceReplacement;
use codeskimr_domain::{ProjectId, Snapshot};
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
    let mut insert = conn.prepare_cached(sql::REFERENCE_INSERT)?;
    for reference in &change.references {
        insert_reference(&mut insert, snapshot, reference)?;
    }
    let stats = totals(conn, change.snapshot)?;
    set_totals(conn, change.snapshot, stats)?;
    for delete in sql::READ_MODELS_DELETE {
        conn.execute(delete, params![snapshot])?;
    }
    read_models::rebuild(conn, change.snapshot, &load_layers(conn, project)?)?;
    Ok(Snapshot { stats, ..latest })
}
