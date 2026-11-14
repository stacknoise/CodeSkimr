//! Registered projects: the `ProjectRepository` port and saving a project.

use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{PreviousIndex, ProjectRepository};
use codeskimr_domain::{
    BranchName, ContentHash, CredentialId, FileId, FolderPath, GitUrl, LanguageId, Layer,
    LayerName, Loc, Project, ProjectId, ProjectName, ProjectSource, QualifiedName, RelativePath,
    SnapshotId, SourceFile,
};
use rusqlite::{Connection, OptionalExtension, Row, params};

use crate::codec::{Cols, column_int, id_from, ordinals, small_from};
use crate::consts::{
    FIRST_PROJECT_ID, ID_STEP, NO_BRANCH, NO_CREDENTIAL, SOURCE_FOLDER, SOURCE_GIT,
};
use crate::error::{StorageError, StorageResult};
use crate::previous::load_latest_index;
use crate::sql;
use crate::store::SqliteStore;

/// A source as stored: kind, location, branch and the name of the credential (never its secret).
struct SourceColumns {
    kind: &'static str,
    location: String,
    branch: String,
    credential: String,
}

impl SourceColumns {
    fn of(source: &ProjectSource) -> Self {
        match source {
            ProjectSource::Folder(path) => Self {
                kind: SOURCE_FOLDER,
                location: path.as_str().to_owned(),
                branch: NO_BRANCH.to_owned(),
                credential: NO_CREDENTIAL.to_owned(),
            },
            ProjectSource::Git {
                url,
                branch,
                credential,
            } => Self {
                kind: SOURCE_GIT,
                location: url.as_str().to_owned(),
                branch: branch.as_str().to_owned(),
                credential: credential
                    .as_ref()
                    .map_or_else(|| NO_CREDENTIAL.to_owned(), |id| id.as_str().to_owned()),
            },
        }
    }

    fn source(
        kind: &str,
        location: String,
        branch: String,
        credential: String,
    ) -> StorageResult<ProjectSource> {
        if kind == SOURCE_GIT {
            Ok(ProjectSource::Git {
                url: GitUrl::new(location)?,
                branch: BranchName::new(branch)?,
                credential: (credential != NO_CREDENTIAL)
                    .then(|| CredentialId::new(credential))
                    .transpose()?,
            })
        } else {
            Ok(ProjectSource::Folder(FolderPath::new(location)?))
        }
    }
}

impl SqliteStore {
    /// Registers a project, or updates the name, source and layers of a known id.
    ///
    /// # Errors
    /// `PRJ-301` if another project has the same source, otherwise a coded `STO` failure.
    pub fn save_project(&self, project: &Project) -> Result<(), CodedError> {
        self.write(|tx| save(tx, project)).map_err(CodedError::from)
    }

    /// The project registered for `source`, or a new one with the next free id, in one
    /// transaction: two processes that register at the same time never get the same id.
    ///
    /// # Errors
    /// A coded `STO` failure.
    pub fn register_project(
        &self,
        name: ProjectName,
        source: ProjectSource,
    ) -> Result<Project, CodedError> {
        self.write(|tx| {
            if let Some(known) = project_of_source(tx, &source)? {
                return load_project(tx, ProjectId::new(id_from(known)?));
            }
            let highest: Option<i64> = tx.query_row(sql::MAX_PROJECT_ID, [], |row| row.get(0))?;
            let next = highest.map_or(Ok(FIRST_PROJECT_ID), |id| {
                id_from(id).map(|id| id.saturating_add(ID_STEP))
            })?;
            let project = Project {
                id: ProjectId::new(next),
                name,
                source,
                layers: Vec::new(),
            };
            save(tx, &project)?;
            Ok(project)
        })
        .map_err(CodedError::from)
    }

    /// The project registered for `source`, if there is one.
    ///
    /// # Errors
    /// A coded `STO` failure.
    pub fn project_by_source(&self, source: &ProjectSource) -> Result<Option<Project>, CodedError> {
        self.read(|conn| {
            let columns = SourceColumns::of(source);
            let id: Option<i64> = conn
                .query_row(
                    sql::PROJECT_BY_SOURCE,
                    params![columns.kind, columns.location, columns.branch],
                    |row| row.get(0),
                )
                .optional()?;
            id.map(|raw| load_project(conn, ProjectId::new(id_from(raw)?)))
                .transpose()
        })
        .map_err(CodedError::from)
    }
}

/// The raw id of the project registered for `source`.
fn project_of_source(conn: &Connection, source: &ProjectSource) -> StorageResult<Option<i64>> {
    let columns = SourceColumns::of(source);
    Ok(conn
        .query_row(
            sql::PROJECT_BY_SOURCE,
            params![columns.kind, columns.location, columns.branch],
            |row| row.get(0),
        )
        .optional()?)
}

fn save(conn: &Connection, project: &Project) -> StorageResult<()> {
    let id = column_int(project.id.get())?;
    let source = SourceColumns::of(&project.source);
    let owner: Option<i64> = conn
        .query_row(
            sql::PROJECT_BY_SOURCE,
            params![source.kind, source.location, source.branch],
            |row| row.get(0),
        )
        .optional()?;
    if owner.is_some_and(|other| other != id) {
        return Err(StorageError::ProjectExists);
    }
    conn.execute(
        sql::PROJECT_UPSERT,
        params![
            id,
            project.name.as_str(),
            source.kind,
            source.location,
            source.branch,
            source.credential
        ],
    )?;
    conn.execute(sql::LAYERS_DELETE, params![id])?;
    for (position, layer) in ordinals().zip(&project.layers) {
        save_layer(conn, id, position, layer)?;
    }
    Ok(())
}

