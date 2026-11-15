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

/// The ids the new data brings itself and the ids it points at; what is not new is looked up in
/// the carried rows, all ids of a kind in one statement.
struct Known {
    snapshot: i64,
    files: HashSet<u64>,
    symbols: HashSet<u64>,
}

/// The files and symbols that the new rows point at, without the ones the data brings itself.
#[derive(Default)]
struct Wanted {
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

    fn want_file(&self, wanted: &mut Wanted, file: u64) {
        if !self.files.contains(&file) {
            wanted.files.insert(file);
        }
    }

    fn want_symbol(&self, wanted: &mut Wanted, symbol: u64) {
        if !self.symbols.contains(&symbol) {
            wanted.symbols.insert(symbol);
        }
    }

    /// Whether every wanted id is in the carried rows of the snapshot.
    fn all_carried(&self, conn: &Connection, wanted: &Wanted) -> StorageResult<bool> {
        Ok(
            self.count_matches(conn, sql::FILES_IN_SNAPSHOT_COUNT, &wanted.files)?
                && self.count_matches(conn, sql::SYMBOLS_IN_SNAPSHOT_COUNT, &wanted.symbols)?,
        )
    }

    /// Whether `count_query` finds all of `ids`: snapshot and a JSON list of the ids.
    fn count_matches(
        &self,
        conn: &Connection,
        count_query: &str,
        ids: &HashSet<u64>,
    ) -> StorageResult<bool> {
        let mut columns = Vec::with_capacity(ids.len());
        for id in ids {
            columns.push(column_int(*id)?);
        }
        let list = json_list(columns.into_iter());
        let found: i64 = conn
            .prepare_cached(count_query)?
            .query_row(params![self.snapshot, list], |row| row.get(0))?;
        Ok(usize::try_from(found).is_ok_and(|found| found == ids.len()))
    }
}

/// Fails with `DanglingReference` if a new symbol, reference, metric or fingerprint points at a
/// file or symbol that the snapshot does not hold. Run after the carried rows are in place.
pub(crate) fn check(conn: &Connection, data: &SnapshotData) -> StorageResult<()> {
    let known = Known::of(data)?;
    let mut wanted = Wanted::default();
    for symbol in &data.symbols {
        known.want_file(&mut wanted, symbol.file.get());
        if let Some(parent) = symbol.parent {
            known.want_symbol(&mut wanted, parent.get());
        }
    }
    for reference in &data.references {
        known.want_file(&mut wanted, reference.file.get());
        known.want_symbol(&mut wanted, reference.from.get());
        known.want_symbol(&mut wanted, reference.to.get());
    }
    for metric in &data.metrics {
        known.want_symbol(&mut wanted, metric.symbol.get());
    }
    for found in &data.fingerprints {
        known.want_symbol(&mut wanted, found.symbol.get());
    }
    if known.all_carried(conn, &wanted)? {
        Ok(())
    } else {
        Err(StorageError::DanglingReference)
    }
}
