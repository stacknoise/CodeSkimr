//! What a project leaves out of its scans: the `ScopeStore` port and the reading side of
//! `ProjectRepository::scope` (ADR 0101).

use codeskimr_application::error::CodedError;
use codeskimr_application::ports::ScopeStore;
use codeskimr_domain::{LanguageId, ProjectId, RelativePath, Scope};
use rusqlite::{Connection, Transaction, params};

use crate::codec::column_int;
use crate::error::StorageResult;
use crate::projects::load_project;
use crate::sql;
use crate::store::ProjectDb;

impl ProjectDb {
    /// The scope of the project; `PRJ-201` if it does not exist.
    pub(crate) fn stored_scope(&self, id: ProjectId) -> Result<Scope, CodedError> {
        self.read(|conn| {
            load_project(conn, id)?;
            load_scope(conn, id)
        })
        .map_err(CodedError::from)
    }
}

impl ScopeStore for ProjectDb {
    fn store_scope(&self, project: ProjectId, scope: &Scope) -> Result<(), CodedError> {
        self.write(|tx| {
            load_project(tx, project)?;
            replace_scope(tx, project, scope)
        })
        .map_err(CodedError::from)
    }
}

fn load_scope(conn: &Connection, project: ProjectId) -> StorageResult<Scope> {
    let id = column_int(project.get())?;
    let mut languages = Vec::new();
    let mut statement = conn.prepare_cached(sql::SCOPE_LANGUAGES)?;
    for name in statement.query_map(params![id], |row| row.get::<_, String>(0))? {
        languages.push(LanguageId::new(name?)?);
    }
    let mut folders = Vec::new();
    let mut statement = conn.prepare_cached(sql::SCOPE_FOLDERS)?;
    for name in statement.query_map(params![id], |row| row.get::<_, String>(0))? {
        folders.push(RelativePath::new(name?)?);
    }
    Ok(Scope::new(languages, folders))
}

fn replace_scope(tx: &Transaction<'_>, project: ProjectId, scope: &Scope) -> StorageResult<()> {
    let id = column_int(project.get())?;
    tx.execute(sql::SCOPE_LANGUAGES_DELETE, params![id])?;
    tx.execute(sql::SCOPE_FOLDERS_DELETE, params![id])?;
    for language in scope.languages() {
        tx.execute(sql::SCOPE_LANGUAGE_INSERT, params![id, language.as_str()])?;
    }
    for folder in scope.folders() {
        tx.execute(sql::SCOPE_FOLDER_INSERT, params![id, folder.as_str()])?;
    }
    Ok(())
}
