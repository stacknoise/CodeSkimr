//! Registered projects: the `ProjectRepository` port and saving a project.

use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{
    PendingReference, PreviousIndex, PreviousState, ProjectRepository, SkippedFile, StoredNames,
    StoredReferences,
};
use codeskimr_codes::ErrorCode;
use codeskimr_domain::{
    BranchName, ContentHash, CredentialId, FileId, FolderPath, GitUrl, LanguageId, Layer,
    LayerName, Loc, Project, ProjectId, ProjectName, ProjectSource, QualifiedName, Reference,
    RelativePath, SnapshotId, SourceFile, SymbolId, SymbolPair,
};
use rusqlite::{Connection, OptionalExtension, Row, params};

use crate::codec::{Cols, column_int, id_from, ordinals, rows_of_snapshot, small_from};
use crate::consts::{
    FIRST_PROJECT_ID, ID_STEP, NO_BRANCH, NO_CREDENTIAL, SOURCE_FOLDER, SOURCE_GIT,
};
use crate::error::{StorageError, StorageResult};
use crate::pending;
use crate::previous::{latest_snapshot_id, load_latest_index, load_symbols};
use crate::read_models;
use crate::references;
use crate::sql;
use crate::store::ProjectDb;

/// A source as stored: kind, location, branch and the name of the credential (never its secret).
pub(crate) struct SourceColumns {
    pub(crate) kind: &'static str,
    pub(crate) location: String,
    pub(crate) branch: String,
    credential: String,
}

