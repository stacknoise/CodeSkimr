//! Queries on the read models.

use codeskimr_application::error::CodedError;
use codeskimr_application::ports::SearchHit;
use codeskimr_domain::{
    LayerEdge, LayerName, ProjectId, QualifiedName, RelativePath, Snapshot, SnapshotId, SymbolId,
    SymbolName,
};
use rusqlite::{Connection, OptionalExtension, params};

use crate::codec::{Cols, column_int, id_from, label_from, small_from};
use crate::consts::{LIKE_ANY, LIKE_ESCAPE, LIKE_ONE};
use crate::error::StorageResult;
use crate::sql;
use crate::store::SqliteStore;

/// The metric strip of a type: members and the metrics of the type and its members.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TypeSummary {
    /// Number of direct members.
    pub members: u32,
    /// Lines of code of the type, if measured.
    pub loc: Option<f64>,
    /// Highest cyclomatic complexity among the members, if measured.
    pub max_complexity: Option<f64>,
    /// Maintainability index of the type, if measured.
    pub maintainability: Option<f64>,
}

/// Escapes the wildcards of `LIKE` in `text`.
fn escape_like(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for character in text.chars() {
        if [LIKE_ESCAPE, LIKE_ANY, LIKE_ONE].contains(&character) {
            out.push(LIKE_ESCAPE);
        }
        out.push(character);
    }
    out
}

/// `LIKE` patterns for a query: every character in order (fuzzy), and the plain prefix.
fn patterns(query: &str) -> (String, String) {
    let mut fuzzy = String::from(LIKE_ANY);
    for character in query.chars() {
        fuzzy.push_str(&escape_like(&character.to_string()));
        fuzzy.push(LIKE_ANY);
    }
    (fuzzy, format!("{}{LIKE_ANY}", escape_like(query)))
}

fn search(
    conn: &Connection,
    snapshot: i64,
    query: &str,
    limit: u32,
) -> StorageResult<Vec<SearchHit>> {
    let query = query.to_ascii_lowercase();
    if query.is_empty() {
        return Ok(Vec::new());
    }
    let (fuzzy, prefix) = patterns(&query);
    let mut statement = conn.prepare_cached(sql::SEARCH)?;
    let rows = statement.query_map(
        params![snapshot, fuzzy, query, prefix, query, i64::from(limit)],
        |row| {
            let cols = Cols::new(row);
            Ok((
                cols.take::<i64>()?,
                cols.take::<String>()?,
                cols.take::<String>()?,
                cols.take::<String>()?,
                cols.take::<String>()?,
            ))
        },
    )?;
    let mut hits = Vec::new();
    for row in rows {
        let (symbol, name, qualified, kind, path) = row?;
        hits.push(SearchHit {
            symbol: SymbolId::new(id_from(symbol)?),
            name: SymbolName::new(name)?,
            qualified_name: QualifiedName::new(qualified)?,
            kind: label_from(&kind)?,
            path: RelativePath::new(path)?,
        });
    }
    Ok(hits)
}

fn layer_edges(conn: &Connection, snapshot: i64) -> StorageResult<Vec<LayerEdge>> {
    let mut statement = conn.prepare_cached(sql::EDGES)?;
    let rows = statement.query_map(params![snapshot], |row| {
        let cols = Cols::new(row);
        Ok((
            cols.take::<String>()?,
            cols.take::<String>()?,
            cols.take::<i64>()?,
            cols.take::<bool>()?,
            cols.take::<bool>()?,
        ))
    })?;
    let mut edges = Vec::new();
    for row in rows {
        let (from, to, count, violates_layering, in_cycle) = row?;
        edges.push(LayerEdge {
            from: LayerName::new(from)?,
            to: LayerName::new(to)?,
            count: small_from(count)?,
            violates_layering,
            in_cycle,
        });
    }
    Ok(edges)
}

fn type_summary(
    conn: &Connection,
    snapshot: i64,
    symbol: i64,
) -> StorageResult<Option<TypeSummary>> {
    let row = conn
        .query_row(sql::TYPE_SUMMARY, params![snapshot, symbol], |row| {
            let cols = Cols::new(row);
            Ok((
                cols.take::<i64>()?,
                cols.take::<Option<f64>>()?,
                cols.take::<Option<f64>>()?,
                cols.take::<Option<f64>>()?,
            ))
        })
        .optional()?;
    row.map(|(members, loc, max_complexity, maintainability)| {
        Ok(TypeSummary {
            members: small_from(members)?,
            loc,
            max_complexity,
            maintainability,
        })
    })
    .transpose()
}

impl SqliteStore {
    /// Number of references to a symbol, the "Usages" column; zero if nothing refers to it.
    ///
    /// # Errors
    /// A coded `STO` failure.
    pub fn usage_count(&self, snapshot: SnapshotId, symbol: SymbolId) -> Result<u32, CodedError> {
        self.read(|conn| {
            let found: Option<i64> = conn
                .query_row(
                    sql::USAGES_OF,
                    params![column_int(snapshot.get())?, column_int(symbol.get())?],
                    |row| row.get(0),
                )
                .optional()?;
            found.map_or(Ok(u32::default()), small_from)
        })
        .map_err(CodedError::from)
    }

    /// Fuzzy search over the symbol names of a snapshot: the characters of `query` must appear in
    /// the name in that order, case-insensitively. Exact matches come first, then prefix and
    /// substring matches, shorter names before longer ones. Parameters are not indexed.
    ///
    /// # Errors
    /// A coded `STO` failure.
    pub fn search(
        &self,
        snapshot: SnapshotId,
        query: &str,
        limit: u32,
    ) -> Result<Vec<SearchHit>, CodedError> {
        self.read(|conn| search(conn, column_int(snapshot.get())?, query, limit))
            .map_err(CodedError::from)
    }

    /// The dependency map of a snapshot: references aggregated per pair of layers.
    ///
    /// # Errors
    /// A coded `STO` failure.
    pub fn layer_edges(&self, snapshot: SnapshotId) -> Result<Vec<LayerEdge>, CodedError> {
        self.read(|conn| layer_edges(conn, column_int(snapshot.get())?))
            .map_err(CodedError::from)
    }

    /// The metric strip of a type, `None` for symbols that are not types.
    ///
    /// # Errors
    /// A coded `STO` failure.
    pub fn type_summary(
        &self,
        snapshot: SnapshotId,
        symbol: SymbolId,
    ) -> Result<Option<TypeSummary>, CodedError> {
        self.read(|conn| type_summary(conn, column_int(snapshot.get())?, column_int(symbol.get())?))
            .map_err(CodedError::from)
    }

    /// The newest snapshot of a project, `None` before the first scan.
    ///
    /// # Errors
    /// A coded `STO` failure.
    pub fn latest_snapshot(&self, project: ProjectId) -> Result<Option<Snapshot>, CodedError> {
        self.read(|conn| crate::snapshots::latest_snapshot(conn, project))
            .map_err(CodedError::from)
    }
}
