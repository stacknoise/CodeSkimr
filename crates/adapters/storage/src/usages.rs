//! The reverse index of a snapshot and the `UsageIndex` port.
//!
//! The references of a snapshot are loaded once into a `ReverseIndex` and kept for the snapshot
//! that was asked about last; a snapshot never changes, a new scan gets a new id. The
//! symbols and files that name the entries of an answer are loaded with it, so an answer is
//! assembled in memory.

use std::sync::{Arc, PoisonError};

use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{
    DependencyEdge, DependencyPage, DependencyQuery, ImpactPage, SymbolLabel, UsageIndex, UsagePage,
};
use codeskimr_domain::{
    Direction, NamespaceGraph, QualifiedName, ReferenceKind, RelativePath, SnapshotId, SymbolId,
    UsageQuery,
};
use rusqlite::{Connection, params};

use crate::codec::{Cols, column_int, label_from, small_from};
use crate::error::{StorageError, StorageResult};
use crate::lookup::SnapshotLookup;
use crate::sql;
use crate::store::SqliteStore;

impl SqliteStore {
    /// The reverse index of a snapshot, built on first use. A snapshot that is not stored (never
    /// existed, or pruned by the retention) has an empty index that is not kept; like the other
    /// queries, usages of such a snapshot are empty.
    pub(crate) fn lookup(&self, snapshot: SnapshotId) -> StorageResult<Arc<SnapshotLookup>> {
        let raw = column_int(snapshot.get())?;
        let stored: i64 = self.read(|conn| {
            Ok(conn.query_row(sql::SNAPSHOT_EXISTS, params![raw], |row| row.get(0))?)
        })?;
        if stored == 0 {
            return Ok(Arc::new(SnapshotLookup::default()));
        }
        let mut cached = self.reverse.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some((id, index)) = cached.as_ref()
            && *id == snapshot
        {
            return Ok(Arc::clone(index));
        }
        let loaded = Arc::new(self.read(|conn| SnapshotLookup::load(conn, raw))?);
        *cached = Some((snapshot, Arc::clone(&loaded)));
        Ok(loaded)
    }

    fn usage_page(
        &self,
        snapshot: SnapshotId,
        query: &UsageQuery,
        limit: u32,
    ) -> StorageResult<UsagePage> {
        let lookup = self.lookup(snapshot)?;
        let found = lookup.index.usages(query);
        let entries = found
            .iter()
            .take(limit as usize)
            .map(|usage| lookup.entry(usage))
            .collect::<StorageResult<Vec<_>>>()?;
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
        let lookup = self.lookup(snapshot)?;
        let reached = lookup.index.reached(&query);
        let symbols = reached
            .iter()
            .take(limit as usize)
            .map(|found| lookup.impacted(found))
            .collect::<StorageResult<Vec<_>>>()?;
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

impl SqliteStore {
    /// The namespaces of a snapshot and the references between them. A snapshot that is not
    /// stored has none.
    ///
    /// # Errors
    /// A coded `STO` failure.
    pub fn namespace_graph(&self, snapshot: SnapshotId) -> Result<NamespaceGraph, CodedError> {
        self.lookup(snapshot)
            .map(|found| found.namespaces.clone())
            .map_err(CodedError::from)
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
