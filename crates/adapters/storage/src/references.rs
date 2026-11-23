//! The references of a snapshot as stored.

use std::collections::{BTreeSet, HashMap, HashSet};

use codeskimr_domain::{
    Confidence, Direction, FileId, Reference, ReferenceId, ReferenceKind, SymbolId, SymbolPair,
    UsageTally,
};
use rusqlite::{Connection, params};

use crate::codec::{Cols, SpanColumns, column_int, id_from, id_list, label_from};
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

/// The references of a snapshot that start in or point at the symbols `ids`, as the view
/// `snapshot_references` holds them: one statement through the index on the end of a reference,
/// whatever the size of the project (ADR 0089).
pub(crate) fn load_around(
    conn: &Connection,
    snapshot: i64,
    direction: Direction,
    ids: &[SymbolId],
) -> StorageResult<Vec<Reference>> {
    let statement = match direction {
        Direction::Incoming => sql::REFERENCES_INTO,
        Direction::Outgoing => sql::REFERENCES_OUT_OF,
    };
    let list = id_list(ids.iter().map(|id| id.get()))?;
    let mut prepared = conn.prepare_cached(statement)?;
    let mut rows = prepared.query(params![snapshot, list])?;
    let mut references = Vec::new();
    while let Some(row) = rows.next()? {
        references.push(decode_reference(&read_reference(&Cols::new(row))?)?);
    }
    Ok(references)
}

/// How the references of a snapshot are used, counted in one pass over the table without keeping
/// them (ADR 0089): the usage count of every symbol, the files that hold a reference of kind
/// `test` and the symbols that override a member.
#[derive(Debug, Default)]
pub(crate) struct ReferenceFacts {
    pub(crate) tallies: HashMap<SymbolId, UsageTally>,
    pub(crate) test_files: HashSet<FileId>,
    pub(crate) overriding: HashSet<SymbolId>,
}

/// The kind, whether it is probable and whether level 2 verified it, of a row of
/// `REFERENCE_TALLY_ROWS`, read without copying the text.
fn tally_columns(row: &rusqlite::Row<'_>) -> StorageResult<(ReferenceKind, bool, bool)> {
    const KIND: usize = 3;
    let text = |column: usize| -> rusqlite::Result<&str> {
        row.get_ref(column)?.as_str().map_err(rusqlite::Error::from)
    };
    let kind = label_from::<ReferenceKind>(text(KIND)?)?;
    let probable = text(KIND + 1)? == Confidence::Probable.label();
    Ok((kind, probable, row.get(KIND + 2)?))
}

/// Counts the references of a snapshot: those whose file and target it holds. References of kind
/// `test` and of a symbol to itself are no usages; a reference is certain if its resolution is,
/// or if level 2 verified it.
pub(crate) fn reference_facts(conn: &Connection, snapshot: i64) -> StorageResult<ReferenceFacts> {
    let files = id_set(conn, sql::FILE_IDS, snapshot)?;
    let symbols = id_set(conn, sql::SYMBOL_IDS, snapshot)?;
    let mut prepared = conn.prepare_cached(sql::REFERENCE_TALLY_ROWS)?;
    let mut rows = prepared.query(params![snapshot])?;
    let mut facts = ReferenceFacts::default();
    while let Some(row) = rows.next()? {
        let cols = Cols::new(row);
        let (file, from, to): (i64, i64, i64) = (cols.take()?, cols.take()?, cols.take()?);
        if !files.contains(&file) || !symbols.contains(&to) {
            continue;
        }
        let (kind, probable, verified) = tally_columns(row)?;
        let (from, to) = (SymbolId::new(id_from(from)?), SymbolId::new(id_from(to)?));
        match kind {
            ReferenceKind::Test => {
                facts.test_files.insert(FileId::new(id_from(file)?));
            }
            _ if from == to => {}
            _ => {
                facts
                    .tallies
                    .entry(to)
                    .or_default()
                    .count(verified || !probable);
            }
        }
        if kind == ReferenceKind::Override {
            facts.overriding.insert(from);
        }
    }
    Ok(facts)
}

/// The ends (start, target) of the references of a snapshot that are not of kind `test` and not
/// probable, whose file and target the snapshot holds, one per reference. They are what the
/// dependencies between namespaces count (issue #464).
pub(crate) fn certain_ends(
    conn: &Connection,
    snapshot: i64,
) -> StorageResult<Vec<(SymbolId, SymbolId)>> {
    let files = id_set(conn, sql::FILE_IDS, snapshot)?;
    let symbols = id_set(conn, sql::SYMBOL_IDS, snapshot)?;
    let mut prepared = conn.prepare_cached(sql::REFERENCE_ENDS_CERTAIN)?;
    let mut rows = prepared.query(params![
        snapshot,
        ReferenceKind::Test.label(),
        Confidence::Probable.label()
    ])?;
    let mut ends = Vec::new();
    while let Some(row) = rows.next()? {
        let cols = Cols::new(row);
        let (file, from, to): (i64, i64, i64) = (cols.take()?, cols.take()?, cols.take()?);
        if files.contains(&file) && symbols.contains(&to) {
            ends.push((SymbolId::new(id_from(from)?), SymbolId::new(id_from(to)?)));
        }
    }
    Ok(ends)
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
/// what coupling needs of them (ADR 0086).
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
        let cols = Cols::new(row);
        let (file, from, to): (i64, i64, i64) = (cols.take()?, cols.take()?, cols.take()?);
        if files.contains(&file) && symbols.contains(&to) && !skipped.contains(&from) {
            found.insert((id_from(from)?, id_from(to)?));
        }
    }
    Ok(found
        .into_iter()
        .map(|(from, to)| (SymbolId::new(from), SymbolId::new(to)))
        .collect())
}
