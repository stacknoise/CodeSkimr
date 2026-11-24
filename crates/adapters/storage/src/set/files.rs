//! The project files of a store: where they lie, how they are opened, and the clean-up at the
//! start (ADR 0095).

use std::fs;
use std::path::{Path, PathBuf};

use codeskimr_domain::ProjectId;
use rusqlite::{Connection, OpenFlags, OptionalExtension, params};

use super::SqliteStore;
use super::consts::{
    APPLICATION_ID, FILE_IDENTITY, FILE_IDENTITY_INSERT, FILE_IDENTITY_SCHEMA, LOG_FILE_LEFT_ALONE,
    LOG_OLD_DATABASE_IGNORED, LOG_ORPHAN_DELETED, LOG_REMOVAL_FINISHED, OLD_DATABASE_FILE,
    PRAGMA_APPLICATION_ID, PROJECT_FILE_EXTENSION, PROJECTS_FOLDER, SEEDED_TABLES, SEQUENCE_SEED,
    SIDE_FILE_SUFFIXES,
};
use crate::codec::column_int;
use crate::error::StorageResult;
use crate::store::ProjectDb;
use codeskimr_application::ids::first_snapshot_of;

/// The size of the file and of its write-ahead log in bytes, `None` if the file is missing.
pub(super) fn size_with_log(path: &Path) -> Option<u64> {
    let main = fs::metadata(path).ok()?.len();
    let logs: u64 = side_files(path)
        .iter()
        .filter_map(|side| fs::metadata(side).ok())
        .map(|meta| meta.len())
        .sum();
    Some(main.saturating_add(logs))
}

/// The files `SQLite` keeps next to the database at `path`.
pub(super) fn side_files(path: &Path) -> Vec<PathBuf> {
    SIDE_FILE_SUFFIXES
        .iter()
        .map(|suffix| {
            let mut name = path.as_os_str().to_owned();
            name.push(suffix);
            PathBuf::from(name)
        })
        .collect()
}

/// The project a file of the projects folder belongs to, by its name.
fn project_of_file(path: &Path) -> Option<ProjectId> {
    let extension_ok = path
        .extension()
        .is_some_and(|ext| ext == PROJECT_FILE_EXTENSION);
    let number: u64 = path.file_stem()?.to_str()?.parse().ok()?;
    extension_ok.then(|| ProjectId::new(number))
}

impl SqliteStore {
    /// Says once that the shared database of an earlier version lies next to the catalog and is
    /// no longer used (ADR 0095: nothing is migrated).
    pub(super) fn note_old_database(catalog: &Path) {
        let old = catalog.with_file_name(OLD_DATABASE_FILE);
        if old.exists() {
            tracing::info!(event = LOG_OLD_DATABASE_IGNORED, file = %old.display());
        }
    }

    /// The path of the file of `project`; `None` for a store in memory.
    pub(super) fn path_of(&self, project: ProjectId) -> Option<PathBuf> {
        let folder = self.folder.as_ref()?;
        Some(
            folder
                .join(PROJECTS_FOLDER)
                .join(project.get().to_string())
                .with_extension(PROJECT_FILE_EXTENSION),
        )
    }

    /// Opens the file of `project`, creating it with the identity of the installation.
    pub(super) fn open_file(&self, project: ProjectId) -> StorageResult<ProjectDb> {
        let db = match self.path_of(project) {
            Some(path) => ProjectDb::open_checked(&path, self.retention, self.options)?,
            None => ProjectDb::prepared(
                Connection::open_in_memory()?,
                None,
                self.retention,
                self.options,
            )?,
        };
        let installation = self.registry.installation();
        db.write(|tx| {
            tx.pragma_update(None, PRAGMA_APPLICATION_ID, APPLICATION_ID)?;
            tx.execute_batch(FILE_IDENTITY_SCHEMA)?;
            let known: Option<String> = tx
                .query_row(FILE_IDENTITY, [], |row| row.get(0))
                .optional()?;
            if known.is_none() {
                tx.execute(
                    FILE_IDENTITY_INSERT,
                    params![installation, column_int(project.get())?],
                )?;
                // The ids of notes and alerts name the project, like those of the snapshots.
                let base = column_int(first_snapshot_of(project) - 1)?;
                for table in SEEDED_TABLES {
                    tx.execute(SEQUENCE_SEED, params![table, base])?;
                }
            }
            Ok(())
        })?;
        Ok(db)
    }

    /// Finishes the removals that were interrupted: the projects that the catalog marks.
    pub(super) fn finish_removals(&self) {
        let Ok(entries) = self.entries() else {
            return;
        };
        for entry in entries.into_iter().filter(|entry| entry.removing) {
            if self.remove_marked(entry.project).is_ok() {
                tracing::info!(event = LOG_REMOVAL_FINISHED, project = entry.project.get());
            }
        }
    }

    /// Looks at the files of the projects folder that the catalog does not list. A file is
    /// deleted only if the catalog is intact, the file carries the identity of this installation
    /// and its number is not above the highest number the catalog ever handed out; anything
    /// else is left alone and logged, so that a lost or older catalog never costs a project.
    pub(super) fn clean_orphans(&self) {
        let Some(folder) = self.folder.as_ref().map(|dir| dir.join(PROJECTS_FOLDER)) else {
            return;
        };
        let (Ok(listed), Ok(highest), Ok(files)) = (
            self.entries(),
            self.registry.highest_number_ever(),
            fs::read_dir(&folder),
        ) else {
            return;
        };
        let intact = self.registry.intact();
        for path in files.filter_map(Result::ok).map(|file| file.path()) {
            let Some(project) = project_of_file(&path) else {
                continue;
            };
            if listed.iter().any(|entry| entry.project == project) {
                continue;
            }
            let ours = intact
                && project.get() <= highest
                && has_identity(&path, self.registry.installation(), project)
                // Asked again right before the delete: a second process may have registered
                // the project since the list above was read.
                && self.registry.removing(project).is_ok_and(|row| row.is_none());
            if ours && delete_with_log(&path).is_ok() {
                tracing::info!(event = LOG_ORPHAN_DELETED, project = project.get());
            } else {
                tracing::warn!(event = LOG_FILE_LEFT_ALONE, project = project.get());
            }
        }
    }
}

/// Whether the file at `path` carries the identity of this installation for `project`.
fn has_identity(path: &Path, installation: &str, project: ProjectId) -> bool {
    let Ok(conn) = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY) else {
        return false;
    };
    let id: Option<(String, i64)> = conn
        .query_row(FILE_IDENTITY, [], |row| Ok((row.get(0)?, row.get(1)?)))
        .ok();
    id.is_some_and(|(found, number)| {
        found == installation && u64::try_from(number).is_ok_and(|n| n == project.get())
    })
}

/// Deletes a database file and its side files.
pub(super) fn delete_with_log(path: &Path) -> std::io::Result<()> {
    for side in side_files(path) {
        match fs::remove_file(&side) {
            Err(error) if error.kind() != std::io::ErrorKind::NotFound => return Err(error),
            _ => {}
        }
    }
    match fs::remove_file(path) {
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => Err(error),
        _ => Ok(()),
    }
}
