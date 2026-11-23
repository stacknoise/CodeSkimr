//! Conversions between domain values and the integers and labels of the columns.

use std::str::FromStr;

use codeskimr_domain::{Position, Span};

use std::cell::Cell;

use rusqlite::Row;
use rusqlite::types::FromSql;

use crate::consts::{
    COLUMN_STEP, FIRST_COLUMN, FIRST_POSITION, JSON_LIST_CLOSE, JSON_LIST_OPEN, JSON_LIST_SEPARATOR,
};
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

/// The four numbers of a span.
#[derive(Debug, Clone, Copy)]
pub struct SpanColumns {
    /// Start line.
    pub start_line: i64,
    /// Start column.
    pub start_column: i64,
    /// End line.
    pub end_line: i64,
    /// End column.
    pub end_column: i64,
}

impl SpanColumns {
    /// Reads the four columns of a span from the next columns of a row.
    ///
    /// # Errors
    /// A `rusqlite` error if a column is missing or not a number.
    pub fn read(cols: &Cols<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            start_line: cols.take()?,
            start_column: cols.take()?,
            end_line: cols.take()?,
            end_column: cols.take()?,
        })
    }

    /// The span these columns describe.
    ///
    /// # Errors
    /// `OutOfRange` for a negative column, a model error for an inverted span.
    pub fn span(&self) -> StorageResult<Span> {
        let start = Position {
            line: small_from(self.start_line)?,
            column: small_from(self.start_column)?,
        };
        let end = Position {
            line: small_from(self.end_line)?,
            column: small_from(self.end_column)?,
        };
        Ok(Span::new(start, end)?)
    }

    /// Splits a span into its columns.
    pub fn of(span: &Span) -> Self {
        let (start, end) = (span.start(), span.end());
        Self {
            start_line: i64::from(start.line),
            start_column: i64::from(start.column),
            end_line: i64::from(end.line),
            end_column: i64::from(end.column),
        }
    }
}

/// Reads the columns of a row one after the other, so no column index is spelled out.
pub struct Cols<'a> {
    row: &'a Row<'a>,
    next: Cell<usize>,
}

impl<'a> Cols<'a> {
    /// Starts at the first column.
    pub const fn new(row: &'a Row<'a>) -> Self {
        Self {
            row,
            next: Cell::new(FIRST_COLUMN),
        }
    }

    /// The next column.
    pub fn take<T: FromSql>(&self) -> rusqlite::Result<T> {
        let index = self.next.get();
        self.next.set(index + COLUMN_STEP);
        self.row.get(index)
    }
}

/// Positions 0, 1, 2, ... for ordered child rows.
pub fn ordinals() -> impl Iterator<Item = i64> {
    FIRST_POSITION..
}

/// A list of numbers as the JSON text that `json_each` reads, so one statement takes any number
/// of values.
pub(crate) fn json_list(values: impl Iterator<Item = i64>) -> String {
    let items: Vec<String> = values.map(|value| value.to_string()).collect();
    format!(
        "{JSON_LIST_OPEN}{}{JSON_LIST_CLOSE}",
        items.join(&JSON_LIST_SEPARATOR.to_string())
    )
}

/// Ids as the JSON text that `json_each` reads.
///
/// # Errors
/// `OutOfRange` for an id beyond the column range.
pub(crate) fn id_list(ids: impl Iterator<Item = u64>) -> StorageResult<String> {
    let mut columns = Vec::new();
    for id in ids {
        columns.push(column_int(id)?);
    }
    Ok(json_list(columns.into_iter()))
}
