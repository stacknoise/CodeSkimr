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
use crate::consts::{KEPT_LOOKUPS, PRAGMA_DATA_VERSION};
use crate::error::{StorageError, StorageResult};
use crate::lookup::SnapshotLookup;
use crate::sql;
use crate::store::{Build, BuildGate, SqliteStore};

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
        let version = self.read(data_version)?;
        if let Some(found) = self.kept(snapshot, version) {
            return Ok(found);
        }
        let gate = self.build_gate(raw, Build::Index);
        let _building = gate.lock().unwrap_or_else(PoisonError::into_inner);
        // Another caller may have built it while this one waited for the gate.
        if let Some(found) = self.kept(snapshot, version) {
            return Ok(found);
        }
        let loaded = Arc::new(self.read_apart(|conn| SnapshotLookup::load(conn, raw))?);
        self.keep(snapshot, (&loaded, version));
        Ok(loaded)
    }

    /// The kept index of `snapshot` if it was built at `version`; it becomes the newest.
    fn kept(&self, snapshot: SnapshotId, version: i64) -> Option<Arc<SnapshotLookup>> {
        let mut kept = self.reverse.lock().unwrap_or_else(PoisonError::into_inner);
        let at = kept
            .iter()
            .position(|(id, _, built_at)| *id == snapshot && *built_at == version)?;
        let entry = kept.remove(at);
        let found = Arc::clone(&entry.1);
        kept.insert(0, entry);
        Some(found)
    }

    /// Keeps the index of `snapshot` as the newest, instead of any older one of the same
    /// snapshot; the oldest ones beyond `KEPT_LOOKUPS` go.
    fn keep(&self, snapshot: SnapshotId, (loaded, version): (&Arc<SnapshotLookup>, i64)) {
        let mut kept = self.reverse.lock().unwrap_or_else(PoisonError::into_inner);
        kept.retain(|(id, _, _)| *id != snapshot);
        kept.insert(0, (snapshot, Arc::clone(loaded), version));
        kept.truncate(KEPT_LOOKUPS);
    }

    /// The lock that serialises the builds of one snapshot.
    pub(crate) fn build_gate(&self, snapshot: i64, what: Build) -> BuildGate {
        let mut gates = self.builds.lock().unwrap_or_else(PoisonError::into_inner);
        // A gate nobody holds is dropped, so the map does not grow with every snapshot.
        gates.retain(|_, gate| Arc::strong_count(gate) > 1);
        Arc::clone(gates.entry((snapshot, what)).or_default())
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

    /// The snapshot whose index was asked about last and is kept in memory, if any.
    #[must_use]
    pub fn kept_snapshot(&self) -> Option<SnapshotId> {
        let kept = self.reverse.lock().unwrap_or_else(PoisonError::into_inner);
        kept.first().map(|(id, _, _)| *id)
    }

    /// Replaces the kept index of the project of `snapshot` after a write: the references of the
    /// snapshot may have changed (level 2 replaces them), or a newer snapshot of the same project
    /// has taken the place of the one that was asked about. Only a store that has answered a
    /// query for that project builds it again, so that a scan nobody queries pays nothing, and
    /// the index of another project stays. A failure drops the replaced indexes, to be built by
    /// the next query.
    pub(crate) fn refresh_lookup(&self, snapshot: SnapshotId) {
        let Ok(raw) = column_int(snapshot.get()) else {
            return;
        };
        // The gate first: a build that is still running for this snapshot has read the references
        // before this write, so it is waited for, found in the list below and built again.
        let gate = self.build_gate(raw, Build::Index);
        let _building = gate.lock().unwrap_or_else(PoisonError::into_inner);
        self.facts
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clear();
        let replaced = self.kept_of_project_of(raw);
        if replaced.is_empty() {
            return;
        }
        let loaded = self.read(data_version).and_then(|version| {
            self.read_apart(|conn| SnapshotLookup::load(conn, raw))
                .map(|loaded| (Arc::new(loaded), version))
        });
        let mut kept = self.reverse.lock().unwrap_or_else(PoisonError::into_inner);
        kept.retain(|(id, _, _)| !replaced.contains(id) && *id != snapshot);
        if let Ok((loaded, version)) = loaded {
            kept.insert(0, (snapshot, loaded, version));
            kept.truncate(KEPT_LOOKUPS);
        }
    }

    /// The kept snapshots that belong to the project of snapshot `raw`, or to no known project.
    fn kept_of_project_of(&self, raw: i64) -> Vec<SnapshotId> {
        let ids: Vec<SnapshotId> = {
            let kept = self.reverse.lock().unwrap_or_else(PoisonError::into_inner);
            kept.iter().map(|(id, _, _)| *id).collect()
        };
        self.read(|conn| {
            let own = project_of(conn, raw)?;
            Ok(ids
                .into_iter()
                .filter(|id| {
                    let other = column_int(id.get())
                        .ok()
                        .and_then(|other| project_of(conn, other).ok().flatten());
                    other.is_none() || other == own
                })
                .collect())
        })
        .unwrap_or_default()
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

/// The `data_version` of the database: it changes when another connection commits, never for the
/// commits of this one.
fn data_version(conn: &Connection) -> StorageResult<i64> {
    Ok(conn.pragma_query_value(None, PRAGMA_DATA_VERSION, |row| row.get(0))?)
}
