//! One database file per project and a small catalog (ADR 0095).
//!
//! [`SqliteStore`] is what the program opens. It keeps the catalog (`catalog.db`: the projects
//! and the identity of the installation) and opens the file of a project (`projects/<number>.db`,
//! a [`ProjectDb`]) when a call needs it. A call is routed by the project it names or by the
//! project that its snapshot, annotation or alert id carries. Removing a project removes its file.

mod consts;
mod files;
mod ports;
mod registry;
mod removal;

use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};

use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{DatabaseInfo, DatabaseState};
use codeskimr_domain::{Project, ProjectId, ProjectName, ProjectSource};

use self::consts::OPEN_FILES_MAX;
use self::registry::{Entry, Registry};
use crate::error::{StorageError, StorageResult};
use crate::store::{ProjectDb, StoreOptions};

/// The database of the program: a catalog and one file for each project.
#[derive(Debug)]
pub struct SqliteStore {
    /// The folder of the catalog; `None` for a store in memory.
    folder: Option<PathBuf>,
    registry: Registry,
    /// The open project files, the one used last at the end.
    open: Mutex<Vec<(ProjectId, Arc<ProjectDb>)>>,
    retention: NonZeroU32,
    options: StoreOptions,
}

impl SqliteStore {
    /// Opens the catalog at `catalog`, creating it and its folder if needed, finishes the
    /// removals that were interrupted and leaves the files of other installations alone.
    /// `retention` is the number of snapshots kept per project.
    ///
    /// # Errors
    /// `STO-601` and the other `STO` codes of the catalog or of a project file.
    pub fn open(catalog: &Path, retention: NonZeroU32) -> Result<Self, CodedError> {
        Self::open_with(catalog, retention, StoreOptions::default())
    }

    /// Like [`SqliteStore::open`], with other connection settings than the defaults.
    ///
    /// # Errors
    /// `STO-601` and the other `STO` codes of the catalog or of a project file.
    pub fn open_with(
        catalog: &Path,
        retention: NonZeroU32,
        options: StoreOptions,
    ) -> Result<Self, CodedError> {
        let store = Self {
            folder: catalog.parent().map(Path::to_path_buf),
            registry: Registry::open(catalog)?,
            open: Mutex::new(Vec::new()),
            retention,
            options,
        };
        store.note_old_database(catalog);
        store.finish_removals();
        store.clean_orphans();
        Ok(store)
    }

    /// A store whose catalog and project files live in memory, for tests and short-lived tools.
    ///
    /// # Errors
    /// `STO-601` if the catalog cannot be created.
    pub fn open_in_memory(retention: NonZeroU32) -> Result<Self, CodedError> {
        Ok(Self {
            folder: None,
            registry: Registry::in_memory()?,
            open: Mutex::new(Vec::new()),
            retention,
            options: StoreOptions::default(),
        })
    }

    /// The file of `project`: open already or opened now. A project that is being removed is no
    /// project for anyone but its removal.
    pub(crate) fn db(&self, project: ProjectId) -> StorageResult<Arc<ProjectDb>> {
        match self.registry.removing(project)? {
            Some(false) => self.file(project),
            _ => Err(StorageError::ProjectNotFound),
        }
    }

    /// Like [`Self::db`], also for a project whose removal has begun.
    pub(crate) fn db_any(&self, project: ProjectId) -> StorageResult<Arc<ProjectDb>> {
        match self.registry.removing(project)? {
            Some(_) => self.file(project),
            None => Err(StorageError::ProjectNotFound),
        }
    }

    /// The projects of the catalog, in the order of registration.
    pub(crate) fn entries(&self) -> StorageResult<Vec<Entry>> {
        self.registry.entries()
    }

    /// Registers a project, or updates the name, source and layers of a known id.
    ///
    /// # Errors
    /// `PRJ-301` if another project has the same source, otherwise a coded `STO` failure.
    pub fn save_project(&self, project: &Project) -> Result<(), CodedError> {
        self.saved(project).map_err(CodedError::from)
    }

    fn saved(&self, project: &Project) -> StorageResult<()> {
        if self.registry.removing(project.id)?.is_none() {
            return Err(StorageError::ProjectNotFound);
        }
        self.db(project.id)?.stored(project)?;
        self.registry.set_source(project.id, &project.source)
    }

