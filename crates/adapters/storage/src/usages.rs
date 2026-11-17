//! The reverse index of a snapshot and the `UsageIndex` port.
//!
//! The references of a snapshot are loaded once into a `ReverseIndex` and kept for the snapshot
//! that was asked about last. A scan writes a new snapshot with a new id, but the level 2 import
//! (ADR 0033) replaces the references of the newest one, so the kept index is built again after
//! every write that touches it (`refresh_lookup`). The symbols and files that name the entries of
//! an answer are loaded with it, so an answer is assembled in memory.

use std::sync::{Arc, PoisonError};

use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{
    DependencyEdge, DependencyPage, DependencyQuery, ImpactPage, SymbolLabel, TypeFocus,
    UsageIndex, UsagePage,
};
use codeskimr_domain::{
    Direction, NamespaceGraph, QualifiedName, ReferenceKind, RelativePath, SnapshotId, SymbolId,
    UsageQuery,
};
use rusqlite::{Connection, OptionalExtension, params};

use crate::codec::{Cols, column_int, id_from, label_from, small_from};
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

    /// Builds the index of `snapshot` ahead of the first query, which would otherwise wait for it
    /// (#333).
    ///
    /// # Errors
    /// A coded `STO` failure.
    pub fn warm(&self, snapshot: SnapshotId) -> Result<(), CodedError> {
        self.lookup(snapshot).map(|_| ()).map_err(CodedError::from)
    }

    /// Builds the index of the newest snapshot of any project, if there is one. A snapshot that
    /// is written between reading its id and the load is not the one that is kept; the first
    /// query then builds the index of the newer one.
    ///
    /// # Errors
    /// A coded `STO` failure.
    pub fn warm_newest(&self) -> Result<(), CodedError> {
        let newest: Option<i64> = self
            .read(|conn| Ok(conn.query_row(sql::SNAPSHOT_NEWEST, [], |row| row.get(0))?))
            .map_err(CodedError::from)?;
        match newest {
            Some(id) => self.warm(SnapshotId::new(id_from(id).map_err(CodedError::from)?)),
            None => Ok(()),
        }
    }

    /// The snapshot whose index is kept in memory, if any.
    #[must_use]
    pub fn kept_snapshot(&self) -> Option<SnapshotId> {
        let cached = self.reverse.lock().unwrap_or_else(PoisonError::into_inner);
        cached.as_ref().map(|(id, _)| *id)
    }

    /// Replaces the kept index after a write: the references of the snapshot may have changed
    /// (level 2 replaces them), or a newer snapshot of the same project has taken the place of
    /// the one that was asked about. Only a store that has answered a query builds it again, so
    /// that a scan nobody queries pays nothing, and the index of another project stays. Queries
    /// of this store wait for the build, as they would for the load of the first query. A failure
    /// drops the index, to be built by the next query.
    pub(crate) fn refresh_lookup(&self, snapshot: SnapshotId) {
        let mut cached = self.reverse.lock().unwrap_or_else(PoisonError::into_inner);
        let Some((kept, _)) = cached.as_ref() else {
            return;
        };
        let (kept, raw) = (*kept, column_int(snapshot.get()));
        let loaded = raw.and_then(|raw| {
            self.read(|conn| {
                let other = project_of(conn, column_int(kept.get())?)?;
                if other.is_some() && other != project_of(conn, raw)? {
                    return Ok(None);
                }
                SnapshotLookup::load(conn, raw).map(Some)
            })
        });
        match loaded {
            Ok(Some(loaded)) => *cached = Some((snapshot, Arc::new(loaded))),
            Ok(None) => {}
            Err(_) => *cached = None,
        }
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

/// The project of a snapshot, `None` if the snapshot is not stored.
fn project_of(conn: &Connection, snapshot: i64) -> StorageResult<Option<i64>> {
    Ok(conn
        .query_row(sql::SNAPSHOT_PROJECT, params![snapshot], |row| row.get(0))
        .optional()?)
}

/// The columns of `DEPENDENCIES_*`, still raw.
struct EdgeRow {
    id: i64,
    qualified: String,
    kind: String,
    path: String,
    count: i64,
}

// codeskimr: allow CHK-102 maps a row of the dependency query into its columns
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

impl SqliteStore {
    /// The types around one type of a snapshot; `None` if the symbol is not a type of it.
    ///
    /// # Errors
    /// A coded `STO` failure.
    pub fn type_focus(
        &self,
        snapshot: SnapshotId,
        symbol: SymbolId,
    ) -> Result<Option<TypeFocus>, CodedError> {
        self.lookup(snapshot)
            .and_then(|found| found.type_focus(symbol))
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
