//! The `CodeOutline` port: the tree and the member tables of the explorer.

use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{CodeOutline, MemberRow, OutlineNode};
use codeskimr_domain::{MetricKind, QualifiedName, SnapshotId, SymbolId, SymbolName};
use rusqlite::{Connection, params};

use crate::codec::{Cols, column_int, id_from, label_from, small_from};
use crate::consts::OUTLINE_KINDS;
use crate::error::StorageResult;
use crate::sql;
use crate::store::SqliteStore;

fn nodes_of_kind(
    conn: &Connection,
    snapshot: i64,
    kind: &str,
    nodes: &mut Vec<OutlineNode>,
) -> StorageResult<()> {
    let mut statement = conn.prepare_cached(sql::OUTLINE_OF_KIND)?;
    let rows = statement.query_map(params![snapshot, kind], |row| {
        let cols = Cols::new(row);
        Ok((
            cols.take::<i64>()?,
            cols.take::<String>()?,
            cols.take::<String>()?,
            cols.take::<String>()?,
            cols.take::<Option<i64>>()?,
            cols.take::<i64>()?,
        ))
    })?;
    for row in rows {
        let (id, name, qualified, kind, parent, members) = row?;
        nodes.push(OutlineNode {
            id: SymbolId::new(id_from(id)?),
            name: SymbolName::new(name)?,
            qualified_name: QualifiedName::new(qualified)?,
            kind: label_from(&kind)?,
            parent: parent
                .map(|parent| id_from(parent).map(SymbolId::new))
                .transpose()?,
            members: small_from(members)?,
        });
    }
    Ok(())
}

fn outline(conn: &Connection, snapshot: SnapshotId) -> StorageResult<Vec<OutlineNode>> {
    let raw = column_int(snapshot.get())?;
    let mut nodes = Vec::new();
    for kind in OUTLINE_KINDS {
        nodes_of_kind(conn, raw, kind.label(), &mut nodes)?;
    }
    nodes.sort_by(|a, b| {
        a.qualified_name
            .cmp(&b.qualified_name)
            .then(a.id.cmp(&b.id))
    });
    Ok(nodes)
}

fn member_rows(
    conn: &Connection,
    snapshot: SnapshotId,
    symbol: SymbolId,
) -> StorageResult<Vec<MemberRow>> {
    let mut statement = conn.prepare_cached(sql::MEMBER_ROWS)?;
    let rows = statement.query_map(
        params![
            column_int(snapshot.get())?,
            column_int(symbol.get())?,
            MetricKind::CyclomaticComplexity.label()
        ],
        |row| {
            let cols = Cols::new(row);
            Ok((
                cols.take::<i64>()?,
                cols.take::<String>()?,
                cols.take::<String>()?,
                cols.take::<Option<String>>()?,
                cols.take::<Option<String>>()?,
                cols.take::<i64>()?,
                cols.take::<i64>()?,
                cols.take::<Option<f64>>()?,
                cols.take::<i64>()?,
            ))
        },
    )?;
    let mut found = Vec::new();
    for row in rows {
        let (id, name, kind, visibility, signature, first, last, complexity, usages) = row?;
        found.push(MemberRow {
            id: SymbolId::new(id_from(id)?),
            name: SymbolName::new(name)?,
            kind: label_from(&kind)?,
            visibility: visibility.as_deref().map(label_from).transpose()?,
            signature,
            complexity,
            usages: small_from(usages)?,
            first_line: small_from(first)?,
            last_line: small_from(last)?,
        });
    }
    Ok(found)
}

impl CodeOutline for SqliteStore {
    fn outline(&self, snapshot: SnapshotId) -> Result<Vec<OutlineNode>, CodedError> {
        self.read(|conn| outline(conn, snapshot))
            .map_err(CodedError::from)
    }

    fn member_rows(
        &self,
        snapshot: SnapshotId,
        symbol: SymbolId,
    ) -> Result<Vec<MemberRow>, CodedError> {
        self.read(|conn| member_rows(conn, snapshot, symbol))
            .map_err(CodedError::from)
    }
}
