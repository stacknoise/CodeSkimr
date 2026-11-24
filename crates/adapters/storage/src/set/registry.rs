//! The catalog: the list of the projects and the identity of the installation (ADR 0095).
//!
//! It holds nothing of a project but its number, its source and the mark of a removal in
//! progress; the project itself lives in its own file. It stays small and is written in short
//! transactions only.

use std::fs;
use std::path::Path;
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::{SystemTime, UNIX_EPOCH};

use codeskimr_application::ids::is_project_number;
use codeskimr_domain::ProjectId;
use rusqlite::{Connection, OptionalExtension, params};

use super::consts::APPLICATION_ID;
use super::consts::{
    CHECK_OK, HIGHEST_NUMBER_EVER, INSTALLATION_ID, INSTALLATION_INSERT, INSTALLATION_SEPARATOR,
    PRAGMA_APPLICATION_ID, PROJECT_BY_SOURCE, PROJECT_DELETE, PROJECT_INSERT, PROJECT_INSERT_AS,
    PROJECT_MARK_REMOVING, PROJECT_NUMBERS, PROJECT_REMOVING, PROJECT_SET_SOURCE, PROJECT_SOURCE,
    QUICK_CHECK, REGISTRY_SCHEMA,
};
use crate::codec::{Cols, column_int, id_from};
use crate::consts::{BUSY_TIMEOUT_MS, JOURNAL_MODE_WAL, NO_CREDENTIAL, PRAGMA_JOURNAL_MODE};
use crate::error::{StorageError, StorageResult};
use crate::projects::SourceColumns;
use codeskimr_domain::ProjectSource;
use std::time::Duration;

/// A project of the catalog: its number and whether its removal has begun.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Entry {
    pub(crate) project: ProjectId,
    pub(crate) removing: bool,
}

/// The catalog database.
#[derive(Debug)]
pub(crate) struct Registry {
    conn: Mutex<Connection>,
    installation: String,
}

impl Registry {
    /// Opens the catalog at `path`, creating it with the identity of this installation.
    pub(crate) fn open(path: &Path) -> StorageResult<Self> {
        if let Some(folder) = path.parent() {
            fs::create_dir_all(folder)?;
        }
        Self::prepared(Connection::open(path)?)
    }

    /// A catalog in memory, for tests.
    pub(crate) fn in_memory() -> StorageResult<Self> {
        Self::prepared(Connection::open_in_memory()?)
    }

    fn prepared(conn: Connection) -> StorageResult<Self> {
        conn.busy_timeout(Duration::from_millis(BUSY_TIMEOUT_MS))?;
        // The mode is answered with a row, which `pragma_update` reads and drops.
        let _ = conn.pragma_update(None, PRAGMA_JOURNAL_MODE, JOURNAL_MODE_WAL);
        conn.pragma_update(None, PRAGMA_APPLICATION_ID, APPLICATION_ID)?;
        conn.execute_batch(REGISTRY_SCHEMA)?;
        let known: Option<String> = conn
            .query_row(INSTALLATION_ID, [], |row| row.get(0))
            .optional()?;
        let installation = if let Some(id) = known {
            id
        } else {
            let id = new_installation_id();
            conn.execute(INSTALLATION_INSERT, params![id])?;
            id
        };
        Ok(Self {
            conn: Mutex::new(conn),
            installation,
        })
    }

    fn conn(&self) -> MutexGuard<'_, Connection> {
        self.conn.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The identity of this installation, written into every project file.
    pub(crate) fn installation(&self) -> &str {
        &self.installation
    }

