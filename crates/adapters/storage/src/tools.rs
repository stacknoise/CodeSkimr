//! The `ToolReportStore` port: the newest tool check of each project.

use codeskimr_application::error::CodedError;
use codeskimr_application::ports::ToolReportStore;
use codeskimr_domain::{LanguageId, ProjectId, Timestamp, ToolFinding, ToolReport};
use rusqlite::{Connection, OptionalExtension, Transaction, params};

use crate::codec::{Cols, column_int, label_from, ordinals};
use crate::error::StorageResult;
use crate::sql;
use crate::store::SqliteStore;

fn insert(tx: &Transaction<'_>, project: ProjectId, report: &ToolReport) -> StorageResult<()> {
    let id = column_int(project.get())?;
    tx.execute(sql::TOOL_REPORT_DELETE, params![id])?;
    tx.execute(sql::TOOL_REPORT_INSERT, params![id, report.checked_at.0])?;
    let mut statement = tx.prepare_cached(sql::TOOL_FINDING_INSERT)?;
    for (position, finding) in ordinals().zip(&report.findings) {
        statement.execute(params![
            id,
            position,
            finding.language.as_str(),
            finding.tool,
            finding.role.label(),
            finding.state.label(),
            finding.found,
            finding.required,
            finding.purpose_key,
            finding.install_key,
        ])?;
    }
    Ok(())
}

fn finding(row: &rusqlite::Row<'_>) -> rusqlite::Result<RawFinding> {
    let cols = Cols::new(row);
    Ok(RawFinding {
        language: cols.take()?,
        tool: cols.take()?,
        role: cols.take()?,
        state: cols.take()?,
        found: cols.take()?,
        required: cols.take()?,
        purpose_key: cols.take()?,
        install_key: cols.take()?,
    })
}

/// A finding as the columns hold it, before the labels are checked.
struct RawFinding {
    language: String,
    tool: String,
    role: String,
    state: String,
    found: Option<String>,
    required: Option<String>,
    purpose_key: String,
    install_key: String,
}

impl RawFinding {
    fn finding(self) -> StorageResult<ToolFinding> {
        Ok(ToolFinding {
            language: LanguageId::new(self.language)?,
            tool: self.tool,
            role: label_from(&self.role)?,
            state: label_from(&self.state)?,
            found: self.found,
            required: self.required,
            purpose_key: self.purpose_key,
            install_key: self.install_key,
        })
    }
}

fn load(conn: &Connection, project: ProjectId) -> StorageResult<Option<ToolReport>> {
    let id = column_int(project.get())?;
    let checked: Option<i64> = conn
        .query_row(sql::TOOL_REPORT_TIME, params![id], |row| row.get(0))
        .optional()?;
    let Some(checked_at) = checked else {
        return Ok(None);
    };
    let mut statement = conn.prepare_cached(sql::TOOL_FINDINGS)?;
    let rows = statement.query_map(params![id], finding)?;
    let mut findings = Vec::new();
    for row in rows {
        findings.push(row?.finding()?);
    }
    Ok(Some(ToolReport {
        checked_at: Timestamp(checked_at),
        findings,
    }))
}

impl ToolReportStore for SqliteStore {
    fn replace(&self, project: ProjectId, report: &ToolReport) -> Result<(), CodedError> {
        self.write(|tx| insert(tx, project, report))
            .map_err(CodedError::from)
    }

    fn load(&self, project: ProjectId) -> Result<Option<ToolReport>, CodedError> {
        self.read(|conn| load(conn, project))
            .map_err(CodedError::from)
    }
}
