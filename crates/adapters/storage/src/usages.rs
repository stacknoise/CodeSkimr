//! The `UsageIndex` port and the graph queries, answered from the database (ADR 0089).
//!
//! No index is built when the program starts or when a project is opened: a question reads the
//! references it needs through the indexes of the database, so the first answer comes as fast as
//! the second and the memory stays what the answer needs.

use std::sync::{Arc, PoisonError};

use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{
    DependencyEdge, DependencyPage, DependencyQuery, ImpactPage, SymbolLabel, TypeFocus,
    UsageIndex, UsagePage,
};
use codeskimr_domain::{
    Direction, NamespaceGraph, QualifiedName, ReferenceKind, RelativePath, ReverseIndex,
    SnapshotId, SymbolId, UsageQuery,
};
use rusqlite::{Connection, params};

use crate::answers;
use crate::codec::{Cols, column_int, label_from, small_from};
use crate::consts::{KEPT_WALKS, PRAGMA_DATA_VERSION};
use crate::error::{StorageError, StorageResult};
use crate::sql;
use crate::store::{Build, Kept, SqliteStore};

impl SqliteStore {
    /// The `data_version` of the database: it changes when another connection commits, never for
    /// the commits of this one.
    pub(crate) fn data_version(&self) -> StorageResult<i64> {
        self.read(data_version)
    }

    fn usage_page(
        &self,
        snapshot: SnapshotId,
        query: &UsageQuery,
        limit: u32,
    ) -> StorageResult<UsagePage> {
        let raw = column_int(snapshot.get())?;
        let index = self.walked(snapshot, query)?;
        let (total, entries) =
            self.read(|conn| answers::usages(conn, raw, (&index, query), limit as usize))?;
        Ok(UsagePage { total, entries })
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
        let raw = column_int(snapshot.get())?;
        let index = self.walked(snapshot, &query)?;
        let (total, symbols) =
            self.read(|conn| answers::impact(conn, raw, (&index, &query), limit as usize))?;
        Ok(ImpactPage { total, symbols })
    }

    /// The references a walk of `query` touches: those of a kept walk that covers it, or read now
    /// and kept.
    fn walked(&self, snapshot: SnapshotId, query: &UsageQuery) -> StorageResult<Arc<ReverseIndex>> {
        let raw = column_int(snapshot.get())?;
        let version = self.data_version()?;
        if let Some(found) = self.kept_walk(snapshot, version, query) {
            return Ok(found);
        }
        let written = self.write_count();
        let index = Arc::new(self.read(|conn| answers::walked_references(conn, raw, query))?);
        let mut kept = self.walks.lock().unwrap_or_else(PoisonError::into_inner);
        if self.unwritten_since(written) {
            kept.insert(0, (snapshot, version, *query, Arc::clone(&index)));
            kept.truncate(KEPT_WALKS);
        }
        Ok(index)
    }

    /// A kept walk of `snapshot` read at `version` that covers `query`: the same symbol and
    /// direction, at least as many steps, and tests if `query` wants them.
    fn kept_walk(
        &self,
        snapshot: SnapshotId,
        version: i64,
        query: &UsageQuery,
    ) -> Option<Arc<ReverseIndex>> {
        let kept = self.walks.lock().unwrap_or_else(PoisonError::into_inner);
        kept.iter()
            .find(|(id, built_at, held, _)| {
                *id == snapshot
                    && *built_at == version
                    && held.symbol == query.symbol
                    && held.direction == query.direction
                    && held.depth >= query.depth
                    && (held.include_tests || !query.include_tests)
            })
            .map(|(.., index)| Arc::clone(index))
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
    /// What `build` makes of a snapshot, kept in `kept` as long as no write came in: a second
    /// caller for the same build waits for the first and gets its result. The build reads on a
    /// connection of its own.
    fn built<T: Clone>(
        &self,
        (kept, what): (&Kept<T>, Build),
        snapshot: SnapshotId,
        build: impl FnOnce(&Connection, i64) -> StorageResult<T>,
    ) -> StorageResult<T> {
        let raw = column_int(snapshot.get())?;
        let version = self.data_version()?;
        if let Some(found) = kept.get(snapshot, version) {
            return Ok(found);
        }
        let gate = self.build_gate(raw, what);
        let _building = gate.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(found) = kept.get(snapshot, version) {
            return Ok(found);
        }
        let written = self.write_count();
        let value = self.read_apart(|conn| build(conn, raw))?;
        kept.put((snapshot, version), &value, || {
            self.unwritten_since(written)
        });
        Ok(value)
    }

    /// The namespaces of a snapshot and the references between them. A snapshot that is not
    /// stored has none. The graph is kept for the snapshot; it is small, while reading it takes a
    /// pass over the references of a large project.
    ///
    /// # Errors
    /// A coded `STO` failure.
    pub fn namespace_graph(&self, snapshot: SnapshotId) -> Result<NamespaceGraph, CodedError> {
        self.built((&self.graphs, Build::Graph), snapshot, answers::namespaces)
            .map_err(CodedError::from)
    }

    /// The types around one type of a snapshot; `None` if the symbol is not a type of it. The
    /// owning types of all symbols are built once per snapshot and kept; a click reads the
    /// references that touch the members of the type.
    ///
    /// # Errors
    /// A coded `STO` failure.
    pub fn type_focus(
        &self,
        snapshot: SnapshotId,
        symbol: SymbolId,
    ) -> Result<Option<TypeFocus>, CodedError> {
        let raw = column_int(snapshot.get()).map_err(CodedError::from)?;
        self.built(
            (&self.owners, Build::Owners),
            snapshot,
            answers::type_owners,
        )
        .and_then(|owners| self.read(|conn| answers::type_focus(conn, raw, &owners, symbol)))
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

/// The `data_version` of the database: it changes when another connection commits, never for the
/// commits of this one.
fn data_version(conn: &Connection) -> StorageResult<i64> {
    Ok(conn.pragma_query_value(None, PRAGMA_DATA_VERSION, |row| row.get(0))?)
}
