//! Level 2: replaces the references and the symbols of origin `index` of some files of the newest
//! snapshot (ADR 0033, ADR 0055).

use std::collections::{BTreeMap, BTreeSet, HashSet};

use codeskimr_application::ports::Level2Replacement;
use codeskimr_domain::{FileId, ProjectId, Snapshot, SnapshotId, Symbol, SymbolOrigin, coupling};
use rusqlite::{Connection, params};

use crate::codec::{column_int, id_from};
use crate::error::{StorageError, StorageResult};
use crate::headers::{
    insert_metric, insert_reference, insert_symbol, latest_snapshot, set_totals, totals,
};
use crate::previous::load_latest_index;
use crate::projects::load_layers;
use crate::read_models;
use crate::references::id_set;
use crate::sql;

/// Whether `id` is in `ids`; an id beyond the column range is in none.
fn contains(ids: &HashSet<i64>, id: u64) -> bool {
    column_int(id).is_ok_and(|value| ids.contains(&value))
}

/// Fails with `DanglingReference` unless every file is in the snapshot. The files of the snapshot
/// are read once, so the check costs one query however many files there are.
fn check_files(conn: &Connection, snapshot: i64, change: &Level2Replacement) -> StorageResult<()> {
    let held = id_set(conn, sql::FILE_IDS, snapshot)?;
    if change.files.iter().all(|file| contains(&held, file.get())) {
        Ok(())
    } else {
        Err(StorageError::DanglingReference)
    }
}

/// Fails with `DanglingReference` unless every new symbol is of origin `index` and stands in one
/// of the files, and every parent is a stored symbol of the snapshot or a new one of the same
/// file. `held` are the symbols of the snapshot, read after the new ones are inserted.
fn check_symbols(held: &HashSet<i64>, change: &Level2Replacement) -> StorageResult<()> {
    let files: HashSet<FileId> = change.files.iter().copied().collect();
    let files_of: BTreeMap<_, _> = change.symbols.iter().map(|s| (s.id, s.file)).collect();
    let ok = change.symbols.iter().all(|symbol| {
        symbol.origin == SymbolOrigin::Index
            && files.contains(&symbol.file)
            && symbol
                .parent
                .is_none_or(|parent| match files_of.get(&parent) {
                    Some(file) => *file == symbol.file,
                    None => contains(held, parent.get()),
                })
    });
    if ok {
        Ok(())
    } else {
        Err(StorageError::DanglingReference)
    }
}

/// Fails with `DanglingReference` unless every reference stands in one of the files and joins two
/// symbols of the snapshot (`held`, read after the new symbols are inserted).
fn check_references(held: &HashSet<i64>, change: &Level2Replacement) -> StorageResult<()> {
    let files: HashSet<u64> = change.files.iter().map(|file| file.get()).collect();
    let ok = change.references.iter().all(|reference| {
        files.contains(&reference.file.get())
            && contains(held, reference.from.get())
            && contains(held, reference.to.get())
    });
    if ok {
        Ok(())
    } else {
        Err(StorageError::DanglingReference)
    }
}

/// Removes the symbols of origin `index` of the files, and with them every reference that points
/// at one or starts in one (also from another file: its target is gone), and the level 1
/// references of the files.
fn clear(conn: &Connection, files: &[FileId]) -> StorageResult<()> {
    for file in files {
        let id = column_int(file.get())?;
        conn.execute(sql::REFERENCES_OF_FILE_DELETE, params![id])?;
        let origin = SymbolOrigin::Index.label();
        conn.execute(sql::ORIGIN_REFERENCES_DELETE, params![id, origin])?;
        conn.execute(sql::ORIGIN_SYMBOLS_DELETE, params![id, origin])?;
    }
    Ok(())
}

fn insert_symbols(conn: &Connection, symbols: &[Symbol]) -> StorageResult<()> {
    symbols
        .iter()
        .try_for_each(|symbol| insert_symbol(conn, symbol, None))
}

fn project_of(conn: &Connection, snapshot: i64) -> StorageResult<ProjectId> {
    let id: i64 = conn
        .query_row(sql::SNAPSHOT_PROJECT, params![snapshot], |row| row.get(0))
        .map_err(|_| StorageError::DanglingReference)?;
    Ok(ProjectId::new(id_from(id)?))
}

/// Replaces the symbols of origin `index` and the references of the files in the newest snapshot of its project; any other snapshot
/// is refused, because its files may be shared with newer ones.
pub(crate) fn replace_level2(
    conn: &Connection,
    change: &Level2Replacement,
) -> StorageResult<Snapshot> {
    let snapshot = column_int(change.snapshot.get())?;
    let project = project_of(conn, snapshot)?;
    let latest = latest_snapshot(conn, project)?.ok_or(StorageError::DanglingReference)?;
    if latest.id != change.snapshot {
        return Err(StorageError::DanglingReference);
    }
    check_files(conn, snapshot, change)?;
    clear(conn, &change.files)?;
    insert_symbols(conn, &change.symbols)?;
    let held = id_set(conn, sql::SYMBOL_IDS, snapshot)?;
    check_symbols(&held, change)?;
    check_references(&held, change)?;
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

fn holders(conn: &Connection, change: &Level2Replacement) -> StorageResult<Holders> {
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
