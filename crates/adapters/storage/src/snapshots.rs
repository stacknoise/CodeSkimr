//! The `SnapshotStore` port: one snapshot written atomically, with its read models.

use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{SnapshotData, SnapshotStore};
use codeskimr_domain::{
    CommitHash, Metric, ProjectId, Reference, RelativePath, ScanMode, Snapshot, SnapshotId,
    SnapshotStats, SourceFile, Symbol, Timestamp, Visibility,
};
use rusqlite::{Connection, OptionalExtension, params};

use crate::codec::{Cols, SpanColumns, column_int, id_from, label_from, small_from};
use crate::consts::{PRAGMA_DEFER_FOREIGN_KEYS, PRAGMA_ON};
use crate::error::{StorageError, StorageResult};
use crate::projects::load_layers;
use crate::read_models;
use crate::similarity;
use crate::sql;
use crate::store::SqliteStore;

impl SnapshotStore for SqliteStore {
    fn persist(&self, data: SnapshotData) -> Result<Snapshot, CodedError> {
        self.write(|tx| write_snapshot(tx, &data, self.retention().get()))
            .map_err(CodedError::from)
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
    insert_facts(conn, data)?;
    if header.mode == ScanMode::Incremental {
        carry_over(conn, header.project, header.id, &data.removed)?;
    }
    let stats = totals(conn, header.id)?;
    set_totals(conn, header.id, stats)?;
    let layers = load_layers(conn, header.project)?;
    read_models::rebuild(conn, header.id, &layers)?;
    conn.execute(sql::SNAPSHOT_PRUNE, params![project, keep])?;
    Ok(Snapshot {
        stats,
        ..header.clone()
    })
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
        insert_symbol(conn, snapshot, symbol)?;
    }
    for reference in &data.references {
        insert_reference(conn, snapshot, reference)?;
    }
    for metric in &data.metrics {
        insert_metric(conn, snapshot, metric)?;
    }
    similarity::insert_all(conn, snapshot, &data.fingerprints)?;
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

fn insert_symbol(conn: &Connection, snapshot: i64, symbol: &Symbol) -> StorageResult<()> {
    let parent = symbol
        .parent
        .map(|parent| column_int(parent.get()))
        .transpose()?;
    let span = SpanColumns::of(&symbol.span);
    conn.prepare_cached(sql::SYMBOL_INSERT)?.execute(params![
        snapshot,
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
    ])?;
    Ok(())
}

fn insert_reference(conn: &Connection, snapshot: i64, reference: &Reference) -> StorageResult<()> {
    let span = SpanColumns::of(&reference.span);
    conn.prepare_cached(sql::REFERENCE_INSERT)?
        .execute(params![
            snapshot,
            column_int(reference.id.get())?,
            column_int(reference.from.get())?,
            column_int(reference.to.get())?,
            column_int(reference.file.get())?,
            span.start_line,
            span.start_column,
            span.end_line,
            span.end_column,
            reference.kind.label(),
            reference.confidence.label(),
        ])?;
    Ok(())
}

fn insert_metric(conn: &Connection, snapshot: i64, metric: &Metric) -> StorageResult<()> {
    conn.prepare_cached(sql::METRIC_INSERT)?.execute(params![
        snapshot,
        column_int(metric.symbol.get())?,
        metric.kind.label(),
        metric.value.get(),
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
    for statement in [
        sql::CARRY_SYMBOLS,
        sql::CARRY_REFERENCES,
        sql::CARRY_METRICS,
        sql::CARRY_FINGERPRINTS,
    ] {
        conn.execute(statement, params![previous, new])?;
    }
    Ok(())
}

fn totals(conn: &Connection, snapshot: SnapshotId) -> StorageResult<SnapshotStats> {
    let (files, symbols, references): (i64, i64, i64) =
        conn.query_row(sql::TOTALS, params![column_int(snapshot.get())?], |row| {
            let cols = Cols::new(row);
            Ok((cols.take()?, cols.take()?, cols.take()?))
        })?;
    Ok(SnapshotStats {
        files: small_from(files)?,
        symbols: small_from(symbols)?,
        references: small_from(references)?,
    })
}

fn set_totals(conn: &Connection, snapshot: SnapshotId, stats: SnapshotStats) -> StorageResult<()> {
    conn.execute(
        sql::SNAPSHOT_TOTALS,
        params![
            column_int(snapshot.get())?,
            i64::from(stats.files),
            i64::from(stats.symbols),
            i64::from(stats.references),
        ],
    )?;
    Ok(())
}

/// The newest snapshot of a project, if it has one.
pub fn latest_snapshot(conn: &Connection, project: ProjectId) -> StorageResult<Option<Snapshot>> {
    let row = conn
        .query_row(
            sql::LATEST_SNAPSHOT,
            params![column_int(project.get())?],
            |row| {
                let cols = Cols::new(row);
                Ok((
                    cols.take::<i64>()?,
                    cols.take::<Option<String>>()?,
                    cols.take::<i64>()?,
                    cols.take::<String>()?,
                    (
                        cols.take::<i64>()?,
                        cols.take::<i64>()?,
                        cols.take::<i64>()?,
                    ),
                ))
            },
        )
        .optional()?;
    row.map(
        |(id, commit, created, mode, (files, symbols, references))| {
            Ok(Snapshot {
                id: SnapshotId::new(id_from(id)?),
                project,
                commit: commit.map(CommitHash::new).transpose()?,
                created: Timestamp(created),
                mode: label_from(&mode)?,
                stats: SnapshotStats {
                    files: small_from(files)?,
                    symbols: small_from(symbols)?,
                    references: small_from(references)?,
                },
            })
        },
    )
    .transpose()
}
