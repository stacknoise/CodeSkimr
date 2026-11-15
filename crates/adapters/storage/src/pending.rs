//! The names as written, kept for the next incremental scan (issue #49).

use codeskimr_application::ports::{PendingReference, Receiver};
use codeskimr_domain::{FileId, QualifiedName, ReferenceKind, SymbolId};
use rusqlite::{Connection, params};

use crate::codec::{Cols, SpanColumns, column_int, id_from, label_from};
use crate::consts::{
    NAME_SEPARATOR, RECEIVER_BASE, RECEIVER_FREE, RECEIVER_NAMED, RECEIVER_OWN, RECEIVER_TYPED,
    RECEIVER_UNKNOWN,
};
use crate::error::{StorageError, StorageResult};
use crate::similarity::json_list;
use crate::sql;

/// The label and the name column of a receiver.
fn receiver_columns(receiver: Option<&Receiver>) -> (Option<&'static str>, Option<&str>) {
    match receiver {
        None => (None, None),
        Some(Receiver::Typed(name)) => (Some(RECEIVER_TYPED), Some(name.as_str())),
        Some(Receiver::Named(name)) => (Some(RECEIVER_NAMED), Some(name.as_str())),
        Some(Receiver::Own) => (Some(RECEIVER_OWN), None),
        Some(Receiver::Base) => (Some(RECEIVER_BASE), None),
        Some(Receiver::Unknown) => (Some(RECEIVER_UNKNOWN), None),
        Some(Receiver::Free) => (Some(RECEIVER_FREE), None),
    }
}

fn receiver_of(kind: Option<&str>, name: Option<String>) -> StorageResult<Option<Receiver>> {
    let qualified = |name: Option<String>| -> StorageResult<QualifiedName> {
        Ok(QualifiedName::new(name.ok_or(StorageError::OutOfRange)?)?)
    };
    Ok(match kind {
        None => None,
        Some(RECEIVER_TYPED) => Some(Receiver::Typed(qualified(name)?)),
        Some(RECEIVER_NAMED) => Some(Receiver::Named(qualified(name)?)),
        Some(RECEIVER_OWN) => Some(Receiver::Own),
        Some(RECEIVER_BASE) => Some(Receiver::Base),
        Some(RECEIVER_UNKNOWN) => Some(Receiver::Unknown),
        Some(RECEIVER_FREE) => Some(Receiver::Free),
        Some(_) => return Err(StorageError::OutOfRange),
    })
}

/// Writes the names of the analysed files.
pub fn insert_all(conn: &Connection, names: &[PendingReference]) -> StorageResult<()> {
    let mut statement = conn.prepare_cached(sql::PENDING_INSERT)?;
    for name in names {
        let span = SpanColumns::of(&name.span);
        let (receiver, receiver_name) = receiver_columns(name.receiver.as_ref());
        statement.execute(params![
            column_int(name.file.get())?,
            column_int(name.from.get())?,
            span.start_line,
            span.start_column,
            span.end_line,
            span.end_column,
            name.kind.label(),
            name.target.as_str(),
            receiver,
            receiver_name,
        ])?;
    }
    Ok(())
}

/// Marks the snapshot as one that holds the names as written.
pub fn mark_stored(conn: &Connection, snapshot: i64) -> StorageResult<()> {
    conn.execute(sql::SNAPSHOT_NAMES_SET, params![snapshot])?;
    Ok(())
}

/// Whether the snapshot holds the names as written.
pub fn is_stored(conn: &Connection, snapshot: i64) -> StorageResult<bool> {
    Ok(
        conn.query_row(sql::SNAPSHOT_NAMES_STORED, params![snapshot], |row| {
            row.get(0)
        })?,
    )
}

