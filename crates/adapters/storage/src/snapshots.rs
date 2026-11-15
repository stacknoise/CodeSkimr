//! The `SnapshotStore` port: one snapshot written atomically, with its read models.

use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{Level2Replacement, SnapshotData, SnapshotStore};
use codeskimr_domain::{
    CommitHash, ProjectId, RelativePath, ScanMode, Snapshot, SnapshotId, SourceFile, Symbol,
    Visibility,
};
use rusqlite::{Connection, params};

use crate::codec::{SpanColumns, column_int};
use crate::consts::{
    BULK_REFERENCES_MIN, BULK_REFERENCES_SHARE_DIVISOR, PRAGMA_DEFER_FOREIGN_KEYS, PRAGMA_ON,
};
use crate::error::{StorageError, StorageResult};
use crate::headers::{insert_metric, insert_reference, set_totals, totals};
use crate::integrity;
use crate::pending;
use crate::projects::load_layers;
use crate::read_models;
use crate::similarity;
use crate::sql;
use crate::store::SqliteStore;

impl SnapshotStore for SqliteStore {
    fn persist(&self, data: SnapshotData) -> Result<Snapshot, CodedError> {
        let stored = self
            .write(|tx| write_snapshot(tx, &data, self.retention().get()))
            .map_err(CodedError::from)?;
        self.refresh_lookup(stored.id);
        Ok(stored)
    }

    fn replace_level2(&self, replacement: Level2Replacement) -> Result<Snapshot, CodedError> {
        let stored = self
            .write(|tx| crate::replace::replace_level2(tx, &replacement))
            .map_err(CodedError::from)?;
        self.refresh_lookup(stored.id);
        Ok(stored)
    }
}

fn write_snapshot(conn: &Connection, data: &SnapshotData, keep: u32) -> StorageResult<Snapshot> {
    let header = &data.snapshot;
    let project = column_int(header.project.get())?;
    let known: i64 = conn.query_row(sql::PROJECT_EXISTS, params![project], |row| row.get(0))?;
    if known == 0 {
        return Err(StorageError::ProjectNotFound);
    }
    conn.pragma_update(None, PRAGMA_DEFER_FOREIGN_KEYS, PRAGMA_ON)?;
    insert_header(conn, header)?;
    conn.execute(
        sql::SNAPSHOT_NAMING_SET,
        params![data.naming_rules, column_int(header.id.get())?],
    )?;
    insert_facts(conn, data)?;
    insert_parse_times(conn, data)?;
    if header.mode == ScanMode::Incremental {
        carry_over(conn, header.project, header.id, &data.removed)?;
    }
    integrity::check(conn, data)?;
    // The totals count the references from the usage counts, so the read models come first.
    let layers = load_layers(conn, header.project)?;
    read_models::rebuild(conn, header.id, &layers)?;
    let stats = totals(conn, header.id)?;
    set_totals(conn, header.id, stats)?;
    prune(conn, project, keep)?;
    Ok(Snapshot {
        stats,
        ..header.clone()
    })
}

