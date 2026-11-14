//! The `SQLite` connection and the unit of work.

use std::fs;
use std::num::NonZeroU32;
use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use codeskimr_application::error::CodedError;
use codeskimr_domain::SnapshotId;
use rusqlite::{Connection, ErrorCode, Transaction, TransactionBehavior};

use crate::consts::{
    BUSY_TIMEOUT_MS, CACHE_SIZE_KIB, FIRST_ID, ID_STEP, JOURNAL_MODE_WAL, PRAGMA_CACHE_SIZE,
    PRAGMA_FOREIGN_KEYS, PRAGMA_JOURNAL_MODE, PRAGMA_MAX_PAGE_COUNT, PRAGMA_ON, PRAGMA_PAGE_COUNT,
    PRAGMA_SYNCHRONOUS, PRAGMA_TEMP_STORE, SYNCHRONOUS_NORMAL, TEMP_STORE_MEMORY,
    WAL_SWITCH_ATTEMPTS, WAL_SWITCH_PAUSE_MS,
};
use crate::error::StorageResult;
use crate::sql;
use crate::{codec, migrate};

/// Connection settings that the product leaves at their defaults. Tests and tools change them, for
/// example to reach the lock-conflict and disk-full errors quickly.
#[derive(Debug, Clone, Copy)]
pub struct StoreOptions {
    /// How long a statement waits for a lock held by another connection before it fails.
    pub busy_timeout: Duration,
    /// How many pages the file may grow by after the migrations; `None` means no limit.
    pub growth_limit_pages: Option<NonZeroU32>,
}

impl Default for StoreOptions {
    fn default() -> Self {
        Self {
            busy_timeout: Duration::from_millis(BUSY_TIMEOUT_MS),
            growth_limit_pages: None,
        }
    }
}

/// Caps the file at its current size plus `pages`; a write beyond that fails like a full disk.
fn limit_growth(conn: &Connection, pages: NonZeroU32) -> rusqlite::Result<()> {
    let used: u32 = conn.pragma_query_value(None, PRAGMA_PAGE_COUNT, |row| row.get(0))?;
    conn.pragma_update(
        None,
        PRAGMA_MAX_PAGE_COUNT,
        used.saturating_add(pages.get()),
    )
}

/// The `SQLite` database of one workspace. It implements the storage ports and offers the queries
/// on the read models. Share it behind an `Arc`; one writer at a time, readers wait for it.
#[derive(Debug)]
pub struct SqliteStore {
    conn: Mutex<Connection>,
    retention: NonZeroU32,
    /// The reverse index and the places of the snapshot that was asked about last; a snapshot
    /// never changes.
    pub(crate) reverse: Mutex<Option<(SnapshotId, Arc<crate::lookup::SnapshotLookup>)>>,
}

impl SqliteStore {
    /// Opens the database file, creating it and its folder if needed, and applies the migrations.
    /// `retention` is the number of snapshots kept per project (`snapshots.retention_count`).
    ///
    /// # Errors
    /// `STO-301` if a migration fails, `STO-501` if the disk is full, otherwise `STO-601`.
    pub fn open(path: &Path, retention: NonZeroU32) -> Result<Self, CodedError> {
        Self::open_with(path, retention, StoreOptions::default())
    }

    /// Like [`SqliteStore::open`], with other connection settings than the defaults.
    ///
    /// # Errors
    /// `STO-301` if a migration fails, `STO-501` if the disk is full, otherwise `STO-601`.
    pub fn open_with(
        path: &Path,
        retention: NonZeroU32,
        options: StoreOptions,
    ) -> Result<Self, CodedError> {
        Self::open_checked(path, retention, options).map_err(CodedError::from)
    }

    /// Opens a private in-memory database, for tests and short-lived tools.
    ///
    /// # Errors
    /// `STO-301` if a migration fails, otherwise `STO-601`.
    pub fn open_in_memory(retention: NonZeroU32) -> Result<Self, CodedError> {
        Connection::open_in_memory()
            .map_err(Into::into)
            .and_then(|conn| Self::prepared(conn, retention, StoreOptions::default()))
            .map_err(CodedError::from)
    }

    fn open_checked(
        path: &Path,
        retention: NonZeroU32,
        options: StoreOptions,
    ) -> StorageResult<Self> {
        if let Some(folder) = path.parent() {
            fs::create_dir_all(folder)?;
        }
        Self::prepared(Connection::open(path)?, retention, options)
    }

    fn prepared(
        mut conn: Connection,
        retention: NonZeroU32,
        options: StoreOptions,
    ) -> StorageResult<Self> {
        conn.busy_timeout(options.busy_timeout)?;
        conn.pragma_update(None, PRAGMA_FOREIGN_KEYS, PRAGMA_ON)?;
        enable_wal(&conn)?;
        conn.pragma_update(None, PRAGMA_SYNCHRONOUS, SYNCHRONOUS_NORMAL)?;
        conn.pragma_update(None, PRAGMA_CACHE_SIZE, CACHE_SIZE_KIB)?;
        conn.pragma_update(None, PRAGMA_TEMP_STORE, TEMP_STORE_MEMORY)?;
        migrate::apply(&mut conn)?;
        if let Some(pages) = options.growth_limit_pages {
            limit_growth(&conn, pages)?;
        }
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

/// Switches the file to the write-ahead log. While another connection is switching or migrating
/// the same new file, `SQLite` answers "busy" at once, so the switch is tried again for a while.
fn enable_wal(conn: &Connection) -> rusqlite::Result<()> {
    let mut attempt = 1;
    loop {
        match conn.pragma_update(None, PRAGMA_JOURNAL_MODE, JOURNAL_MODE_WAL) {
            Err(rusqlite::Error::SqliteFailure(failure, _))
                if matches!(
                    failure.code,
                    ErrorCode::DatabaseBusy | ErrorCode::DatabaseLocked
                ) && attempt < WAL_SWITCH_ATTEMPTS =>
            {
                attempt += 1;
                std::thread::sleep(Duration::from_millis(WAL_SWITCH_PAUSE_MS));
            }
            other => return other,
        }
    }
}