/// Which names to read.
pub enum Which<'a> {
    /// Base lists.
    Bases,
    /// The imports of these files.
    ImportsOf(&'a [FileId]),
    /// Names written inside these symbols of these files.
    Callers(&'a [FileId], &'a [SymbolId]),
    /// Names other than imports whose target or named receiver is one of these simple names.
    Targeting(&'a [String]),
    /// Imports whose target is one of these qualified names.
    ImportsTargeting(&'a [String]),
    /// All names of these files.
    OfFiles(&'a [FileId]),
    /// All names.
    All,
}

/// The names of the files of the snapshot that `which` asks for.
///
/// # Errors
/// `OutOfRange` if the snapshot does not exist (it was pruned), otherwise a database failure.
pub fn load(
    conn: &Connection,
    snapshot: i64,
    which: &Which<'_>,
) -> StorageResult<Vec<PendingReference>> {
    let exists: i64 = conn.query_row(sql::SNAPSHOT_EXISTS, params![snapshot], |row| row.get(0))?;
    if exists == 0 {
        return Err(StorageError::OutOfRange);
    }
    let import = ReferenceKind::Import.label();
    match which {
        Which::Bases => run(conn, sql::PENDING_BASES, params![snapshot]),
        Which::ImportsOf(files) => {
            let list = id_list(files.iter().map(|file| file.get()))?;
            run(
                conn,
                sql::PENDING_IMPORTS_OF,
                params![snapshot, list, import],
            )
        }
        Which::Callers(files, callers) => {
            let files = id_list(files.iter().map(|file| file.get()))?;
            let callers = id_list(callers.iter().map(|caller| caller.get()))?;
            run(
                conn,
                sql::PENDING_OF_CALLERS,
                params![snapshot, files, callers],
            )
        }
        Which::Targeting(names) => run(
            conn,
            sql::PENDING_TARGETING,
            params![snapshot, text_list(names), import],
        ),
        Which::ImportsTargeting(names) => run(
            conn,
            sql::PENDING_IMPORTS_TARGETING,
            params![snapshot, text_list(names), import],
        ),
        Which::OfFiles(files) => {
            let list = id_list(files.iter().map(|file| file.get()))?;
            run(conn, sql::PENDING_OF_FILES, params![snapshot, list])
        }
        Which::All => run(conn, sql::PENDING_OF_SNAPSHOT, params![snapshot]),
    }
}

/// Runs one of the statements above and decodes its rows.
fn run(
    conn: &Connection,
    statement: &str,
    params: impl rusqlite::Params,
) -> StorageResult<Vec<PendingReference>> {
    let mut prepared = conn.prepare_cached(statement)?;
    let rows = prepared.query_map(params, read_row)?;
    rows.map(|row| decode(row?)).collect()
}

/// The names in one text, each between two separators, so that SQL can ask whether a column is
/// one of them with `instr`. A name that holds the separator can only match too much, which
/// makes an incremental scan resolve a name again that it could have kept.
fn text_list(names: &[String]) -> String {
    let mut list = String::from(NAME_SEPARATOR);
    for name in names {
        list.push_str(name);
        list.push(NAME_SEPARATOR);
    }
    list
}

fn id_list(ids: impl Iterator<Item = u64>) -> StorageResult<String> {
    let mut columns = Vec::new();
    for id in ids {
        columns.push(column_int(id)?);
    }
    Ok(json_list(columns.into_iter()))
}

type Raw = (
    i64,
    i64,
    SpanColumns,
    String,
    String,
    Option<String>,
    Option<String>,
);

fn read_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Raw> {
    let cols = Cols::new(row);
    Ok((
        cols.take()?,
        cols.take()?,
        SpanColumns::read(&cols)?,
        cols.take()?,
        cols.take()?,
        cols.take()?,
        cols.take()?,
    ))
}

fn decode(raw: Raw) -> StorageResult<PendingReference> {
    let (file, from, span, kind, target, receiver, receiver_name) = raw;
    Ok(PendingReference {
        from: SymbolId::new(id_from(from)?),
        file: FileId::new(id_from(file)?),
        span: span.span()?,
        kind: label_from::<ReferenceKind>(&kind)?,
        target: QualifiedName::new(target)?,
        receiver: receiver_of(receiver.as_deref(), receiver_name)?,
    })
}
