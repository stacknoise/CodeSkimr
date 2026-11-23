//! The `SQLite` connection and the unit of work.

use std::fs;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use codeskimr_application::error::CodedError;
use codeskimr_domain::SnapshotId;
use rusqlite::{Connection, ErrorCode, OpenFlags, Transaction, TransactionBehavior};

use crate::consts::{
    BUSY_TIMEOUT_MS, CACHE_SIZE_KIB, FIRST_ID, ID_STEP, JOURNAL_MODE_WAL,
    LOG_READ_CONNECTION_UNAVAILABLE, PRAGMA_CACHE_SIZE, PRAGMA_FOREIGN_KEYS, PRAGMA_JOURNAL_MODE,
    PRAGMA_MAX_PAGE_COUNT, PRAGMA_ON, PRAGMA_PAGE_COUNT, PRAGMA_SYNCHRONOUS, PRAGMA_TEMP_STORE,
    SYNCHRONOUS_NORMAL, TEMP_STORE_MEMORY, WAL_SWITCH_ATTEMPTS, WAL_SWITCH_PAUSE_MS,
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
    /// The database file, so that a long read can use a connection of its own and leave `conn`
    /// to the queries of the screens; `None` for an in-memory database.
    file: Option<PathBuf>,
    retention: NonZeroU32,
    /// The reverse indexes and the places of the snapshots that were asked about last, the
    /// newest first, at most `KEPT_LOOKUPS`: a small project that is opened next to a large one
    /// does not push it out, and a build of one does not make the other wait. They are kept only
    /// while no other connection has committed since (`data_version`): the level 2 import of a
    /// scan from the browser or of `codeskimr scan` replaces the references of a snapshot through
    /// its own store. The lock is held for a look into the list, never during a build.
    pub(crate) reverse: Mutex<Vec<KeptLookup>>,
    /// One lock per snapshot whose index is built: a second caller for the same snapshot waits
    /// for the first and finds its result; callers for other snapshots do not wait.
    pub(crate) builds: Mutex<std::collections::HashMap<i64, Arc<Mutex<()>>>>,
    /// The facts of the symbols for the statistics, marked with the test folders they were
    /// built for; kept for the lookup they were built from and dropped with it.
    pub(crate) facts: Mutex<Option<KeptFacts>>,
}

/// What [`SqliteStore`] keeps for the statistics: the lookup the usage counts come from, the
/// names of the test folders, and the facts.
pub(crate) type KeptFacts = (
    std::sync::Weak<crate::lookup::SnapshotLookup>,
    Vec<String>,
    Arc<Vec<codeskimr_domain::SymbolFacts>>,
);

/// What [`SqliteStore`] keeps of the reverse index: the snapshot, its lookup and the
/// `data_version` of the database when it was built.
pub(crate) type KeptLookup = (SnapshotId, Arc<crate::lookup::SnapshotLookup>, i64);

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
            .and_then(|conn| Self::prepared(conn, None, retention, StoreOptions::default()))
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
        Self::prepared(
            Connection::open(path)?,
            Some(path.to_path_buf()),
            retention,
            options,
        )
    }

    fn prepared(
        mut conn: Connection,
        file: Option<PathBuf>,
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
            file,
            retention,
            reverse: Mutex::new(Vec::new()),
            builds: Mutex::new(std::collections::HashMap::new()),
            facts: Mutex::new(None),
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

    /// Runs a long read on a read-only connection of its own, so that the other queries and the
    /// writers are not held up by it (the write-ahead log lets readers run side by side). The
    /// whole read is one read transaction, so it sees one consistent state even if a writer
    /// commits meanwhile. While it runs, the log cannot be checkpointed past its start, so the
    /// log file may grow until it ends. Without a file (in-memory database) it runs on the shared
    /// connection; so it does if the extra connection cannot be opened, which is logged.
    pub(crate) fn read_apart<T>(
        &self,
        work: impl FnOnce(&Connection) -> StorageResult<T>,
    ) -> StorageResult<T> {
        let Some(file) = self.file.as_deref() else {
            return self.read(work);
        };
        let opened = Connection::open_with_flags(
            file,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        );
        match opened {
            Ok(conn) => {
                let tx = conn.unchecked_transaction()?;
                let value = work(&tx)?;
                tx.commit()?;
                Ok(value)
            }
            Err(cause) => {
                tracing::warn!(event = LOG_READ_CONNECTION_UNAVAILABLE, %cause);
                self.read(work)
            }
        }
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

#[cfg(test)]
mod tests {
    use std::num::NonZeroU32;

    use super::SqliteStore;

    fn file_store(name: &str) -> (SqliteStore, std::path::PathBuf) {
        let folder =
            std::env::temp_dir().join(format!("codeskimr-apart-{}-{name}", std::process::id()));
        let path = folder.join("workspace.db");
        let store = SqliteStore::open(&path, NonZeroU32::new(3).unwrap()).unwrap();
        (store, folder)
    }

    #[test]
    fn a_read_apart_does_not_hold_up_a_write_on_the_shared_connection() {
        let (store, folder) = file_store("write");
        let written = store
            .read_apart(|_| store.write(|tx| Ok(tx.execute("CREATE TABLE probe (x)", [])?)))
            .unwrap();
        assert_eq!(written, 0);
        std::fs::remove_dir_all(folder).unwrap();
    }

    #[test]
    fn a_read_apart_sees_one_state_although_a_writer_commits_meanwhile() {
        let (store, folder) = file_store("state");
        store
            .write(|tx| {
                Ok(tx.execute_batch("CREATE TABLE probe (x); INSERT INTO probe VALUES (1);")?)
            })
            .unwrap();
        let (before, after) = store
            .read_apart(|conn| {
                let count = |c: &rusqlite::Connection| -> i64 {
                    c.query_row("SELECT COUNT(*) FROM probe", [], |row| row.get(0))
                        .unwrap()
                };
                let before = count(conn);
                store.write(|tx| Ok(tx.execute("INSERT INTO probe VALUES (2)", [])?))?;
                Ok((before, count(conn)))
            })
            .unwrap();
        assert_eq!((before, after), (1, 1));
        assert_eq!(
            store
                .read(
                    |c| Ok(c.query_row("SELECT COUNT(*) FROM probe", [], |r| r.get::<_, i64>(0))?)
                )
                .unwrap(),
            2
        );
        std::fs::remove_dir_all(folder).unwrap();
    }
}
