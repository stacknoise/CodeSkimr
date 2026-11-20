//! The references of a snapshot as stored.

use std::collections::HashSet;

use codeskimr_domain::{Confidence, FileId, Reference, ReferenceId, ReferenceKind, SymbolId};
use rusqlite::{Connection, params};

use crate::codec::{Cols, SpanColumns, id_from, label_from};
use crate::error::StorageResult;
use crate::sql;

/// The columns of `REFERENCES_OF_SNAPSHOT`, still raw.
struct ReferenceRow {
    id: i64,
    from: i64,
    to: i64,
    file: i64,
    span: SpanColumns,
    kind: String,
    confidence: String,
    verified: bool,
}

// codeskimr: allow CHK-102 maps a row of the reference query into its columns
fn read_reference(cols: &Cols<'_>) -> rusqlite::Result<ReferenceRow> {
    Ok(ReferenceRow {
        id: cols.take()?,
        from: cols.take()?,
        to: cols.take()?,
        file: cols.take()?,
        span: SpanColumns::read(cols)?,
        kind: cols.take()?,
        confidence: cols.take()?,
        verified: cols.take()?,
    })
}

fn decode_reference(row: &ReferenceRow) -> StorageResult<Reference> {
    Ok(Reference {
        id: ReferenceId::new(id_from(row.id)?),
        from: SymbolId::new(id_from(row.from)?),
        to: SymbolId::new(id_from(row.to)?),
        file: FileId::new(id_from(row.file)?),
        span: row.span.span()?,
        kind: label_from::<ReferenceKind>(&row.kind)?,
        confidence: label_from::<Confidence>(&row.confidence)?,
        verified: row.verified,
    })
}

/// The ids that the first column of `statement` returns for `snapshot`.
pub(crate) fn id_set(
    conn: &Connection,
    statement: &str,
    snapshot: i64,
) -> StorageResult<HashSet<i64>> {
    let mut prepared = conn.prepare_cached(statement)?;
    let rows = prepared.query_map(params![snapshot], |row| row.get::<_, i64>(0))?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// The references of a snapshot: those whose file and target it holds.
pub(crate) fn load_references(conn: &Connection, snapshot: i64) -> StorageResult<Vec<Reference>> {
    let files = id_set(conn, sql::FILE_IDS, snapshot)?;
    let symbols = id_set(conn, sql::SYMBOL_IDS, snapshot)?;
    let mut prepared = conn.prepare_cached(sql::REFERENCES_OF_SNAPSHOT)?;
    let rows = prepared.query_map(params![snapshot], |row| read_reference(&Cols::new(row)))?;
    let mut references = Vec::new();
    for row in rows {
        let row = row?;
        if files.contains(&row.file) && symbols.contains(&row.to) {
            references.push(decode_reference(&row)?);
        }
    }
    Ok(references)
}
