//! The `ProjectCatalog` port: the project list with what the newest snapshot says.

use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{
    LanguageShare, LeftOut, ProjectCatalog, ProjectOverview, ScanWarning,
};
use codeskimr_codes::ErrorCode;
use std::collections::HashMap;

use codeskimr_domain::{LanguageId, ProjectId, RelativePath, SnapshotId};
use rusqlite::{Connection, params};

use crate::codec::{column_int, id_from, rows_of_snapshot, small_from};
use crate::error::StorageResult;
use crate::headers::latest_snapshot;
use crate::projects::{load_project, load_project_any};
use crate::sql;
use crate::store::SqliteStore;

fn project_ids(conn: &Connection) -> StorageResult<Vec<ProjectId>> {
    let mut statement = conn.prepare_cached(sql::PROJECT_IDS)?;
    let rows = statement.query_map([], |row| row.get::<_, i64>(0))?;
    let mut ids = Vec::new();
    for row in rows {
        ids.push(ProjectId::new(id_from(row?)?));
    }
    Ok(ids)
}

fn languages(conn: &Connection, snapshot: SnapshotId) -> StorageResult<Vec<LanguageShare>> {
    // The symbols are counted in one pass over the symbols of the snapshot, not once per language.
    let symbols: HashMap<String, i64> =
        rows_of_snapshot(conn, sql::LANGUAGE_SYMBOLS, snapshot, |cols| {
            Ok((cols.take::<String>()?, cols.take::<i64>()?))
        })?
        .into_iter()
        .collect();
    let rows = rows_of_snapshot(conn, sql::LANGUAGE_SHARES, snapshot, |cols| {
        Ok((
            cols.take::<String>()?,
            cols.take::<i64>()?,
            cols.take::<i64>()?,
        ))
    })?;
    let times: HashMap<String, i64> = rows_of_snapshot(conn, sql::PARSE_TIMES, snapshot, |cols| {
        Ok((cols.take::<String>()?, cols.take::<i64>()?))
    })?
    .into_iter()
    .collect();
    let mut shares = Vec::new();
    for (language, files, loc) in rows {
        shares.push(LanguageShare {
            parse_micros: times
                .get(&language)
                .map(|micros| u64::try_from(*micros).unwrap_or_default()),
            symbols: small_from(symbols.get(&language).copied().unwrap_or_default())?,
            language: LanguageId::new(language)?,
            files: small_from(files)?,
            loc: small_from(loc)?,
        });
    }
    Ok(shares)
}

/// The warnings of the scan that wrote `snapshot`. A code that this version does not know any more
/// is left out.
pub(crate) fn warnings(conn: &Connection, snapshot: SnapshotId) -> StorageResult<Vec<ScanWarning>> {
    let rows = rows_of_snapshot(conn, sql::WARNINGS, snapshot, |cols| {
        Ok((cols.take::<String>()?, cols.take::<i64>()?))
    })?;
    Ok(rows
        .into_iter()
        .filter_map(|(id, count)| {
            let code = ErrorCode::ALL
                .iter()
                .copied()
                .find(|code| code.id() == id)?;
            Some(ScanWarning {
                code,
                count: u64::try_from(count).unwrap_or_default(),
            })
        })
        .collect())
}

/// The files the scan that wrote `snapshot` left out by name. A path or a code that this version
/// does not know any more is left out.
pub(crate) fn left_out(conn: &Connection, snapshot: SnapshotId) -> StorageResult<Vec<LeftOut>> {
    let rows = rows_of_snapshot(conn, sql::LEFT_OUT, snapshot, |cols| {
        Ok((
            cols.take::<String>()?,
            cols.take::<String>()?,
            cols.take::<bool>()?,
        ))
    })?;
    Ok(rows
        .into_iter()
        .filter_map(|(path, id, kept)| {
            let code = ErrorCode::ALL
                .iter()
                .copied()
                .find(|code| code.id() == id)?;
            let path = RelativePath::new(path).ok()?;
            Some(LeftOut { path, code, kept })
        })
        .collect())
}

fn overview(conn: &Connection, id: ProjectId) -> StorageResult<ProjectOverview> {
    let removing: i64 = conn.query_row(
        sql::PROJECT_REMOVING,
        params![column_int(id.get())?],
        |row| row.get(0),
    )?;
    if removing != 0 {
        return Ok(ProjectOverview {
            project: load_project_any(conn, id)?,
            latest: None,
            types: u32::default(),
            languages: Vec::new(),
            warnings: Vec::new(),
            left_out: Vec::new(),
            removing: true,
        });
    }
    let project = load_project(conn, id)?;
    let latest = latest_snapshot(conn, id)?;
    let (types, languages, (warnings, left_out)) = match &latest {
        Some(snapshot) => {
            let raw = column_int(snapshot.id.get())?;
            let types: i64 = conn.query_row(sql::TYPE_COUNT, params![raw], |row| row.get(0))?;
            (
                small_from(types)?,
                languages(conn, snapshot.id)?,
                (warnings(conn, snapshot.id)?, left_out(conn, snapshot.id)?),
            )
        }
        None => (u32::default(), Vec::new(), (Vec::new(), Vec::new())),
    };
    Ok(ProjectOverview {
        project,
        latest,
        types,
        languages,
        warnings,
        left_out,
        removing: false,
    })
}

fn languages_of(conn: &Connection, id: ProjectId) -> StorageResult<Vec<LanguageShare>> {
    load_project(conn, id)?;
    latest_snapshot(conn, id)?
        .map_or_else(|| Ok(Vec::new()), |snapshot| languages(conn, snapshot.id))
}

impl ProjectCatalog for SqliteStore {
    fn languages(&self, project: ProjectId) -> Result<Vec<LanguageShare>, CodedError> {
        self.read(|conn| languages_of(conn, project))
            .map_err(CodedError::from)
    }

    fn overviews(&self) -> Result<Vec<ProjectOverview>, CodedError> {
        // One read transaction, so that a scan committed meanwhile cannot split the list.
        self.read_apart(|conn| {
            project_ids(conn)?
                .into_iter()
                .map(|id| overview(conn, id))
                .collect()
        })
        .map_err(CodedError::from)
    }
}
