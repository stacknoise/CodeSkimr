//! The user's decision about level 2 for a project: the `ConsentStore` port (ADR 0033).

use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{ConsentGrant, ConsentRecord, ConsentStore};
use codeskimr_domain::ProjectId;
use rusqlite::{OptionalExtension, params};

use crate::codec::column_int;
use crate::error::StorageError;
use crate::sql;
use crate::store::SqliteStore;

impl ConsentStore for SqliteStore {
    fn consent(&self, project: ProjectId) -> Result<ConsentRecord, CodedError> {
        self.read(|conn| {
            let row: Option<(bool, bool)> = conn
                .query_row(
                    sql::CONSENT_BY_PROJECT,
                    params![column_int(project.get())?],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .optional()?;
            Ok(
                row.map_or_else(ConsentRecord::default, |(enabled, confirmed)| {
                    ConsentRecord { enabled, confirmed }
                }),
            )
        })
        .map_err(CodedError::from)
    }

    fn store_consent(&self, grant: &ConsentGrant) -> Result<(), CodedError> {
        let (project, record) = (grant.project(), grant.record());
        self.write(|tx| {
            let written = tx.execute(
                sql::CONSENT_UPSERT,
                params![column_int(project.get())?, record.enabled, record.confirmed],
            )?;
            // Nothing is written for a project that does not exist, and that is not a success.
            if written == 0 {
                return Err(StorageError::ProjectNotFound);
            }
            Ok(())
        })
        .map_err(CodedError::from)
    }
}
