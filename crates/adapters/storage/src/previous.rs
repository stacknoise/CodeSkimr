//! The symbols and references of the newest snapshot, the start of an incremental scan.

use codeskimr_application::ports::PreviousIndex;
use codeskimr_domain::{ProjectId, Symbol};
use rusqlite::{Connection, params};

use crate::codec::{Cols, column_int};
use crate::detail::{decode, read_row};
use crate::error::StorageResult;
use crate::references::load_references;
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
