//! The `CodeOutline` port: the tree and the member tables of the explorer.

use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{
    CodeOutline, FOLDER_NAMESPACE_LANGUAGES, MemberRow, OutlineNode,
};
use codeskimr_domain::{LanguageId, MetricKind, QualifiedName, SnapshotId, SymbolId, SymbolName};
use rusqlite::{Connection, Row, params};

use crate::codec::{Cols, column_int, id_from, json_labels, label_from, small_from};
use crate::consts::{CONTAINER_KINDS, FOLDER_SEPARATOR, TYPE_KINDS};
use crate::error::StorageResult;
use crate::sql;
use crate::store::ProjectDb;

/// A row of `OUTLINE_OF_KIND` as stored.
struct NodeRow {
    kind: String,
    id: i64,
    name: String,
    qualified: String,
    parent: Option<i64>,
    members: i64,
    language: String,
    path: String,
}

/// The folder of a file path: everything before the last `/`, empty for a file in the root.
pub(crate) fn folder_of(path: &str) -> String {
    path.rsplit_once(FOLDER_SEPARATOR)
        .map(|(folder, _)| folder.to_owned())
        .unwrap_or_default()
}

impl NodeRow {
    // codeskimr: allow CHK-102 maps a row of the outline query into its columns
    fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        let cols = Cols::new(row);
        Ok(Self {
            kind: cols.take()?,
            id: cols.take()?,
            name: cols.take()?,
            qualified: cols.take()?,
            parent: cols.take()?,
            members: cols.take()?,
            language: cols.take()?,
            path: cols.take()?,
        })
    }

    fn node(self) -> StorageResult<OutlineNode> {
        let folder = if FOLDER_NAMESPACE_LANGUAGES.contains(&self.language.as_str()) {
            folder_of(&self.path)
        } else {
            String::new()
        };
        Ok(OutlineNode {
            id: SymbolId::new(id_from(self.id)?),
            name: SymbolName::new(self.name)?,
            qualified_name: QualifiedName::new(self.qualified)?,
            kind: label_from(&self.kind)?,
            parent: self
                .parent
                .map(|parent| id_from(parent).map(SymbolId::new))
                .transpose()?,
            members: small_from(self.members)?,
            language: LanguageId::new(self.language)?,
            folder,
        })
    }
}

fn outline(conn: &Connection, snapshot: SnapshotId) -> StorageResult<Vec<OutlineNode>> {
    let raw = column_int(snapshot.get())?;
    let kinds = json_labels(
        CONTAINER_KINDS
            .iter()
            .chain(TYPE_KINDS)
            .map(|kind| kind.label()),
    );
    let mut statement = conn.prepare_cached(sql::OUTLINE_OF_KINDS)?;
    let mut nodes = Vec::new();
    for row in statement.query_map(params![raw, kinds], NodeRow::from_row)? {
        nodes.push(row?.node()?);
    }
    nodes.sort_by(|a, b| {
        a.qualified_name
            .cmp(&b.qualified_name)
            .then(a.id.cmp(&b.id))
    });
    Ok(nodes)
}

/// A row of `MEMBER_ROWS` as stored.
struct MemberColumns {
    id: i64,
    name: String,
    kind: String,
    visibility: Option<String>,
    signature: Option<String>,
    first: i64,
    last: i64,
    qualified_name: String,
    complexity: Option<f64>,
    usages: i64,
}

impl MemberColumns {
    // codeskimr: allow CHK-102 maps a row of the member query into its columns
    fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        let cols = Cols::new(row);
        Ok(Self {
            id: cols.take()?,
            name: cols.take()?,
            kind: cols.take()?,
            visibility: cols.take()?,
            signature: cols.take()?,
            first: cols.take()?,
            last: cols.take()?,
            qualified_name: cols.take()?,
            complexity: cols.take()?,
            usages: cols.take()?,
        })
    }

    fn member(self) -> StorageResult<MemberRow> {
        Ok(MemberRow {
            id: SymbolId::new(id_from(self.id)?),
            name: SymbolName::new(self.name)?,
            qualified_name: QualifiedName::new(self.qualified_name)?,
            kind: label_from(&self.kind)?,
            visibility: self.visibility.as_deref().map(label_from).transpose()?,
            signature: self.signature,
            complexity: self.complexity,
            usages: small_from(self.usages)?,
            first_line: small_from(self.first)?,
            last_line: small_from(self.last)?,
        })
    }
}

fn member_rows(
    conn: &Connection,
    snapshot: SnapshotId,
    symbol: SymbolId,
) -> StorageResult<Vec<MemberRow>> {
    let mut statement = conn.prepare_cached(sql::MEMBER_ROWS)?;
    let params = params![
        column_int(snapshot.get())?,
        column_int(symbol.get())?,
        MetricKind::CyclomaticComplexity.label()
    ];
    let mut found = Vec::new();
    for row in statement.query_map(params, MemberColumns::from_row)? {
        found.push(row?.member()?);
    }
    Ok(found)
}

impl CodeOutline for ProjectDb {
    fn outline(&self, snapshot: SnapshotId) -> Result<Vec<OutlineNode>, CodedError> {
        self.read_apart(|conn| outline(conn, snapshot))
            .map_err(CodedError::from)
    }

    fn member_rows(
        &self,
        snapshot: SnapshotId,
        symbol: SymbolId,
    ) -> Result<Vec<MemberRow>, CodedError> {
        self.read_apart(|conn| member_rows(conn, snapshot, symbol))
            .map_err(CodedError::from)
    }
}
