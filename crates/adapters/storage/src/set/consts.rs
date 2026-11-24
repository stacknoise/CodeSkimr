//! Names and statements of the set of project files (ADR 0095).

/// The shared database of the versions before ADR 0095; it is no longer opened.
pub const OLD_DATABASE_FILE: &str = "codeskimr.db";
/// The folder below the catalog's folder that holds one file per project.
pub const PROJECTS_FOLDER: &str = "projects";
/// The extension of a project file.
pub const PROJECT_FILE_EXTENSION: &str = "db";
/// Suffixes of the files `SQLite` keeps next to a database in write-ahead mode.
pub const SIDE_FILE_SUFFIXES: [&str; 2] = ["-wal", "-shm"];
/// The program's identity of a catalog and its project files, written to both so that a file of
/// another installation is never taken for an orphan (`PRAGMA application_id`).
pub const APPLICATION_ID: i64 = 0x434B_534D;
/// Pragma that holds [`APPLICATION_ID`].
pub const PRAGMA_APPLICATION_ID: &str = "application_id";

/// Most project files that stay open; the least recently used is closed first.
pub const OPEN_FILES_MAX: usize = 32;
/// How long a removal waits for the calls that still use the file, and how long it retries to
/// delete a file that the system still holds, in milliseconds.
pub const REMOVAL_WAIT_MS: u64 = 30_000;
/// The pause between two tries of the removal, in milliseconds.
pub const REMOVAL_PAUSE_MS: u64 = 50;

/// The reason of a removal that gave up because a call still uses the file.
pub const REMOVAL_STILL_USED: &str = "the project file is still in use";

/// Log event: a project file without a catalog row was found and left alone.
pub const LOG_FILE_LEFT_ALONE: &str = "project_file_left_alone";
/// Log event: a project file without a catalog row was deleted.
pub const LOG_ORPHAN_DELETED: &str = "project_file_orphan_deleted";
/// Log event: the database of an older version is no longer used.
pub const LOG_OLD_DATABASE_IGNORED: &str = "old_database_ignored";
/// Log event: a removal that was interrupted is finished.
pub const LOG_REMOVAL_FINISHED: &str = "project_removal_finished";

/// Between the time and the process in the name of an installation.
pub const INSTALLATION_SEPARATOR: char = '-';

/// Creates the catalog: the projects and the identity of the installation.
pub const REGISTRY_SCHEMA: &str = "\
    CREATE TABLE IF NOT EXISTS projects ( \
        number INTEGER PRIMARY KEY AUTOINCREMENT, \
        kind TEXT NOT NULL, location TEXT NOT NULL, branch TEXT NOT NULL, \
        removing INTEGER NOT NULL DEFAULT 0, \
        UNIQUE (kind, location, branch)) STRICT; \
    CREATE TABLE IF NOT EXISTS installation (id TEXT NOT NULL) STRICT;";
/// The installation id, if one was written.
pub const INSTALLATION_ID: &str = "SELECT id FROM installation LIMIT 1";
/// Writes the installation id.
pub const INSTALLATION_INSERT: &str = "INSERT INTO installation (id) VALUES (?1)";
/// Registers a project: kind, location, branch.
pub const PROJECT_INSERT: &str =
    "INSERT INTO projects (kind, location, branch) VALUES (?1, ?2, ?3)";
/// Registers a project under a given number: number, kind, location, branch.
pub const PROJECT_INSERT_AS: &str =
    "INSERT INTO projects (number, kind, location, branch) VALUES (?1, ?2, ?3, ?4)";
/// The project of a source: kind, location, branch; its number and whether it is being removed.
pub const PROJECT_BY_SOURCE: &str =
    "SELECT number, removing FROM projects WHERE kind = ?1 AND location = ?2 AND branch = ?3";
/// Every registered project with its removal mark, in the order of registration.
pub const PROJECT_NUMBERS: &str = "SELECT number, removing FROM projects ORDER BY number";
/// The removal mark of a project: its number.
pub const PROJECT_REMOVING: &str = "SELECT removing FROM projects WHERE number = ?1";
/// Marks a project as being removed: its number.
pub const PROJECT_MARK_REMOVING: &str = "UPDATE projects SET removing = 1 WHERE number = ?1";
/// Gives a project another source: kind, location, branch, number.
pub const PROJECT_SET_SOURCE: &str =
    "UPDATE projects SET kind = ?1, location = ?2, branch = ?3 WHERE number = ?4";
/// Removes the row of a project: its number.
pub const PROJECT_DELETE: &str = "DELETE FROM projects WHERE number = ?1";
/// The highest project number ever handed out, kept by `AUTOINCREMENT` and not by the rows.
pub const HIGHEST_NUMBER_EVER: &str = "SELECT seq FROM sqlite_sequence WHERE name = 'projects'";
/// A quick check that the catalog is intact.
pub const QUICK_CHECK: &str = "PRAGMA quick_check";
/// The answer of a check that found nothing.
pub const CHECK_OK: &str = "ok";

/// Marks a project file with the installation and the project it belongs to.
pub const FILE_IDENTITY_SCHEMA: &str = "\
    CREATE TABLE IF NOT EXISTS identity (installation TEXT NOT NULL, project INTEGER NOT NULL) STRICT;";
/// The identity of a project file.
pub const FILE_IDENTITY: &str = "SELECT installation, project FROM identity LIMIT 1";
/// Writes the identity of a project file: installation, project.
pub const FILE_IDENTITY_INSERT: &str =
    "INSERT INTO identity (installation, project) VALUES (?1, ?2)";
/// Starts the counters of the annotations and alerts of a project at its number (ADR 0095), so
/// that their ids name the project: name, first value.
pub const SEQUENCE_SEED: &str = "INSERT INTO sqlite_sequence (name, seq) VALUES (?1, ?2)";
/// The tables whose ids count from the project number.
pub const SEEDED_TABLES: [&str; 2] = ["annotation", "alert"];
