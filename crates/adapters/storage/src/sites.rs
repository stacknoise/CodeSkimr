//! The `ReferenceSites` port: the references that leave the functions of some files.

use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{
    ReferencePlace, ReferencePlaces, ReferenceSite, ReferenceSites,
};
use codeskimr_domain::{
    Confidence, NamespaceGraph, QualifiedName, ReferenceKind, RelativePath, SnapshotId,
};
use rusqlite::{Connection, params};

use crate::codec::{Cols, SpanColumns, column_int};
use crate::error::StorageResult;
use crate::sql;
use crate::store::ProjectDb;

/// The columns of `REFERENCE_SITES`, still raw.
struct Row {
    from: String,
    span: SpanColumns,
    to: String,
}

fn rows_of(conn: &Connection, snapshot: i64, file: &RelativePath) -> StorageResult<Vec<Row>> {
    let mut statement = conn.prepare_cached(sql::REFERENCE_SITES)?;
    let bind = params![
        snapshot,
        ReferenceKind::Test.label(),
        Confidence::Certain.label(),
        file.as_str()
    ];
    let rows = statement.query_map(bind, |row| {
        let cols = Cols::new(row);
        let _path: String = cols.take()?;
        Ok(Row {
            from: cols.take()?,
            span: SpanColumns::read(&cols)?,
            to: cols.take()?,
        })
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}

fn sites(
    conn: &Connection,
    snapshot: SnapshotId,
    files: &[RelativePath],
) -> StorageResult<Vec<ReferenceSite>> {
    let snapshot = column_int(snapshot.get())?;
    let mut found = Vec::new();
    for file in files {
        for row in rows_of(conn, snapshot, file)? {
            found.push(ReferenceSite {
                path: file.clone(),
                from: QualifiedName::new(row.from)?,
                span: row.span.span()?,
                to: QualifiedName::new(row.to)?,
            });
        }
    }
    Ok(found)
}

impl ReferenceSites for ProjectDb {
    fn references_from(
        &self,
        snapshot: SnapshotId,
        files: &[RelativePath],
    ) -> Result<Vec<ReferenceSite>, CodedError> {
        self.read(|conn| sites(conn, snapshot, files))
            .map_err(CodedError::from)
    }
}

/// Hands every certain reference of the snapshot to `visit`, one row at a time.
fn places(
    conn: &Connection,
    snapshot: SnapshotId,
    visit: &mut dyn FnMut(ReferencePlace),
) -> StorageResult<()> {
    let mut statement = conn.prepare_cached(sql::REFERENCE_PLACES)?;
    let bind = params![
        column_int(snapshot.get())?,
        ReferenceKind::Test.label(),
        Confidence::Certain.label()
    ];
    let mut rows = statement.query(bind)?;
    while let Some(row) = rows.next()? {
        let cols = Cols::new(row);
        let path: String = cols.take()?;
        let span = SpanColumns::read(&cols)?;
        let (from, to): (String, String) = (cols.take()?, cols.take()?);
        visit(ReferencePlace {
            path: RelativePath::new(path)?,
            span: span.span()?,
            from: QualifiedName::new(from)?,
            to: QualifiedName::new(to)?,
        });
    }
    Ok(())
}

impl ReferencePlaces for ProjectDb {
    fn for_each_reference(
        &self,
        snapshot: SnapshotId,
        visit: &mut dyn FnMut(ReferencePlace),
    ) -> Result<(), CodedError> {
        self.read(|conn| places(conn, snapshot, visit))
            .map_err(CodedError::from)
    }

    fn namespaces(&self, snapshot: SnapshotId) -> Result<NamespaceGraph, CodedError> {
        self.namespace_graph(snapshot)
    }
}
