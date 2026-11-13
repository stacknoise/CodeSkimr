//! The symbols and references of the newest snapshot, the start of an incremental scan.

use codeskimr_application::ports::PreviousIndex;
use codeskimr_domain::{
    Confidence, FileId, ProjectId, Reference, ReferenceId, ReferenceKind, Symbol, SymbolId,
};
use rusqlite::{Connection, params};

use crate::codec::{Cols, SpanColumns, column_int, id_from, label_from};
use crate::detail::{decode, read_row};
use crate::error::StorageResult;
use crate::sql;

/// The index of the newest snapshot of `project`, empty if there is none.
pub fn load_latest_index(conn: &Connection, project: ProjectId) -> StorageResult<PreviousIndex> {
    let latest: Option<i64> = conn.query_row(
        sql::LATEST_SNAPSHOT_ID,
        params![column_int(project.get())?],
        |row| row.get(0),
    )?;
    let Some(snapshot) = latest else {
        return Ok(PreviousIndex::default());
    };
    Ok(PreviousIndex {
        symbols: load_symbols(conn, snapshot)?,
        references: load_references(conn, snapshot)?,
    })
}

fn load_symbols(conn: &Connection, snapshot: i64) -> StorageResult<Vec<Symbol>> {
    let statement = format!("{}{}", sql::SYMBOL_SELECT, sql::SYMBOL_ALL);
    let mut prepared = conn.prepare_cached(&statement)?;
    let rows = prepared.query_map(params![snapshot], |row| read_row(&Cols::new(row)))?;
    let mut symbols = Vec::new();
    for row in rows {
        symbols.push(decode(row?)?.0);
    }
    Ok(symbols)
}

/// The columns of `REFERENCES_OF_SNAPSHOT`, still raw.
struct ReferenceRow {
    id: i64,
    from: i64,
    to: i64,
    file: i64,
    span: SpanColumns,
    kind: String,
    confidence: String,
}

fn read_reference(cols: &Cols<'_>) -> rusqlite::Result<ReferenceRow> {
    Ok(ReferenceRow {
        id: cols.take()?,
        from: cols.take()?,
        to: cols.take()?,
        file: cols.take()?,
        span: SpanColumns::read(cols)?,
        kind: cols.take()?,
        confidence: cols.take()?,
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
    })
}

pub(crate) fn load_references(conn: &Connection, snapshot: i64) -> StorageResult<Vec<Reference>> {
    let mut prepared = conn.prepare_cached(sql::REFERENCES_OF_SNAPSHOT)?;
    let rows = prepared.query_map(params![snapshot], |row| read_reference(&Cols::new(row)))?;
    let mut references = Vec::new();
    for row in rows {
        references.push(decode_reference(&row?)?);
    }
    Ok(references)
}
