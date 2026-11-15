//! The names as written, kept for the next incremental scan (issue #49).

use codeskimr_application::ports::{PendingReference, Receiver};
use codeskimr_domain::{FileId, QualifiedName, ReferenceKind, SymbolId};
use rusqlite::{Connection, params};

use crate::codec::{Cols, SpanColumns, column_int, id_from, label_from};
use crate::consts::{
    RECEIVER_BASE, RECEIVER_FREE, RECEIVER_NAMED, RECEIVER_OWN, RECEIVER_TYPED, RECEIVER_UNKNOWN,
};
use crate::error::{StorageError, StorageResult};
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

/// The names of all files of the snapshot.
pub fn load(conn: &Connection, snapshot: i64) -> StorageResult<Vec<PendingReference>> {
    let mut prepared = conn.prepare_cached(sql::PENDING_OF_SNAPSHOT)?;
    let rows = prepared.query_map(params![snapshot], |row| {
        let cols = Cols::new(row);
        Ok((
            cols.take::<i64>()?,
            cols.take::<i64>()?,
            SpanColumns::read(&cols)?,
            cols.take::<String>()?,
            cols.take::<String>()?,
            cols.take::<Option<String>>()?,
            cols.take::<Option<String>>()?,
        ))
    })?;
    let mut names = Vec::new();
    for row in rows {
        let (file, from, span, kind, target, receiver, receiver_name) = row?;
        names.push(PendingReference {
            from: SymbolId::new(id_from(from)?),
            file: FileId::new(id_from(file)?),
            span: span.span()?,
            kind: label_from::<ReferenceKind>(&kind)?,
            target: QualifiedName::new(target)?,
            receiver: receiver_of(receiver.as_deref(), receiver_name)?,
        });
    }
    Ok(names)
}
