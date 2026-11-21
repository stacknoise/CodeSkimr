//! Removing a project: the `ProjectRemoval` port.

use codeskimr_application::error::CodedError;
use codeskimr_application::ports::ProjectRemoval;
use codeskimr_domain::ProjectId;
use rusqlite::params;

use crate::codec::column_int;
use crate::error::StorageError;
use crate::snapshots::delete_snapshot;
use crate::sql;
use crate::store::SqliteStore;

impl ProjectRemoval for SqliteStore {
    fn remove_project(&self, id: ProjectId) -> Result<(), CodedError> {
        self.write(|tx| {
            let project = column_int(id.get())?;
            let known: i64 =
                tx.query_row(sql::PROJECT_EXISTS, params![project], |row| row.get(0))?;
            if known == 0 {
                return Err(StorageError::ProjectNotFound);
            }
            let snapshots = tx
                .prepare_cached(sql::SNAPSHOTS_OF_PROJECT)?
                .query_map(params![project], |row| row.get::<_, i64>(0))?
                .collect::<Result<Vec<_>, _>>()?;
            for snapshot in snapshots {
                delete_snapshot(tx, snapshot)?;
            }
            tx.execute(sql::PROJECT_DELETE, params![project])?;
            Ok(())
        })
        .map_err(CodedError::from)
    }
}
