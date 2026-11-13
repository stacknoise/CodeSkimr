//! The lookup of one symbol and the `CodeIndex` port.

use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{CodeIndex, MemberRef, SearchHit, SymbolDetail, SymbolKey};
use codeskimr_domain::{
    FileId, LayerEdge, Metric, MetricValue, Position, ProjectId, QualifiedName, RelativePath,
    Snapshot, SnapshotId, Span, Symbol, SymbolId, SymbolName,
};
use rusqlite::{Connection, OptionalExtension, params};

use crate::codec::{Cols, column_int, id_from, label_from, small_from};
use crate::error::StorageResult;
use crate::sql;
use crate::store::SqliteStore;

/// The columns of `SYMBOL_SELECT`, still raw.
struct SymbolRow {
    id: i64,
    file: i64,
    kind: String,
    name: String,
    qualified: String,
    parent: Option<i64>,
    visibility: Option<String>,
    is_static: bool,
    span: [i64; 4],
    signature: Option<String>,
    doc: Option<String>,
    path: String,
}

fn read_row(cols: &Cols<'_>) -> rusqlite::Result<SymbolRow> {
    Ok(SymbolRow {
        id: cols.take()?,
        file: cols.take()?,
        kind: cols.take()?,
        name: cols.take()?,
        qualified: cols.take()?,
        parent: cols.take()?,
        visibility: cols.take()?,
        is_static: cols.take()?,
        span: [cols.take()?, cols.take()?, cols.take()?, cols.take()?],
        signature: cols.take()?,
        doc: cols.take()?,
        path: cols.take()?,
    })
}

fn position(line: i64, column: i64) -> StorageResult<Position> {
    Ok(Position {
        line: small_from(line)?,
        column: small_from(column)?,
    })
}

fn decode(row: SymbolRow) -> StorageResult<(Symbol, RelativePath)> {
    let [start_line, start_column, end_line, end_column] = row.span;
    let symbol = Symbol {
        id: SymbolId::new(id_from(row.id)?),
        file: FileId::new(id_from(row.file)?),
        kind: label_from(&row.kind)?,
        name: SymbolName::new(row.name)?,
        qualified_name: QualifiedName::new(row.qualified)?,
        parent: row
            .parent
            .map(|parent| id_from(parent).map(SymbolId::new))
            .transpose()?,
        visibility: row.visibility.as_deref().map(label_from).transpose()?,
        is_static: row.is_static,
        span: Span::new(
            position(start_line, start_column)?,
            position(end_line, end_column)?,
        )?,
        signature: row.signature,
        doc: row.doc,
    };
    Ok((symbol, RelativePath::new(row.path)?))
}

fn find_symbol(
    conn: &Connection,
    snapshot: i64,
    key: &SymbolKey,
) -> StorageResult<Option<(Symbol, RelativePath)>> {
    let found = match key {
        SymbolKey::Id(id) => {
            let statement = format!("{}{}", sql::SYMBOL_SELECT, sql::SYMBOL_BY_ID);
            conn.prepare_cached(&statement)?
                .query_row(params![snapshot, column_int(id.get())?], |row| {
                    read_row(&Cols::new(row))
                })
                .optional()?
        }
        SymbolKey::Qualified(name) => {
            let statement = format!("{}{}", sql::SYMBOL_SELECT, sql::SYMBOL_BY_NAME);
            conn.prepare_cached(&statement)?
                .query_row(params![snapshot, name.as_str()], |row| {
                    read_row(&Cols::new(row))
                })
                .optional()?
        }
    };
    found.map(decode).transpose()
}

fn members(conn: &Connection, snapshot: i64, symbol: SymbolId) -> StorageResult<Vec<MemberRef>> {
    let mut statement = conn.prepare_cached(sql::MEMBERS_OF)?;
    let rows = statement.query_map(params![snapshot, column_int(symbol.get())?], |row| {
        let cols = Cols::new(row);
        Ok((
            cols.take::<i64>()?,
            cols.take::<String>()?,
            cols.take::<String>()?,
        ))
    })?;
    let mut found = Vec::new();
    for row in rows {
        let (id, name, kind) = row?;
        found.push(MemberRef {
            id: SymbolId::new(id_from(id)?),
            name: SymbolName::new(name)?,
            kind: label_from(&kind)?,
        });
    }
    Ok(found)
}

fn metrics(conn: &Connection, snapshot: i64, symbol: SymbolId) -> StorageResult<Vec<Metric>> {
    let mut statement = conn.prepare_cached(sql::METRICS_OF)?;
    let rows = statement.query_map(params![snapshot, column_int(symbol.get())?], |row| {
        let cols = Cols::new(row);
        Ok((cols.take::<String>()?, cols.take::<f64>()?))
    })?;
    let mut found = Vec::new();
    for row in rows {
        let (kind, value) = row?;
        found.push(Metric {
            symbol,
            kind: label_from(&kind)?,
            value: MetricValue::new(value)?,
        });
    }
    Ok(found)
}

fn detail(
    conn: &Connection,
    snapshot: i64,
    key: &SymbolKey,
) -> StorageResult<Option<SymbolDetail>> {
    let Some((symbol, path)) = find_symbol(conn, snapshot, key)? else {
        return Ok(None);
    };
    Ok(Some(SymbolDetail {
        members: members(conn, snapshot, symbol.id)?,
        metrics: metrics(conn, snapshot, symbol.id)?,
        symbol,
        path,
    }))
}

impl SqliteStore {
    /// One symbol with its members and metrics, `None` if the snapshot has no such symbol.
    ///
    /// # Errors
    /// A coded `STO` failure.
    pub fn symbol_detail(
        &self,
        snapshot: SnapshotId,
        key: &SymbolKey,
    ) -> Result<Option<SymbolDetail>, CodedError> {
        self.read(|conn| detail(conn, column_int(snapshot.get())?, key))
            .map_err(CodedError::from)
    }
}

impl CodeIndex for SqliteStore {
    fn latest_snapshot(&self, project: ProjectId) -> Result<Option<Snapshot>, CodedError> {
        Self::latest_snapshot(self, project)
    }

    fn search(
        &self,
        snapshot: SnapshotId,
        query: &str,
        limit: u32,
    ) -> Result<Vec<SearchHit>, CodedError> {
        Self::search(self, snapshot, query, limit)
    }

    fn symbol(
        &self,
        snapshot: SnapshotId,
        key: &SymbolKey,
    ) -> Result<Option<SymbolDetail>, CodedError> {
        self.symbol_detail(snapshot, key)
    }

    fn layer_edges(&self, snapshot: SnapshotId) -> Result<Vec<LayerEdge>, CodedError> {
        Self::layer_edges(self, snapshot)
    }
}
