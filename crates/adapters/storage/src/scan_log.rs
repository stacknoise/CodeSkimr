//! The log of the scans of a project: one row for every scan, also one that found nothing changed
//! (ADR 0093, ADR 0097). A scan that writes a snapshot logs it as its own; a scan that confirms the
//! newest snapshot logs the snapshot it confirmed, so several rows can share one snapshot. The rows
//! go with their snapshot (`ON DELETE CASCADE`), and a snapshot keeps at most the newest
//! `snapshots.retention_count` rows besides the one of the scan that wrote it.

use codeskimr_application::ports::{Confirmation, ScanEntry, SnapshotData};
use codeskimr_domain::{CommitHash, ProjectId, ScanMode, SnapshotId, Timestamp};
use rusqlite::{Connection, params};
use std::time::Instant;

use crate::codec::{Cols, column_int, id_from, label_from, small_from};
use crate::consts::UNCHANGED_FILES;
use crate::error::StorageResult;
use crate::sql;

/// How long a scan that began at `began` has taken, in milliseconds, saturating.
fn elapsed_ms(began: Instant) -> i64 {
    i64::try_from(began.elapsed().as_millis()).unwrap_or(i64::MAX)
}

/// One scan as the log stores it.
struct Logged<'a> {
    snapshot: SnapshotId,
    created: Timestamp,
    commit: Option<&'a CommitHash>,
    mode: ScanMode,
    began: Option<Instant>,
    files_changed: Option<u32>,
}

fn insert(conn: &Connection, scan: &Logged<'_>) -> StorageResult<()> {
    conn.prepare_cached(sql::SCAN_LOG_INSERT)?.execute(params![
        column_int(scan.snapshot.get())?,
        scan.created.0,
        scan.commit.map(CommitHash::as_str),
        scan.mode.label(),
        scan.began.map(elapsed_ms),
        scan.files_changed.map(i64::from),
    ])?;
    Ok(())
}

/// Logs the scan that wrote the snapshot of `data`.
pub(crate) fn record_written(conn: &Connection, data: &SnapshotData) -> StorageResult<()> {
    let header = &data.snapshot;
    insert(
        conn,
        &Logged {
            snapshot: header.id,
            created: header.created,
            commit: header.commit.as_ref(),
            mode: header.mode,
            began: data.began,
            files_changed: data.files_changed,
        },
    )
}

/// Logs a scan that found nothing changed against the snapshot it confirmed (no file changed),
/// then drops the rows of that snapshot except the one of the scan that wrote it and the newest
/// `keep`.
pub(crate) fn record_confirmed(
    conn: &Connection,
    scan: &Confirmation,
    keep: u32,
) -> StorageResult<()> {
    insert(
        conn,
        &Logged {
            snapshot: scan.snapshot,
            created: scan.at,
            commit: scan.commit.as_ref(),
            mode: ScanMode::Incremental,
            began: scan.began,
            files_changed: Some(UNCHANGED_FILES),
        },
    )?;
    conn.prepare_cached(sql::SCAN_LOG_TRIM)?
        .execute(params![column_int(scan.snapshot.get())?, keep])?;
    Ok(())
}

/// The retained scans of `project`, newest first, with the totals of the snapshot each names.
pub(crate) fn history(conn: &Connection, project: ProjectId) -> StorageResult<Vec<ScanEntry>> {
    let mut statement = conn.prepare_cached(sql::SCAN_HISTORY)?;
    let rows = statement.query_map(params![column_int(project.get())?], |row| {
        let cols = Cols::new(row);
        Ok(Stored {
            scan: cols.take()?,
            snapshot: cols.take()?,
            commit: cols.take()?,
            created: cols.take()?,
            mode: cols.take()?,
            totals: (cols.take()?, cols.take()?, cols.take()?),
            timing: (cols.take()?, cols.take()?),
        })
    })?;
    rows.map(|row| entry(row?)).collect()
}

/// A row of the history as stored.
struct Stored {
    scan: i64,
    snapshot: i64,
    commit: Option<String>,
    created: i64,
    mode: String,
    /// Files, symbols and references of the snapshot.
    totals: (i64, i64, i64),
    /// Duration in milliseconds and changed files.
    timing: (Option<i64>, Option<i64>),
}

fn entry(row: Stored) -> StorageResult<ScanEntry> {
    Ok(ScanEntry {
        scan: id_from(row.scan)?,
        snapshot: SnapshotId::new(id_from(row.snapshot)?),
        created: Timestamp(row.created),
        commit: row.commit.map(CommitHash::new).transpose()?,
        mode: label_from(&row.mode)?,
        files: small_from(row.totals.0)?,
        symbols: small_from(row.totals.1)?,
        duration_ms: row.timing.0.map(id_from).transpose()?,
        files_changed: row.timing.1.map(small_from).transpose()?,
    })
}