    /// The project of `source`, if it is registered.
    pub(crate) fn find(&self, source: &ProjectSource) -> StorageResult<Option<Entry>> {
        let columns = SourceColumns::of(source);
        let found: Option<(i64, i64)> = self
            .conn()
            .query_row(
                PROJECT_BY_SOURCE,
                params![columns.kind, columns.location, columns.branch],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        found
            .map(|(number, removing)| entry(number, removing))
            .transpose()
    }

    /// Registers `source` and returns its new number. A number that ids cannot carry is refused.
    pub(crate) fn register(&self, source: &ProjectSource) -> StorageResult<ProjectId> {
        let columns = SourceColumns::of(source);
        let conn = self.conn();
        conn.execute(
            PROJECT_INSERT,
            params![columns.kind, columns.location, columns.branch],
        )?;
        let project = ProjectId::new(id_from(conn.last_insert_rowid())?);
        if is_project_number(project) {
            Ok(project)
        } else {
            conn.execute(PROJECT_DELETE, params![column_int(project.get())?])?;
            Err(StorageError::OutOfRange)
        }
    }

    /// Registers `source` under a number the caller chose, for a project that is saved with its
    /// id before the catalog knows it. A number that ids cannot carry is refused.
    pub(crate) fn register_as(
        &self,
        project: ProjectId,
        source: &ProjectSource,
    ) -> StorageResult<()> {
        if !is_project_number(project) {
            return Err(StorageError::OutOfRange);
        }
        let columns = SourceColumns::of(source);
        self.conn().execute(
            PROJECT_INSERT_AS,
            params![
                column_int(project.get())?,
                columns.kind,
                columns.location,
                columns.branch
            ],
        )?;
        Ok(())
    }

    /// Every registered project, in the order of registration.
    pub(crate) fn entries(&self) -> StorageResult<Vec<Entry>> {
        let conn = self.conn();
        let mut statement = conn.prepare_cached(PROJECT_NUMBERS)?;
        let rows =
            statement.query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)))?;
        let mut all = Vec::new();
        for row in rows {
            let (number, removing) = row?;
            all.push(entry(number, removing)?);
        }
        Ok(all)
    }

    /// Whether the removal of `project` has begun; `None` for a project that is not registered.
    pub(crate) fn removing(&self, project: ProjectId) -> StorageResult<Option<bool>> {
        let mark: Option<i64> = self
            .conn()
            .query_row(
                PROJECT_REMOVING,
                params![column_int(project.get())?],
                |row| row.get(0),
            )
            .optional()?;
        Ok(mark.map(|mark| mark != 0))
    }

    /// The source the catalog holds for `project`, without a credential (the catalog keeps none);
    /// it is the index copy, the project file holds the one that counts (ADR 0095).
    pub(crate) fn source_of(&self, project: ProjectId) -> StorageResult<ProjectSource> {
        let (kind, location, branch): (String, String, String) = self
            .conn()
            .query_row(PROJECT_SOURCE, params![column_int(project.get())?], |row| {
                let cols = Cols::new(row);
                Ok((cols.take()?, cols.take()?, cols.take()?))
            })
            .optional()?
            .ok_or(StorageError::ProjectNotFound)?;
        SourceColumns::source(&kind, location, branch, NO_CREDENTIAL.to_owned())
    }

    /// Marks `project` as being removed.
    pub(crate) fn mark_removing(&self, project: ProjectId) -> StorageResult<()> {
        self.conn()
            .execute(PROJECT_MARK_REMOVING, params![column_int(project.get())?])?;
        Ok(())
    }

    /// Gives `project` another source.
    pub(crate) fn set_source(
        &self,
        project: ProjectId,
        source: &ProjectSource,
    ) -> StorageResult<()> {
        let columns = SourceColumns::of(source);
        self.conn().execute(
            PROJECT_SET_SOURCE,
            params![
                columns.kind,
                columns.location,
                columns.branch,
                column_int(project.get())?
            ],
        )?;
        Ok(())
    }

    /// Removes the row of `project`.
    pub(crate) fn delete(&self, project: ProjectId) -> StorageResult<()> {
        self.conn()
            .execute(PROJECT_DELETE, params![column_int(project.get())?])?;
        Ok(())
    }

    /// The highest project number ever handed out; the counter survives the removal of rows.
    pub(crate) fn highest_number_ever(&self) -> StorageResult<u64> {
        let seq: Option<i64> = self
            .conn()
            .query_row(HIGHEST_NUMBER_EVER, [], |row| row.get(0))
            .optional()?;
        seq.map_or(Ok(0), id_from)
    }

    /// Whether the catalog passes the quick check of `SQLite`.
    pub(crate) fn intact(&self) -> bool {
        self.conn()
            .query_row(QUICK_CHECK, [], |row| row.get::<_, String>(0))
            .is_ok_and(|answer| answer == CHECK_OK)
    }
}

fn entry(number: i64, removing: i64) -> StorageResult<Entry> {
    Ok(Entry {
        project: ProjectId::new(id_from(number)?),
        removing: removing != 0,
    })
}

/// A name for this installation: the time of its first start and the process that made it. It
/// tells installations apart; it is not a secret.
fn new_installation_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_nanos());
    format!("{nanos:x}{INSTALLATION_SEPARATOR}{:x}", std::process::id())
}
