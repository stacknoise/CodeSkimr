//! The reverse index of a snapshot and the `UsageIndex` port.
//!
//! The references of a snapshot are loaded once into a `ReverseIndex` and kept for the snapshot
//! that was asked about last; a snapshot never changes, a new scan gets a new id. Only the
//! entries of the answer are looked up in the database.

use std::sync::{Arc, PoisonError};

use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{
    ImpactPage, ImpactedSymbol, SymbolLabel, UsageEntry, UsageIndex, UsagePage,
};
use codeskimr_domain::{
    Direction, QualifiedName, RelativePath, ReverseIndex, SnapshotId, SymbolId, UsageQuery,
    Visibility,
};
use rusqlite::{Connection, OptionalExtension, params};

use crate::codec::{Cols, column_int, label_from};
use crate::error::{StorageError, StorageResult};
use crate::previous::load_references;
use crate::sql;
use crate::store::SqliteStore;

/// A symbol as it is stored: its label, visibility and file.
struct Placed {
    label: SymbolLabel,
    visibility: Option<Visibility>,
    path: RelativePath,
}

fn placed(conn: &Connection, snapshot: i64, symbol: SymbolId) -> StorageResult<Placed> {
    let row = conn
        .prepare_cached(sql::SYMBOL_LABEL)?
        .query_row(params![snapshot, column_int(symbol.get())?], |row| {
            let cols = Cols::new(row);
            Ok((
                cols.take::<String>()?,
                cols.take::<String>()?,
                cols.take::<Option<String>>()?,
                cols.take::<String>()?,
            ))
        })
        .optional()?;
    let (qualified, kind, visibility, path) = row.ok_or(StorageError::DanglingReference)?;
    Ok(Placed {
        label: SymbolLabel {
            id: symbol,
            qualified_name: QualifiedName::new(qualified)?,
            kind: label_from(&kind)?,
        },
        visibility: visibility.as_deref().map(label_from).transpose()?,
        path: RelativePath::new(path)?,
    })
}

fn file_path(conn: &Connection, snapshot: i64, file: u64) -> StorageResult<RelativePath> {
    let path: Option<String> = conn
        .prepare_cached(sql::FILE_PATH)?
        .query_row(params![snapshot, column_int(file)?], |row| row.get(0))
        .optional()?;
    Ok(RelativePath::new(
        path.ok_or(StorageError::DanglingReference)?,
    )?)
}

fn entry(
    conn: &Connection,
    snapshot: i64,
    usage: &codeskimr_domain::Usage,
) -> StorageResult<UsageEntry> {
    let reference = usage.reference;
    Ok(UsageEntry {
        depth: usage.depth,
        reference,
        path: file_path(conn, snapshot, reference.file.get())?,
        from: placed(conn, snapshot, reference.from)?.label,
        to: placed(conn, snapshot, reference.to)?.label,
    })
}

fn impacted(
    conn: &Connection,
    snapshot: i64,
    reached: &codeskimr_domain::Reached,
) -> StorageResult<ImpactedSymbol> {
    let found = placed(conn, snapshot, reached.symbol)?;
    Ok(ImpactedSymbol {
        label: found.label,
        depth: reached.depth,
        visibility: found.visibility,
        path: found.path,
    })
}

impl SqliteStore {
    /// The reverse index of a snapshot, built on first use.
    fn reverse_index(&self, snapshot: SnapshotId) -> StorageResult<Arc<ReverseIndex>> {
        let mut cached = self.reverse.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some((id, index)) = cached.as_ref()
            && *id == snapshot
        {
            return Ok(Arc::clone(index));
        }
        let references = self.read(|conn| load_references(conn, column_int(snapshot.get())?))?;
        let index = Arc::new(ReverseIndex::new(references));
        *cached = Some((snapshot, Arc::clone(&index)));
        Ok(index)
    }

    fn usage_page(
        &self,
        snapshot: SnapshotId,
        query: &UsageQuery,
        limit: u32,
    ) -> StorageResult<UsagePage> {
        let found = self.reverse_index(snapshot)?.usages(query);
        let raw = column_int(snapshot.get())?;
        let entries = self.read(|conn| {
            found
                .iter()
                .take(limit as usize)
                .map(|usage| entry(conn, raw, usage))
                .collect::<StorageResult<Vec<_>>>()
        })?;
        Ok(UsagePage {
            total: found.len(),
            entries,
        })
    }

    fn impact_page(
        &self,
        snapshot: SnapshotId,
        symbol: SymbolId,
        depth: u32,
        limit: u32,
    ) -> StorageResult<ImpactPage> {
        let query = UsageQuery {
            symbol,
            direction: Direction::Incoming,
            depth,
            include_tests: true,
        };
        let reached = self.reverse_index(snapshot)?.reached(&query);
        let raw = column_int(snapshot.get())?;
        let symbols = self.read(|conn| {
            reached
                .iter()
                .take(limit as usize)
                .map(|found| impacted(conn, raw, found))
                .collect::<StorageResult<Vec<_>>>()
        })?;
        Ok(ImpactPage {
            total: reached.len(),
            symbols,
        })
    }
}

impl UsageIndex for SqliteStore {
    fn usages(
        &self,
        snapshot: SnapshotId,
        query: &UsageQuery,
        limit: u32,
    ) -> Result<UsagePage, CodedError> {
        self.usage_page(snapshot, query, limit)
            .map_err(CodedError::from)
    }

    fn impact(
        &self,
        snapshot: SnapshotId,
        symbol: SymbolId,
        depth: u32,
        limit: u32,
    ) -> Result<ImpactPage, CodedError> {
        self.impact_page(snapshot, symbol, depth, limit)
            .map_err(CodedError::from)
    }
}
