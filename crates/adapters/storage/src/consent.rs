//! The user's decision about level 2 for a project: the `ConsentStore` port (ADR 0033).

use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{ConsentRecord, ConsentStore};
use codeskimr_domain::ProjectId;
use rusqlite::{OptionalExtension, params};

use crate::codec::column_int;
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

    fn store_consent(&self, project: ProjectId, record: ConsentRecord) -> Result<(), CodedError> {
        self.write(|tx| {
            tx.execute(
                sql::CONSENT_UPSERT,
                params![column_int(project.get())?, record.enabled, record.confirmed],
            )?;
            Ok(())
        })
        .map_err(CodedError::from)
    }
}
