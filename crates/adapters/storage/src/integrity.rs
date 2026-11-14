//! What the foreign keys checked before the rows were shared between snapshots (ADR 0032): every
//! new row points at rows that its snapshot holds. A violation rolls the snapshot back, so no
//! orphan row stays behind that the prune could not find.

use std::collections::HashSet;

use codeskimr_application::ports::SnapshotData;
use rusqlite::{Connection, params};

use crate::codec::column_int;
use crate::error::{StorageError, StorageResult};
use crate::similarity::json_list;
use crate::sql;

/// The ids the new data brings itself; everything else is looked up in the carried rows.
struct Known {
    snapshot: i64,
    files: HashSet<u64>,
    symbols: HashSet<u64>,
}

impl Known {
    fn of(data: &SnapshotData) -> StorageResult<Self> {
        Ok(Self {
            snapshot: column_int(data.snapshot.id.get())?,
            files: data.files.iter().map(|file| file.id.get()).collect(),
            symbols: data.symbols.iter().map(|symbol| symbol.id.get()).collect(),
        })
    }

    fn has_file(&self, conn: &Connection, file: u64) -> StorageResult<bool> {
        self.has(
            conn,
            sql::FILE_IN_SNAPSHOT,
            self.files.contains(&file),
            file,
        )
    }

    fn has_symbol(&self, conn: &Connection, symbol: u64) -> StorageResult<bool> {
        self.has(
            conn,
            sql::SYMBOL_IN_SNAPSHOT,
            self.symbols.contains(&symbol),
            symbol,
        )
    }

    /// Whether every symbol of `ids` is new or in the carried rows; the carried ones are looked up
    /// together, because coupling brings a metric for every namespace of the snapshot.
    fn have_symbols(&self, conn: &Connection, ids: &[u64]) -> StorageResult<bool> {
        let carried: HashSet<u64> = ids
            .iter()
            .copied()
            .filter(|id| !self.symbols.contains(id))
            .collect();
        let mut ids = Vec::with_capacity(carried.len());
        for id in &carried {
            ids.push(column_int(*id)?);
        }
        let list = json_list(ids.into_iter());
        let found: i64 = conn
            .prepare_cached(sql::SYMBOLS_IN_SNAPSHOT_COUNT)?
            .query_row(params![self.snapshot, list], |row| row.get(0))?;
        Ok(usize::try_from(found).is_ok_and(|found| found == carried.len()))
    }

    fn has(&self, conn: &Connection, query: &str, new: bool, id: u64) -> StorageResult<bool> {
        if new {
            return Ok(true);
        }
        let found: bool = conn
            .prepare_cached(query)?
            .query_row(params![self.snapshot, column_int(id)?], |row| row.get(0))?;
        Ok(found)
    }
}

/// Fails with `DanglingReference` if a new symbol, reference, metric or fingerprint points at a
/// file or symbol that the snapshot does not hold. Run after the carried rows are in place.
pub(crate) fn check(conn: &Connection, data: &SnapshotData) -> StorageResult<()> {
    let known = Known::of(data)?;
    let mut ok = true;
    for symbol in &data.symbols {
        ok &= known.has_file(conn, symbol.file.get())?;
        if let Some(parent) = symbol.parent {
            ok &= known.has_symbol(conn, parent.get())?;
        }
    }
    for reference in &data.references {
        ok &= known.has_file(conn, reference.file.get())?
            && known.has_symbol(conn, reference.from.get())?
            && known.has_symbol(conn, reference.to.get())?;
    }
    let measured: Vec<u64> = data.metrics.iter().map(|m| m.symbol.get()).collect();
    ok &= known.have_symbols(conn, &measured)?;
    for found in &data.fingerprints {
        ok &= known.has_symbol(conn, found.symbol.get())?;
    }
    if ok {
        Ok(())
    } else {
        Err(StorageError::DanglingReference)
    }
}
