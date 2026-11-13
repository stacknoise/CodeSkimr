//! Versioned schema migrations, applied at start-up (blueprint 6.2).

use rusqlite::Connection;

use crate::consts::{FIRST_VERSION, MIGRATIONS, PRAGMA_USER_VERSION};
use crate::error::{StorageError, StorageResult};

/// Brings the schema up to the newest version. Every migration runs in its own transaction, so a
/// failure leaves the database at the last good version.
pub fn apply(conn: &mut Connection) -> StorageResult<()> {
    let current: i64 = conn
        .pragma_query_value(None, PRAGMA_USER_VERSION, |row| row.get(0))
        .map_err(StorageError::migration)?;
    let known = i64::try_from(MIGRATIONS.len()).map_err(|_| StorageError::OutOfRange)?;
    if current > known {
        return Err(StorageError::SchemaTooNew);
    }
    for (version, sql) in (FIRST_VERSION..).zip(MIGRATIONS) {
        if version > current {
            apply_one(conn, version, sql).map_err(StorageError::migration)?;
        }
    }
    Ok(())
}

fn apply_one(conn: &mut Connection, version: i64, sql: &str) -> rusqlite::Result<()> {
    let tx = conn.transaction()?;
    tx.execute_batch(sql)?;
    tx.pragma_update(None, PRAGMA_USER_VERSION, version)?;
    tx.commit()
}
