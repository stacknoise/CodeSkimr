//! Conversions between domain values and the integers and labels of the columns.

use std::str::FromStr;

use codeskimr_domain::Span;

use crate::consts::FIRST_POSITION;
use crate::error::{StorageError, StorageResult};

/// An id or count as a column value.
pub fn column_int(value: u64) -> StorageResult<i64> {
    i64::try_from(value).map_err(|_| StorageError::OutOfRange)
}

/// A column value as an id.
pub fn id_from(value: i64) -> StorageResult<u64> {
    u64::try_from(value).map_err(|_| StorageError::OutOfRange)
}

/// A column value as a small number such as a line or a count.
pub fn small_from(value: i64) -> StorageResult<u32> {
    u32::try_from(value).map_err(|_| StorageError::OutOfRange)
}

/// A stored label as an enumeration variant.
pub fn label_from<T>(text: &str) -> StorageResult<T>
where
    T: FromStr<Err = codeskimr_domain::ModelError>,
{
    Ok(text.parse::<T>()?)
}

/// The four numbers of a span: start line, start column, end line, end column.
#[derive(Debug, Clone, Copy)]
pub struct SpanColumns(pub i64, pub i64, pub i64, pub i64);

impl SpanColumns {
    /// Splits a span into its columns.
    pub fn of(span: &Span) -> Self {
        let (start, end) = (span.start(), span.end());
        Self(
            i64::from(start.line),
            i64::from(start.column),
            i64::from(end.line),
            i64::from(end.column),
        )
    }
}

/// Positions 0, 1, 2, ... for ordered child rows.
pub fn ordinals() -> impl Iterator<Item = i64> {
    FIRST_POSITION..
}
