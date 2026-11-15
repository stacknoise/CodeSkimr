//! Level 2: replaces the references of some files of the newest snapshot (ADR 0033).

use std::collections::{BTreeMap, BTreeSet, HashSet};

use codeskimr_application::ports::ReferenceReplacement;
use codeskimr_domain::{FileId, ProjectId, Snapshot, SnapshotId, coupling};
use rusqlite::{Connection, params};

use crate::codec::{column_int, id_from};
use crate::error::{StorageError, StorageResult};
use crate::headers::{insert_metric, insert_reference, latest_snapshot, set_totals, totals};
use crate::previous::load_latest_index;
use crate::projects::load_layers;
use crate::read_models;
use crate::references::id_set;
use crate::sql;

/// Whether `id` is in `ids`; an id beyond the column range is in none.
fn contains(ids: &HashSet<i64>, id: u64) -> bool {
    column_int(id).is_ok_and(|value| ids.contains(&value))
}

/// Fails with `DanglingReference` unless every file is in the snapshot and every reference stands
/// in one of those files and joins two symbols of the snapshot. The files and symbols of the
/// snapshot are read once, so the check costs two queries however many references there are.
fn check(conn: &Connection, snapshot: i64, change: &ReferenceReplacement) -> StorageResult<()> {
    let held_files = id_set(conn, sql::FILE_IDS, snapshot)?;
    let held_symbols = id_set(conn, sql::SYMBOL_IDS, snapshot)?;
    let files: HashSet<u64> = change.files.iter().map(|file| file.get()).collect();
    let ok = files.iter().all(|file| contains(&held_files, *file))
        && change.references.iter().all(|reference| {
            files.contains(&reference.file.get())
                && contains(&held_symbols, reference.from.get())
                && contains(&held_symbols, reference.to.get())
        });
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
    refresh_coupling(conn, change.snapshot, project)?;
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

/// Builds the read models of `snapshot` again and writes its totals.
fn refresh(conn: &Connection, snapshot: SnapshotId, project: ProjectId) -> StorageResult<()> {
    let id = column_int(snapshot.get())?;
    for delete in sql::READ_MODELS_DELETE {
        conn.execute(delete, params![id])?;
    }
    read_models::rebuild(conn, snapshot, &load_layers(conn, project)?)?;
    // The totals count the references from the usage counts, so the read models come first.
    set_totals(conn, snapshot, totals(conn, snapshot)?)
}

/// Calculates the coupling of the newest snapshot again from its stored references, because the
/// replaced ones change which units depend on which (ADR 0037). Older snapshots keep theirs.
fn refresh_coupling(
    conn: &Connection,
    snapshot: SnapshotId,
    project: ProjectId,
) -> StorageResult<()> {
    let id = column_int(snapshot.get())?;
    let index = load_latest_index(conn, project)?;
    let metrics = coupling(&index.symbols, &index.references)?;
    conn.execute(sql::COUPLING_DELETE, params![id])?;
    for metric in &metrics {
        insert_metric(conn, id, metric)?;
    }
    Ok(())
}