fn save_layer(conn: &Connection, project: i64, position: i64, layer: &Layer) -> StorageResult<()> {
    conn.execute(
        sql::LAYER_INSERT,
        params![project, position, layer.name.as_str()],
    )?;
    for (index, namespace) in ordinals().zip(&layer.namespaces) {
        conn.execute(
            sql::NAMESPACE_INSERT,
            params![project, position, index, namespace.as_str()],
        )?;
    }
    Ok(())
}

/// The layers of a project in their declared order.
pub fn load_layers(conn: &Connection, project: ProjectId) -> StorageResult<Vec<Layer>> {
    let id = column_int(project.get())?;
    let mut statement = conn.prepare_cached(sql::LAYERS_OF_PROJECT)?;
    let rows = statement.query_map(params![id], |row| {
        let cols = Cols::new(row);
        Ok((cols.take::<i64>()?, cols.take::<String>()?))
    })?;
    let mut layers = Vec::new();
    for row in rows {
        let (position, name) = row?;
        layers.push(Layer {
            name: LayerName::new(name)?,
            namespaces: load_namespaces(conn, id, position)?,
        });
    }
    Ok(layers)
}

fn load_namespaces(
    conn: &Connection,
    project: i64,
    layer: i64,
) -> StorageResult<Vec<QualifiedName>> {
    let mut statement = conn.prepare_cached(sql::NAMESPACES_OF_LAYER)?;
    let rows = statement.query_map(params![project, layer], |row| row.get::<_, String>(0))?;
    let mut names = Vec::new();
    for row in rows {
        names.push(QualifiedName::new(row?)?);
    }
    Ok(names)
}

/// A row of `projects` as stored.
struct ProjectRow {
    name: String,
    kind: String,
    location: String,
    branch: String,
    credential: String,
}

impl ProjectRow {
    fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        let cols = Cols::new(row);
        Ok(Self {
            name: cols.take()?,
            kind: cols.take()?,
            location: cols.take()?,
            branch: cols.take()?,
            credential: cols.take()?,
        })
    }
}

pub fn load_project(conn: &Connection, id: ProjectId) -> StorageResult<Project> {
    let raw = column_int(id.get())?;
    let row = conn
        .query_row(sql::PROJECT_BY_ID, params![raw], ProjectRow::from_row)
        .optional()?;
    let stored = row.ok_or(StorageError::ProjectNotFound)?;
    Ok(Project {
        id,
        name: ProjectName::new(stored.name)?,
        source: SourceColumns::source(
            &stored.kind,
            stored.location,
            stored.branch,
            stored.credential,
        )?,
        layers: load_layers(conn, id)?,
    })
}

/// A file from a row of `files`: snapshot, id, path, language, hash, loc.
fn file_from(row: &Row<'_>) -> rusqlite::Result<(i64, i64, String, String, String, i64)> {
    let cols = Cols::new(row);
    Ok((
        cols.take()?,
        cols.take()?,
        cols.take()?,
        cols.take()?,
        cols.take()?,
        cols.take()?,
    ))
}

fn load_latest_files(conn: &Connection, project: ProjectId) -> StorageResult<Vec<SourceFile>> {
    let raw = column_int(project.get())?;
    let mut statement = conn.prepare_cached(sql::LATEST_FILES)?;
    let rows = statement.query_map(params![raw], file_from)?;
    let mut files = Vec::new();
    for row in rows {
        let (snapshot, id, path, language, hash, loc) = row?;
        files.push(SourceFile {
            id: FileId::new(id_from(id)?),
            snapshot: SnapshotId::new(id_from(snapshot)?),
            path: RelativePath::new(path)?,
            language: LanguageId::new(language)?,
            content_hash: ContentHash::new(hash)?,
            loc: Loc(small_from(loc)?),
        });
    }
    Ok(files)
}

impl ProjectRepository for SqliteStore {
    fn project(&self, id: ProjectId) -> Result<Project, CodedError> {
        self.read(|conn| load_project(conn, id))
            .map_err(CodedError::from)
    }

    fn latest_files(&self, id: ProjectId) -> Result<Vec<SourceFile>, CodedError> {
        self.read(|conn| load_latest_files(conn, id))
            .map_err(CodedError::from)
    }

    fn latest_index(&self, id: ProjectId) -> Result<PreviousIndex, CodedError> {
        self.read(|conn| load_latest_index(conn, id))
            .map_err(CodedError::from)
    }

    fn latest_naming_rules(&self, id: ProjectId) -> Result<Option<String>, CodedError> {
        self.read(|conn| {
            conn.query_row(
                sql::SNAPSHOT_NAMING_LATEST,
                params![column_int(id.get())?],
                |row| row.get::<_, Option<String>>(0),
            )
            .optional()
            .map(Option::flatten)
            .map_err(StorageError::from)
        })
        .map_err(CodedError::from)
    }
}
