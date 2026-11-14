//! The `ProjectCatalog` port: the project list with what the newest snapshot says.

use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{LanguageShare, ProjectCatalog, ProjectOverview};
use std::collections::HashMap;

use codeskimr_domain::{LanguageId, ProjectId, SnapshotId};
use rusqlite::{Connection, params};

use crate::codec::{Cols, column_int, id_from, small_from};
use crate::error::StorageResult;
use crate::projects::load_project;
use crate::snapshots::latest_snapshot;
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

/// Runs a statement of one snapshot and reads every row with `read`.
fn rows_of_snapshot<T>(
    conn: &Connection,
    statement: &str,
    snapshot: SnapshotId,
    read: impl Fn(&Cols<'_>) -> rusqlite::Result<T>,
) -> StorageResult<Vec<T>> {
    let mut prepared = conn.prepare_cached(statement)?;
    let rows = prepared.query_map(params![column_int(snapshot.get())?], |row| {
        read(&Cols::new(row))
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
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
    let mut shares = Vec::new();
    for (language, files, loc) in rows {
        shares.push(LanguageShare {
            symbols: small_from(symbols.get(&language).copied().unwrap_or_default())?,
            language: LanguageId::new(language)?,
            files: small_from(files)?,
            loc: small_from(loc)?,
        });
    }
    Ok(shares)
}

fn overview(conn: &Connection, id: ProjectId) -> StorageResult<ProjectOverview> {
    let project = load_project(conn, id)?;
    let latest = latest_snapshot(conn, id)?;
    let (types, languages) = match &latest {
        Some(snapshot) => {
            let raw = column_int(snapshot.id.get())?;
            let types: i64 = conn.query_row(sql::TYPE_COUNT, params![raw], |row| row.get(0))?;
            (small_from(types)?, languages(conn, snapshot.id)?)
        }
        None => (u32::default(), Vec::new()),
    };
    Ok(ProjectOverview {
        project,
        latest,
        types,
        languages,
    })
}

impl ProjectCatalog for SqliteStore {
    fn overviews(&self) -> Result<Vec<ProjectOverview>, CodedError> {
        self.read(|conn| {
            project_ids(conn)?
                .into_iter()
                .map(|id| overview(conn, id))
                .collect()
        })
        .map_err(CodedError::from)
    }
}
