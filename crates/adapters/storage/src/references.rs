//! The references of a snapshot as stored.

use std::collections::{BTreeSet, HashSet};

use codeskimr_domain::{
    Confidence, FileId, Reference, ReferenceId, ReferenceKind, SymbolId, SymbolPair,
};
use rusqlite::{Connection, params};

use crate::codec::{Cols, SpanColumns, column_int, id_from, label_from};
use crate::error::StorageResult;
use crate::pending::id_list;
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

/// Which references of a snapshot to read.
pub(crate) enum Which<'a> {
    /// Those that point at one of these symbols.
    To(&'a [SymbolId]),
    /// Those that start in one of these symbols.
    From(&'a [SymbolId]),
    /// Those that stand in one of these files.
    In(&'a [FileId]),
    /// Every reference.
    All,
}

/// The references of a snapshot: those whose file and target it holds.
pub(crate) fn load_references(conn: &Connection, snapshot: i64) -> StorageResult<Vec<Reference>> {
    load(conn, snapshot, &Which::All)
}

/// The references of a snapshot that `which` asks for: those whose file and target it holds.
pub(crate) fn load(
    conn: &Connection,
    snapshot: i64,
    which: &Which<'_>,
) -> StorageResult<Vec<Reference>> {
    let files = id_set(conn, sql::FILE_IDS, snapshot)?;
    let symbols = id_set(conn, sql::SYMBOL_IDS, snapshot)?;
    let symbols_of = |ids: &[SymbolId]| id_list(ids.iter().map(|id| id.get())).map(Some);
    let (statement, list) = match which {
        Which::To(ids) => (sql::REFERENCES_TO, symbols_of(ids)?),
        Which::From(ids) => (sql::REFERENCES_FROM, symbols_of(ids)?),
        Which::In(ids) => (
            sql::REFERENCES_IN,
            Some(id_list(ids.iter().map(|id| id.get()))?),
        ),
        Which::All => (sql::REFERENCES_OF_SNAPSHOT, None),
    };
    let mut prepared = conn.prepare_cached(statement)?;
    let mut rows = match &list {
        Some(list) => prepared.query(params![snapshot, list])?,
        None => prepared.query(params![snapshot])?,
    };
    let mut references = Vec::new();
    while let Some(row) = rows.next()? {
        let row = read_reference(&Cols::new(row))?;
        if files.contains(&row.file) && symbols.contains(&row.to) {
            references.push(decode_reference(&row)?);
        }
    }
    // The statements with a list read through an index; in the order of the ids the result does
    // not depend on which one.
    references.sort_unstable_by_key(|found| found.id);
    Ok(references)
}

/// The files of a snapshot that hold a reference verified by level 2, in ascending order.
pub(crate) fn verified_files(conn: &Connection, snapshot: i64) -> StorageResult<Vec<FileId>> {
    let files = id_set(conn, sql::FILE_IDS, snapshot)?;
    let symbols = id_set(conn, sql::SYMBOL_IDS, snapshot)?;
    let mut prepared = conn.prepare_cached(sql::VERIFIED_REFERENCE_FILES)?;
    let mut rows = prepared.query(params![snapshot])?;
    let mut found = BTreeSet::new();
    while let Some(row) = rows.next()? {
        let (file, to): (i64, i64) = (row.get(0)?, row.get(1)?);
        if files.contains(&file) && symbols.contains(&to) {
            found.insert(id_from(file)?);
        }
    }
    Ok(found.into_iter().map(FileId::new).collect())
}

/// Each pair of start and target of the references of a snapshot once, in ascending order: those
/// that stand in `of` and do not start in `without`, references of kind `test` left out. It is
/// what coupling needs of them (ADR 0085).
pub(crate) fn reference_pairs(
    conn: &Connection,
    snapshot: i64,
    of: &[FileId],
    without: &[SymbolId],
) -> StorageResult<Vec<SymbolPair>> {
    let files = of
        .iter()
        .map(|file| column_int(file.get()))
        .collect::<StorageResult<HashSet<i64>>>()?;
    let skipped = without
        .iter()
        .map(|symbol| column_int(symbol.get()))
        .collect::<StorageResult<HashSet<i64>>>()?;
    let symbols = id_set(conn, sql::SYMBOL_IDS, snapshot)?;
    let mut prepared = conn.prepare_cached(sql::REFERENCE_PAIRS)?;
    let mut rows = prepared.query(params![snapshot, ReferenceKind::Test.label()])?;
    let mut found = BTreeSet::new();
    while let Some(row) = rows.next()? {
        let (file, from, to): (i64, i64, i64) = (row.get(0)?, row.get(1)?, row.get(2)?);
        if files.contains(&file) && symbols.contains(&to) && !skipped.contains(&from) {
            found.insert((id_from(from)?, id_from(to)?));
        }
    }
    Ok(found
        .into_iter()
        .map(|(from, to)| (SymbolId::new(from), SymbolId::new(to)))
        .collect())
}
