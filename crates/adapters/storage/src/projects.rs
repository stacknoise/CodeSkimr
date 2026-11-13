//! Registered projects: the `ProjectRepository` port and saving a project.

use codeskimr_application::error::CodedError;
use codeskimr_application::ports::ProjectRepository;
use codeskimr_domain::{
    BranchName, ContentHash, CredentialId, FileId, FolderPath, GitUrl, LanguageId, Layer,
    LayerName, Loc, Project, ProjectId, ProjectName, ProjectSource, QualifiedName, RelativePath,
    SnapshotId, SourceFile,
};
use rusqlite::{Connection, OptionalExtension, Row, params};

use crate::codec::{Cols, column_int, id_from, ordinals, small_from};
use crate::consts::{NO_BRANCH, NO_CREDENTIAL, SOURCE_FOLDER, SOURCE_GIT};
use crate::error::{StorageError, StorageResult};
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

/// The columns of a project row: name, source kind, location, branch, credential name.
fn project_columns(row: &Row<'_>) -> rusqlite::Result<(String, String, String, String, String)> {
    Ok((
        row.get(0)?,
        row.get(1)?,
        row.get(2)?,
        row.get(3)?,
        row.get(4)?,
    ))
}

fn load_project(conn: &Connection, id: ProjectId) -> StorageResult<Project> {
    let raw = column_int(id.get())?;
    let row = conn
        .query_row(sql::PROJECT_BY_ID, params![raw], project_columns)
        .optional()?;
    let (name, kind, location, branch, credential) = row.ok_or(StorageError::ProjectNotFound)?;
    Ok(Project {
        id,
        name: ProjectName::new(name)?,
        source: SourceColumns::source(&kind, location, branch, credential)?,
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
}
