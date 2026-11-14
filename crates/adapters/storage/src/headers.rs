//! The header of a snapshot as stored: the newest one of a project, its totals and the writing of
//! references, shared by the writes of `snapshots` and `replace`.

use codeskimr_domain::{
    CommitHash, ProjectId, Reference, Snapshot, SnapshotId, SnapshotStats, Timestamp,
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

pub(crate) fn totals(conn: &Connection, snapshot: SnapshotId) -> StorageResult<SnapshotStats> {
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
