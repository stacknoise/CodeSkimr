//! The header of a snapshot as stored: the newest one of a project, its totals and the writing of
//! references and metrics, shared by the writes of `snapshots` and `replace`.

use codeskimr_domain::{
    CommitHash, Metric, ProjectId, Reference, Snapshot, SnapshotId, SnapshotStats, Symbol,
    Timestamp, Visibility,
};
use rusqlite::{Connection, OptionalExtension, Statement, params};

use crate::codec::{Cols, SpanColumns, column_int, id_from, label_from, small_from};
use crate::error::StorageResult;
use crate::sql;

pub(crate) fn insert_reference(
    statement: &mut Statement<'_>,
    snapshot: i64,
    reference: &Reference,
) -> StorageResult<()> {
    let span = SpanColumns::of(&reference.span);
    statement.execute(params![
        column_int(reference.id.get())?,
        snapshot,
        column_int(reference.from.get())?,
        column_int(reference.to.get())?,
        column_int(reference.file.get())?,
        span.start_line,
        span.start_column,
        span.end_line,
        span.end_column,
        reference.kind.label(),
        reference.confidence.label(),
        reference.verified,
    ])?;
    Ok(())
}

/// The totals of a snapshot; `tests` is the number of its references of kind `test`, which
/// `read_models::rebuild` counted.
pub(crate) fn totals(
    conn: &Connection,
    snapshot: SnapshotId,
    tests: u64,
) -> StorageResult<SnapshotStats> {
    let (files, symbols, references): (i64, i64, i64) = conn.query_row(
        sql::TOTALS,
        params![column_int(snapshot.get())?, column_int(tests)?],
        |row| {
            let cols = Cols::new(row);
            Ok((cols.take()?, cols.take()?, cols.take()?))
        },
    )?;
    Ok(SnapshotStats {
        files: small_from(files)?,
        symbols: small_from(symbols)?,
        references: small_from(references)?,
    })
}

pub(crate) fn set_totals(
    conn: &Connection,
    snapshot: SnapshotId,
    stats: SnapshotStats,
) -> StorageResult<()> {
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

/// A metric of its own file is shared by all snapshots; coupling belongs to this snapshot alone
/// (ADR 0037).
pub(crate) fn insert_metric(
    conn: &Connection,
    snapshot: i64,
    metric: &Metric,
) -> StorageResult<()> {
    let symbol = column_int(metric.symbol.get())?;
    if metric.kind.is_per_snapshot() {
        conn.prepare_cached(sql::COUPLING_INSERT)?.execute(params![
            snapshot,
            symbol,
            metric.kind.label(),
            metric.value.get(),
        ])?;
    } else {
        conn.prepare_cached(sql::METRIC_INSERT)?.execute(params![
            symbol,
            metric.kind.label(),
            metric.value.get(),
        ])?;
    }
    Ok(())
}

/// Writes a symbol row with the content hash of the parse stage, `None` for a symbol that has none.
pub(crate) fn insert_symbol(
    conn: &Connection,
    symbol: &Symbol,
    content_hash: Option<&str>,
) -> StorageResult<()> {
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
        content_hash,
    ])?;
    Ok(())
}