/// Deletes all but the newest `keep` snapshots of the project, table by table.
fn prune(conn: &Connection, project: i64, keep: u32) -> StorageResult<()> {
    let mut statement = conn.prepare_cached(sql::SNAPSHOTS_EXPIRED)?;
    let expired = statement
        .query_map(params![project, keep], |row| row.get::<_, i64>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    for snapshot in expired {
        for delete in sql::SHARED_ROWS_DELETE
            .iter()
            .chain(sql::SNAPSHOT_CONTENT_DELETE)
        {
            conn.prepare_cached(delete)?.execute(params![snapshot])?;
        }
        conn.execute(sql::SNAPSHOT_PRUNE, params![snapshot])?;
    }
    Ok(())
}

fn insert_header(conn: &Connection, snapshot: &Snapshot) -> StorageResult<()> {
    conn.execute(
        sql::SNAPSHOT_INSERT,
        params![
            column_int(snapshot.id.get())?,
            column_int(snapshot.project.get())?,
            snapshot.commit.as_ref().map(CommitHash::as_str),
            snapshot.created.0,
            snapshot.mode.label(),
            i64::from(snapshot.stats.files),
            i64::from(snapshot.stats.symbols),
            i64::from(snapshot.stats.references),
        ],
    )?;
    Ok(())
}

fn insert_facts(conn: &Connection, data: &SnapshotData) -> StorageResult<()> {
    let snapshot = column_int(data.snapshot.id.get())?;
    for file in &data.files {
        insert_file(conn, snapshot, file)?;
    }
    for symbol in &data.symbols {
        insert_symbol(conn, symbol)?;
    }
    let bulk = bulk_load_starts(conn, data.references.len())?;
    let mut references = conn.prepare_cached(sql::REFERENCE_INSERT)?;
    for reference in &data.references {
        insert_reference(&mut references, snapshot, reference)?;
    }
    if bulk {
        for create in sql::REFERENCE_INDEXES_CREATE {
            conn.execute(create, [])?;
        }
    }
    for metric in &data.metrics {
        insert_metric(conn, snapshot, metric)?;
    }
    similarity::insert_all(conn, &data.fingerprints)?;
    pending::insert_all(conn, &data.pending)?;
    pending::mark_stored(conn, snapshot)?;
    let mut dropped = conn.prepare_cached(sql::REFERENCE_DELETE)?;
    for id in &data.dropped_references {
        dropped.execute(params![column_int(id.get())?])?;
    }
    Ok(())
}

/// Drops the reference indexes when so many rows arrive that building them afterwards is cheaper
/// than keeping them current; the caller builds them again. Returns whether it did.
fn bulk_load_starts(conn: &Connection, incoming: usize) -> StorageResult<bool> {
    if incoming < BULK_REFERENCES_MIN {
        return Ok(false);
    }
    let stored: i64 = conn.query_row(sql::REFERENCE_COUNT, [], |row| row.get(0))?;
    let stored = usize::try_from(stored).unwrap_or(usize::MAX);
    if incoming < stored / BULK_REFERENCES_SHARE_DIVISOR {
        return Ok(false);
    }
    for drop in sql::REFERENCE_INDEXES_DROP {
        conn.execute(drop, [])?;
    }
    Ok(true)
}

/// The parse times of a full scan; an incremental scan has none of its own and copies the
/// previous ones in `carry_over`.
fn insert_parse_times(conn: &Connection, data: &SnapshotData) -> StorageResult<()> {
    let snapshot = column_int(data.snapshot.id.get())?;
    let mut insert = conn.prepare_cached(sql::PARSE_TIME_INSERT)?;
    for (language, micros) in &data.parse_micros {
        insert.execute(params![snapshot, language.as_str(), column_int(*micros)?])?;
    }
    Ok(())
}

fn insert_file(conn: &Connection, snapshot: i64, file: &SourceFile) -> StorageResult<()> {
    conn.prepare_cached(sql::FILE_INSERT)?.execute(params![
        snapshot,
        column_int(file.id.get())?,
        file.path.as_str(),
        file.language.as_str(),
        file.content_hash.as_str(),
        i64::from(file.loc.0),
    ])?;
    Ok(())
}

pub(crate) fn insert_symbol(conn: &Connection, symbol: &Symbol) -> StorageResult<()> {
    let parent = symbol
        .parent
        .map(|parent| column_int(parent.get()))
        .transpose()?;
    let span = SpanColumns::of(&symbol.span);
    conn.prepare_cached(sql::SYMBOL_INSERT)?.execute(params![
        column_int(symbol.id.get())?,
        column_int(symbol.file.get())?,
        symbol.kind.label(),
        symbol.name.as_str(),
        symbol.qualified_name.as_str(),
        parent,
        symbol.visibility.map(Visibility::label),
        symbol.is_static,
        span.start_line,
        span.start_column,
        span.end_line,
        span.end_column,
        symbol.signature,
        symbol.doc,
        symbol.origin.label(),
    ])?;
    Ok(())
}

/// Takes over what an incremental scan did not analyse again from the previous snapshot, except
/// the files that were removed; their symbols and references stay behind with them.
fn carry_over(
    conn: &Connection,
    project: ProjectId,
    snapshot: SnapshotId,
    removed: &[RelativePath],
) -> StorageResult<()> {
    let new = column_int(snapshot.get())?;
    let previous: Option<i64> = conn.query_row(
        sql::PREVIOUS_SNAPSHOT,
        params![column_int(project.get())?, new],
        |row| row.get(0),
    )?;
    let Some(previous) = previous else {
        return Ok(());
    };
    conn.execute(sql::CARRY_FILES, params![previous, new])?;
    for path in removed {
        conn.execute(sql::FILE_REMOVE, params![new, path.as_str()])?;
    }
    conn.execute(sql::PARSE_TIME_CARRY, params![previous, new])?;
    // Symbols, metrics, references and fingerprints are shared and need no copy (ADR 0032).
    Ok(())
}