    /// Gives the project a new display name; source, scans and layers stay as they are.
    ///
    /// # Errors
    /// `PRJ-201` if the project does not exist, otherwise a coded `STO` failure.
    pub fn rename_project(&self, id: ProjectId, name: ProjectName) -> Result<Project, CodedError> {
        self.db(id)
            .map_err(CodedError::from)?
            .rename_project(id, name)
    }

    /// The project registered for `source`, or a new one with a file of its own.
    ///
    /// # Errors
    /// `PRJ-301` is not raised here; a project that is being removed answers `PRJ-301` until it
    /// is gone. Otherwise a coded `STO` failure.
    pub fn register_project(
        &self,
        name: ProjectName,
        source: ProjectSource,
    ) -> Result<Project, CodedError> {
        self.registered(name, source).map_err(CodedError::from)
    }

    fn registered(&self, name: ProjectName, source: ProjectSource) -> StorageResult<Project> {
        if let Some(known) = self.registry.find(&source)? {
            if known.removing {
                return Err(StorageError::ProjectExists);
            }
            return self.file(known.project)?.load(known.project);
        }
        let number = self.registry.register(&source)?;
        let project = Project {
            id: number,
            name,
            source,
            layers: Vec::new(),
        };
        let created = self
            .file(number)
            .and_then(|db| db.stored(&project));
        if let Err(cause) = created {
            // The project was never usable: take it out of the catalog again.
            self.forget_file(number);
            self.registry.delete(number)?;
            return Err(cause);
        }
        Ok(project)
    }

    /// The project registered for `source`, if there is one.
    ///
    /// # Errors
    /// A coded `STO` failure.
    pub fn project_by_source(&self, source: &ProjectSource) -> Result<Option<Project>, CodedError> {
        let found = self.registry.find(source).map_err(CodedError::from)?;
        match found {
            Some(entry) if !entry.removing => self
                .file(entry.project)
                .and_then(|db| db.load(entry.project))
                .map(Some)
                .map_err(CodedError::from),
            _ => Ok(None),
        }
    }

    /// Where the file of `project` lies and how large it is.
    pub(crate) fn database_info(&self, project: ProjectId, removing: bool) -> Option<DatabaseInfo> {
        let path = self.path_of(project)?;
        let size = files::size_with_log(&path);
        let state = if removing {
            DatabaseState::Removing
        } else if size.is_some() {
            DatabaseState::Ok
        } else {
            DatabaseState::Missing
        };
        Some(DatabaseInfo {
            path: path.to_string_lossy().into_owned(),
            size_bytes: size.unwrap_or_default(),
            state,
        })
    }

    /// Closes the oldest open file that no call uses, when more than the limit are open. A store
    /// in memory keeps all of them, there is nothing to open again.
    fn trim(&self, open: &mut Vec<(ProjectId, Arc<ProjectDb>)>) {
        if self.folder.is_none() {
            return;
        }
        while open.len() > OPEN_FILES_MAX {
            match open.iter().position(|(_, db)| Arc::strong_count(db) == 1) {
                Some(index) => {
                    open.remove(index);
                }
                None => break,
            }
        }
    }

    /// The opened file of `project`, moved to the end of the list of the used ones.
    fn kept(&self, project: ProjectId) -> Option<Arc<ProjectDb>> {
        let mut open = self.open.lock().unwrap_or_else(PoisonError::into_inner);
        let index = open.iter().position(|(id, _)| *id == project)?;
        let used = open.remove(index);
        let db = Arc::clone(&used.1);
        open.push(used);
        Some(db)
    }

    /// Opens the file of `project`, or creates it.
    fn file(&self, project: ProjectId) -> StorageResult<Arc<ProjectDb>> {
        if let Some(db) = self.kept(project) {
            return Ok(db);
        }
        let db = Arc::new(self.open_file(project)?);
        let mut open = self.open.lock().unwrap_or_else(PoisonError::into_inner);
        // Another call may have opened it meanwhile; both share one connection then.
        if let Some((_, known)) = open.iter().find(|(id, _)| *id == project) {
            return Ok(Arc::clone(known));
        }
        open.push((project, Arc::clone(&db)));
        self.trim(&mut open);
        Ok(db)
    }

    /// Drops the open file of `project` from the list; the calls that still hold it finish.
    fn forget_file(&self, project: ProjectId) -> Option<Arc<ProjectDb>> {
        let mut open = self.open.lock().unwrap_or_else(PoisonError::into_inner);
        let index = open.iter().position(|(id, _)| *id == project)?;
        Some(open.remove(index).1)
    }
}
