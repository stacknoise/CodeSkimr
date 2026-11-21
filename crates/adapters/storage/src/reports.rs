//! The `ScanReportStore` port: what the newest scan of a project says about itself (ADR 0072).

use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{
    LanguageLine, LanguageOutcome, LastScan, Level2Report, ReportWrite, ScanReport, ScanReportStore,
};
use codeskimr_codes::ErrorCode;
use codeskimr_domain::{LanguageId, OutcomeKind, ProjectId, Timestamp};
use rusqlite::{Connection, OptionalExtension, Transaction, params};

use crate::codec::{Cols, column_int, id_from, label_from};
use crate::error::{StorageError, StorageResult};
use crate::sql;
use crate::store::SqliteStore;

/// The columns of a language line as stored.
struct LineColumns {
    outcome: OutcomeKind,
    files: Option<i64>,
    stale: Option<i64>,
    reason: Option<&'static str>,
    code: Option<&'static str>,
}

fn columns_of(outcome: &LanguageOutcome) -> StorageResult<LineColumns> {
    let none = LineColumns {
        outcome: OutcomeKind::Skipped,
        files: None,
        stale: None,
        reason: None,
        code: None,
    };
    Ok(match outcome {
        LanguageOutcome::Checked { files, stale } => LineColumns {
            outcome: OutcomeKind::Checked,
            files: Some(column_int(*files)?),
            stale: Some(column_int(*stale)?),
            ..none
        },
        LanguageOutcome::Skipped(reason) => LineColumns {
            reason: Some(reason.label()),
            ..none
        },
        LanguageOutcome::Failed(code) => LineColumns {
            outcome: OutcomeKind::Failed,
            code: Some(code.id()),
            ..none
        },
    })
}

fn insert_languages(tx: &Transaction<'_>, id: i64, lines: &[LanguageLine]) -> StorageResult<()> {
    tx.execute(sql::LEVEL2_LANGUAGES_DELETE, params![id])?;
    let mut statement = tx.prepare_cached(sql::LEVEL2_LANGUAGE_INSERT)?;
    for line in lines {
        let columns = columns_of(&line.outcome)?;
        statement.execute(params![
            id,
            line.language.as_str(),
            columns.outcome.label(),
            columns.files,
            columns.stale,
            columns.reason,
            columns.code,
        ])?;
    }
    Ok(())
}

fn store(tx: &Transaction<'_>, write: &ReportWrite) -> StorageResult<()> {
    let id = column_int(write.project.get())?;
    let written = tx.execute(
        sql::SCAN_REPORT_UPSERT,
        params![
            id,
            write.last.at.0,
            write.last.level.label(),
            write.last.origin.label()
        ],
    )?;
    if written == 0 {
        return Err(StorageError::ProjectNotFound);
    }
    if let Some(level2) = &write.level2 {
        tx.execute(
            sql::LEVEL2_REPORT_UPSERT,
            params![id, level2.at.0, level2.origin.label()],
        )?;
        insert_languages(tx, id, &level2.languages)?;
    }
    Ok(())
}

fn line_of(cols: &Cols<'_>) -> StorageResult<Option<LanguageLine>> {
    let language: String = cols.take()?;
    let outcome: String = cols.take()?;
    let (files, stale): (Option<i64>, Option<i64>) = (cols.take()?, cols.take()?);
    let (reason, code): (Option<String>, Option<String>) = (cols.take()?, cols.take()?);
    let outcome = match label_from::<OutcomeKind>(&outcome)? {
        OutcomeKind::Checked => LanguageOutcome::Checked {
            files: id_from(files.unwrap_or_default())?,
            stale: id_from(stale.unwrap_or_default())?,
        },
        OutcomeKind::Failed => {
            // A code that this version does not know any more leaves the line out.
            let Some(known) = ErrorCode::ALL
                .iter()
                .copied()
                .find(|known| Some(known.id()) == code.as_deref())
            else {
                return Ok(None);
            };
            LanguageOutcome::Failed(known)
        }
        OutcomeKind::Skipped => LanguageOutcome::Skipped(label_from(&reason.unwrap_or_default())?),
    };
    Ok(Some(LanguageLine {
        language: LanguageId::new(language)?,
        outcome,
    }))
}

fn languages(conn: &Connection, id: i64) -> StorageResult<Vec<LanguageLine>> {
    let mut statement = conn.prepare_cached(sql::LEVEL2_LANGUAGES_BY_PROJECT)?;
    let lines = statement
        .query_map(params![id], |row| Ok(line_of(&Cols::new(row))))?
        .collect::<Result<Vec<_>, _>>()?;
    let mut found = Vec::new();
    for line in lines {
        found.extend(line?);
    }
    Ok(found)
}

fn level2_of(conn: &Connection, id: i64) -> StorageResult<Option<Level2Report>> {
    let header: Option<(i64, String)> = conn
        .query_row(sql::LEVEL2_REPORT_BY_PROJECT, params![id], |row| {
            let cols = Cols::new(row);
            Ok((cols.take()?, cols.take()?))
        })
        .optional()?;
    header
        .map(|(at, origin)| {
            Ok(Level2Report {
                at: Timestamp(at),
                origin: label_from(&origin)?,
                languages: languages(conn, id)?,
            })
        })
        .transpose()
}

fn report_of(conn: &Connection, project: ProjectId) -> StorageResult<Option<ScanReport>> {
    let id = column_int(project.get())?;
    let header: Option<(i64, String, String)> = conn
        .query_row(sql::SCAN_REPORT_BY_PROJECT, params![id], |row| {
            let cols = Cols::new(row);
            Ok((cols.take()?, cols.take()?, cols.take()?))
        })
        .optional()?;
    header
        .map(|(at, level, origin)| {
            Ok(ScanReport {
                last: LastScan {
                    at: Timestamp(at),
                    level: label_from(&level)?,
                    origin: label_from(&origin)?,
                },
                level2: level2_of(conn, id)?,
            })
        })
        .transpose()
}

impl ScanReportStore for SqliteStore {
    fn store_report(&self, write: &ReportWrite) -> Result<(), CodedError> {
        self.write(|tx| store(tx, write)).map_err(CodedError::from)
    }

    fn report(&self, project: ProjectId) -> Result<Option<ScanReport>, CodedError> {
        self.read(|conn| report_of(conn, project))
            .map_err(CodedError::from)
    }
}
