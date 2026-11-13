//! The reverse index of a snapshot and the `UsageIndex` port.
//!
//! The references of a snapshot are loaded once into a `ReverseIndex` and kept for the snapshot
//! that was asked about last; a snapshot never changes, a new scan gets a new id. Only the
//! entries of the answer are looked up in the database.

use std::sync::{Arc, PoisonError};

use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{
    DependencyEdge, DependencyPage, DependencyQuery, ImpactPage, ImpactedSymbol, SymbolLabel,
    UsageEntry, UsageIndex, UsagePage,
};
use codeskimr_domain::{
    Direction, QualifiedName, ReferenceKind, RelativePath, ReverseIndex, SnapshotId, SymbolId,
    UsageQuery, Visibility,
};
use rusqlite::{Connection, OptionalExtension, params};

use crate::codec::{Cols, column_int, label_from, small_from};
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

/// The columns of `SYMBOL_LABEL`, still raw.
struct LabelRow {
    qualified: String,
    kind: String,
    visibility: Option<String>,
    path: String,
}

fn read_label(cols: &Cols<'_>) -> rusqlite::Result<LabelRow> {
    Ok(LabelRow {
        qualified: cols.take()?,
        kind: cols.take()?,
        visibility: cols.take()?,
        path: cols.take()?,
    })
}

fn placed(conn: &Connection, snapshot: i64, symbol: SymbolId) -> StorageResult<Placed> {
    let row = conn
        .prepare_cached(sql::SYMBOL_LABEL)?
        .query_row(params![snapshot, column_int(symbol.get())?], |row| {
            read_label(&Cols::new(row))
        })
        .optional()?
        .ok_or(StorageError::DanglingReference)?;
    Ok(Placed {
        label: SymbolLabel {
            id: symbol,
            qualified_name: QualifiedName::new(row.qualified)?,
            kind: label_from(&row.kind)?,
        },
        visibility: row.visibility.as_deref().map(label_from).transpose()?,
        path: RelativePath::new(row.path)?,
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
        verified: reached.verified,
        visibility: found.visibility,
        path: found.path,
    })
}

impl SqliteStore {
    /// The reverse index of a snapshot, built on first use. A snapshot that is not stored (never
    /// existed, or pruned by the retention) has an empty index that is not kept; like the other
    /// queries, usages of such a snapshot are empty.
    fn reverse_index(&self, snapshot: SnapshotId) -> StorageResult<Arc<ReverseIndex>> {
        let raw = column_int(snapshot.get())?;
        let stored: i64 = self.read(|conn| {
            Ok(conn.query_row(sql::SNAPSHOT_EXISTS, params![raw], |row| row.get(0))?)
        })?;
        if stored == 0 {
            return Ok(Arc::new(ReverseIndex::default()));
        }
        let mut cached = self.reverse.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some((id, index)) = cached.as_ref()
            && *id == snapshot
        {
            return Ok(Arc::clone(index));
        }
        let references = self.read(|conn| load_references(conn, raw))?;
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

/// The columns of `DEPENDENCIES_*`, still raw.
struct EdgeRow {
    id: i64,
    qualified: String,
    kind: String,
    path: String,
    count: i64,
}

fn read_edge(cols: &Cols<'_>) -> rusqlite::Result<EdgeRow> {
    Ok(EdgeRow {
        id: cols.take()?,
        qualified: cols.take()?,
        kind: cols.take()?,
        path: cols.take()?,
        count: cols.take()?,
    })
}

fn dependency_edges(
    conn: &Connection,
    snapshot: i64,
    query: &DependencyQuery,
) -> StorageResult<Vec<DependencyEdge>> {
    let sql = match query.direction {
        Direction::Outgoing => sql::DEPENDENCIES_OUTGOING,
        Direction::Incoming => sql::DEPENDENCIES_INCOMING,
    };
    let mut statement = conn.prepare_cached(sql)?;
    let rows = statement.query_map(
        params![
            snapshot,
            column_int(query.symbol.get())?,
            query.include_tests,
            ReferenceKind::Test.label()
        ],
        |row| read_edge(&Cols::new(row)),
    )?;
    let mut edges = Vec::new();
    for row in rows {
        let row = row?;
        edges.push(DependencyEdge {
            other: SymbolLabel {
                id: SymbolId::new(u64::try_from(row.id).map_err(|_| StorageError::OutOfRange)?),
                qualified_name: QualifiedName::new(row.qualified)?,
                kind: label_from(&row.kind)?,
            },
            path: RelativePath::new(row.path)?,
            count: small_from(row.count)?,
        });
    }
    Ok(edges)
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

    fn dependencies(
        &self,
        snapshot: SnapshotId,
        query: &DependencyQuery,
        limit: u32,
    ) -> Result<DependencyPage, CodedError> {
        let raw = column_int(snapshot.get()).map_err(CodedError::from)?;
        let mut edges = self
            .read(|conn| dependency_edges(conn, raw, query))
            .map_err(CodedError::from)?;
        let total = edges.len();
        edges.truncate(limit as usize);
        Ok(DependencyPage { total, edges })
    }
}