impl SourceColumns {
    pub(crate) fn of(source: &ProjectSource) -> Self {
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

    pub(crate) fn source(
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

impl ProjectDb {
    /// Registers a project, or updates the name, source and layers of a known id.
    ///
    /// # Errors
    /// `PRJ-301` if another project has the same source, otherwise a coded `STO` failure.
    pub fn save_project(&self, project: &Project) -> Result<(), CodedError> {
        self.stored(project).map_err(CodedError::from)
    }

    /// [`Self::save_project`] with the failure of the store.
    pub(crate) fn stored(&self, project: &Project) -> StorageResult<()> {
        self.write(|tx| {
            save(tx, project)?;
            // The layer graph of the newest snapshot may have been built again.
            self.forget_builds();
            Ok(())
        })
    }

    /// Marks the project as being removed.
    pub(crate) fn mark_removing(&self, id: ProjectId) -> StorageResult<()> {
        self.write(|tx| {
            tx.execute(sql::PROJECT_REMOVING_SET, params![column_int(id.get())?])?;
            Ok(())
        })
    }

    /// The project with `id`, if it is not being removed.
    pub(crate) fn load(&self, id: ProjectId) -> StorageResult<Project> {
        self.read(|conn| load_project(conn, id))
    }

    /// Gives the project a new display name; source, scans and layers stay as they are.
    ///
    /// # Errors
    /// `PRJ-201` if the project does not exist, otherwise a coded `STO` failure.
    pub fn rename_project(&self, id: ProjectId, name: ProjectName) -> Result<Project, CodedError> {
        self.write(|tx| {
            let mut project = load_project(tx, id)?;
            project.name = name;
            save(tx, &project)?;
            Ok(project)
        })
        .map_err(CodedError::from)
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
                // A project that is being removed still owns its source until it is gone.
                let removing: i64 =
                    tx.query_row(sql::PROJECT_REMOVING, params![known], |row| row.get(0))?;
                if removing != 0 {
                    return Err(StorageError::ProjectExists);
                }
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
            // A project that is being removed is no project any more, for anyone but its removal.
            id.map(|raw| load_project(conn, ProjectId::new(id_from(raw)?)))
                .transpose()
                .or_else(|error| match error {
                    StorageError::ProjectNotFound => Ok(None),
                    other => Err(other),
                })
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
    let before = load_layers(conn, project.id)?;
    conn.execute(sql::LAYERS_DELETE, params![id])?;
    for (position, layer) in ordinals().zip(&project.layers) {
        save_layer(conn, id, position, layer)?;
    }
    if before != project.layers {
        // A scan that finds no file changed writes no snapshot (ADR 0093), so the layer graph of
        // the newest one follows the layers here.
        read_models::relayer(conn, project.id, &project.layers)?;
    }
    Ok(())
}

fn save_layer(conn: &Connection, project: i64, position: i64, layer: &Layer) -> StorageResult<()> {
    conn.execute(
        sql::LAYER_INSERT,
        params![
            project,
            position,
            layer.name.as_str(),
            layer.may_use.is_some()
        ],
    )?;
    for (index, used) in ordinals().zip(layer.may_use.iter().flatten()) {
        conn.execute(
            sql::USED_INSERT,
            params![project, position, index, used.as_str()],
        )?;
    }
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
    let rows = statement.query_map(params![id], LayerRow::from_row)?;
    let mut layers = Vec::new();
    for row in rows {
        let row = row?;
        layers.push(Layer {
            name: LayerName::new(row.name)?,
            namespaces: load_namespaces(conn, id, row.position)?,
            may_use: if row.restricted {
                Some(load_used(conn, id, row.position)?)
            } else {
                None
            },
        });
    }
    Ok(layers)
}

fn load_used(conn: &Connection, project: i64, layer: i64) -> StorageResult<Vec<LayerName>> {
    load_names(conn, sql::USED_OF_LAYER, (project, layer), LayerName::new)
}

fn load_namespaces(
    conn: &Connection,
    project: i64,
    layer: i64,
) -> StorageResult<Vec<QualifiedName>> {
    load_names(
        conn,
        sql::NAMESPACES_OF_LAYER,
        (project, layer),
        QualifiedName::new,
    )
}

/// The names that `query` selects for one layer, each made into its type by `make`.
fn load_names<T, E>(
    conn: &Connection,
    query: &str,
    (project, layer): (i64, i64),
    make: impl Fn(String) -> Result<T, E>,
) -> StorageResult<Vec<T>>
where
    StorageError: From<E>,
{
    let mut statement = conn.prepare_cached(query)?;
    let rows = statement.query_map(params![project, layer], |row| row.get::<_, String>(0))?;
    let mut names = Vec::new();
    for row in rows {
        names.push(make(row?)?);
    }
    Ok(names)
}

/// A row of `layers` as stored.
struct LayerRow {
    position: i64,
    name: String,
    restricted: bool,
}

impl LayerRow {
    // codeskimr: allow CHK-102 maps a row of the layer query into its columns
    fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        let cols = Cols::new(row);
        Ok(Self {
            position: cols.take()?,
            name: cols.take()?,
            restricted: cols.take()?,
        })
    }
}

/// A row of `projects` as stored.
struct ProjectRow {
    name: String,
    kind: String,
    pub(crate) location: String,
    pub(crate) branch: String,
    credential: String,
}

impl ProjectRow {
    // codeskimr: allow CHK-102 maps a row of the project query into its columns
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
    load_with(conn, id, sql::PROJECT_BY_ID)
}

/// The project also while its removal has begun; only the removal and the list need it.
pub fn load_project_any(conn: &Connection, id: ProjectId) -> StorageResult<Project> {
    load_with(conn, id, sql::PROJECT_BY_ID_ANY)
}

fn load_with(conn: &Connection, id: ProjectId, query: &str) -> StorageResult<Project> {
    let raw = column_int(id.get())?;
    let row = conn
        .query_row(query, params![raw], ProjectRow::from_row)
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

fn load_latest_state(conn: &Connection, project: ProjectId) -> StorageResult<PreviousState> {
    load_latest_state_with(conn, project, true)
}

/// The state of the newest snapshot, with its symbols if `symbols` is set.
fn load_latest_state_with(
    conn: &Connection,
    project: ProjectId,
    symbols: bool,
) -> StorageResult<PreviousState> {
    let Some(snapshot) = latest_snapshot_id(conn, project)? else {
        return Ok(PreviousState::default());
    };
    let ambiguous: Option<i64> =
        conn.query_row(sql::SNAPSHOT_AMBIGUOUS, params![snapshot], |row| row.get(0))?;
    Ok(PreviousState {
        files: load_latest_files(conn, project)?,
        symbols: if symbols {
            load_symbols(conn, snapshot)?
        } else {
            Vec::new()
        },
        snapshot: Some(SnapshotId::new(id_from(snapshot)?)),
        names_stored: pending::is_stored(conn, snapshot)?,
        ambiguous_places: ambiguous.map(id_from).transpose()?,
        skipped: load_skipped(conn, snapshot)?,
    })
}

/// The files the scan of `snapshot` left out for good. A row with a code that this version does
/// not know any more is left out: the next scan then looks at the file again.
fn load_skipped(conn: &Connection, snapshot: i64) -> StorageResult<Vec<SkippedFile>> {
    let snapshot = SnapshotId::new(id_from(snapshot)?);
    let rows = rows_of_snapshot(conn, sql::SKIPPED_FILES, snapshot, |cols| {
        Ok((
            cols.take::<String>()?,
            cols.take::<String>()?,
            cols.take::<String>()?,
            cols.take::<String>()?,
        ))
    })?;
    let mut skipped = Vec::new();
    for (path, language, hash, id) in rows {
        let Some(code) = ErrorCode::ALL.iter().copied().find(|code| code.id() == id) else {
            continue;
        };
        skipped.push(SkippedFile {
            path: RelativePath::new(path)?,
            language: LanguageId::new(language)?,
            content_hash: ContentHash::new(hash)?,
            code,
        });
    }
    Ok(skipped)
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

impl ProjectRepository for ProjectDb {
    fn project_for_removal(&self, id: ProjectId) -> Result<Project, CodedError> {
        self.read(|conn| load_project_any(conn, id))
            .map_err(CodedError::from)
    }

    fn project(&self, id: ProjectId) -> Result<Project, CodedError> {
        self.read(|conn| load_project(conn, id))
            .map_err(CodedError::from)
    }

    fn latest_files(&self, id: ProjectId) -> Result<Vec<SourceFile>, CodedError> {
        self.read_latest(|conn| load_latest_files(conn, id))
            .map_err(CodedError::from)
    }

    fn latest_index(&self, id: ProjectId) -> Result<PreviousIndex, CodedError> {
        self.read_latest(|conn| load_latest_index(conn, id))
            .map_err(CodedError::from)
    }

    fn latest_state(&self, id: ProjectId) -> Result<PreviousState, CodedError> {
        self.read_latest(|conn| {
            // One read transaction, so a snapshot written in between cannot split the parts.
            let tx = conn.unchecked_transaction()?;
            load_latest_state(&tx, id)
        })
        .map_err(CodedError::from)
    }

    fn latest_head(&self, id: ProjectId) -> Result<PreviousState, CodedError> {
        self.read_latest(|conn| {
            let tx = conn.unchecked_transaction()?;
            load_latest_state_with(&tx, id, false)
        })
        .map_err(CodedError::from)
    }

    fn snapshot_symbols(
        &self,
        snapshot: SnapshotId,
    ) -> Result<Vec<codeskimr_domain::Symbol>, CodedError> {
        self.read_latest(|conn| load_symbols(conn, column_int(snapshot.get())?))
            .map_err(CodedError::from)
    }

    fn stored_names(
        &self,
        snapshot: SnapshotId,
        which: StoredNames<'_>,
    ) -> Result<Vec<PendingReference>, CodedError> {
        let which = match which {
            StoredNames::Bases => pending::Which::Bases,
            StoredNames::ImportsOf(files) => pending::Which::ImportsOf(files),
            StoredNames::Callers(files, callers) => pending::Which::Callers(files, callers),
            StoredNames::Targeting(names) => pending::Which::Targeting(names),
            StoredNames::ImportsTargeting(names) => pending::Which::ImportsTargeting(names),
            StoredNames::OfFiles(files) => pending::Which::OfFiles(files),
            StoredNames::All => pending::Which::All,
        };
        self.read_latest(|conn| pending::load(conn, column_int(snapshot.get())?, &which))
            .map_err(CodedError::from)
    }

    fn stored_references(
        &self,
        snapshot: SnapshotId,
        which: StoredReferences<'_>,
    ) -> Result<Vec<Reference>, CodedError> {
        let which = match which {
            StoredReferences::To(symbols) => references::Which::To(symbols),
            StoredReferences::From(symbols) => references::Which::From(symbols),
            StoredReferences::In(files) => references::Which::In(files),
            StoredReferences::All => references::Which::All,
        };
        self.read_latest(|conn| references::load(conn, column_int(snapshot.get())?, &which))
            .map_err(CodedError::from)
    }

    fn verified_files(&self, snapshot: SnapshotId) -> Result<Vec<FileId>, CodedError> {
        self.read_latest(|conn| references::verified_files(conn, column_int(snapshot.get())?))
            .map_err(CodedError::from)
    }

    fn reference_pairs(
        &self,
        snapshot: SnapshotId,
        files: &[FileId],
        without: &[SymbolId],
    ) -> Result<Vec<SymbolPair>, CodedError> {
        self.read_latest(|conn| {
            references::reference_pairs(conn, column_int(snapshot.get())?, files, without)
        })
        .map_err(CodedError::from)
    }

    fn latest_naming_rules(&self, id: ProjectId) -> Result<Option<String>, CodedError> {
        self.read_latest(|conn| {
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
