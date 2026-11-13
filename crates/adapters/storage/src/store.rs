//! The `SQLite` connection and the unit of work.

use std::fs;
use std::num::NonZeroU32;
use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use codeskimr_application::error::CodedError;
use codeskimr_domain::SnapshotId;
use rusqlite::{Connection, Transaction, TransactionBehavior};

use crate::consts::{
    BUSY_TIMEOUT_MS, CACHE_SIZE_KIB, FIRST_ID, ID_STEP, JOURNAL_MODE_WAL, PRAGMA_CACHE_SIZE,
    PRAGMA_FOREIGN_KEYS, PRAGMA_JOURNAL_MODE, PRAGMA_ON, PRAGMA_SYNCHRONOUS, PRAGMA_TEMP_STORE,
    SYNCHRONOUS_NORMAL, TEMP_STORE_MEMORY,
};
use crate::error::StorageResult;
use crate::sql;
use crate::{codec, migrate};

/// The `SQLite` database of one workspace. It implements the storage ports and offers the queries
/// on the read models. Share it behind an `Arc`; one writer at a time, readers wait for it.
#[derive(Debug)]
pub struct SqliteStore {
    conn: Mutex<Connection>,
    retention: NonZeroU32,
    /// The reverse index and the places of the snapshot that was asked about last; a snapshot
    /// never changes.
    pub(crate) reverse: Mutex<Option<(SnapshotId, Arc<crate::usages::SnapshotLookup>)>>,
}

impl SqliteStore {
    /// Opens the database file, creating it and its folder if needed, and applies the migrations.
    /// `retention` is the number of snapshots kept per project (`snapshots.retention_count`).
    ///
    /// # Errors
    /// `STO-301` if a migration fails, `STO-501` if the disk is full, otherwise `STO-601`.
    pub fn open(path: &Path, retention: NonZeroU32) -> Result<Self, CodedError> {
        Self::open_checked(path, retention).map_err(CodedError::from)
    }

    /// Opens a private in-memory database, for tests and short-lived tools.
    ///
    /// # Errors
    /// `STO-301` if a migration fails, otherwise `STO-601`.
    pub fn open_in_memory(retention: NonZeroU32) -> Result<Self, CodedError> {
        Connection::open_in_memory()
            .map_err(Into::into)
            .and_then(|conn| Self::prepared(conn, retention))
            .map_err(CodedError::from)
    }

    fn open_checked(path: &Path, retention: NonZeroU32) -> StorageResult<Self> {
        if let Some(folder) = path.parent() {
            fs::create_dir_all(folder)?;
        }
        Self::prepared(Connection::open(path)?, retention)
    }

    fn prepared(mut conn: Connection, retention: NonZeroU32) -> StorageResult<Self> {
        conn.busy_timeout(Duration::from_millis(BUSY_TIMEOUT_MS))?;
        conn.pragma_update(None, PRAGMA_FOREIGN_KEYS, PRAGMA_ON)?;
        conn.pragma_update(None, PRAGMA_JOURNAL_MODE, JOURNAL_MODE_WAL)?;
        conn.pragma_update(None, PRAGMA_SYNCHRONOUS, SYNCHRONOUS_NORMAL)?;
        conn.pragma_update(None, PRAGMA_CACHE_SIZE, CACHE_SIZE_KIB)?;
        conn.pragma_update(None, PRAGMA_TEMP_STORE, TEMP_STORE_MEMORY)?;
        migrate::apply(&mut conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
            retention,
            reverse: Mutex::new(None),
        })
    }

    /// The number of snapshots kept per project.
    pub(crate) const fn retention(&self) -> NonZeroU32 {
        self.retention
    }

    fn connection(&self) -> MutexGuard<'_, Connection> {
        self.conn.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Runs `work` on the connection without a transaction.
    pub(crate) fn read<T>(
        &self,
        work: impl FnOnce(&Connection) -> StorageResult<T>,
    ) -> StorageResult<T> {
        work(&self.connection())
    }

    /// Runs `work` in one write transaction: committed if it succeeds, rolled back otherwise.
    pub(crate) fn write<T>(
        &self,
        work: impl FnOnce(&Transaction<'_>) -> StorageResult<T>,
    ) -> StorageResult<T> {
        let mut conn = self.connection();
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let value = work(&tx)?;
        tx.commit()?;
        Ok(value)
    }

    /// The first id that is higher than every stored id, to start an `IdSequence` with.
    ///
    /// # Errors
    /// `STO-601` if the database cannot be read.
    pub fn next_free_id(&self) -> Result<u64, CodedError> {
        self.read(|conn| {
            let highest: Option<i64> = conn.query_row(sql::MAX_ID, [], |row| row.get(0))?;
            highest.map_or(Ok(FIRST_ID), |id| {
                Ok(codec::id_from(id)?.saturating_add(ID_STEP))
            })
        })
        .map_err(CodedError::from)
    }
}
