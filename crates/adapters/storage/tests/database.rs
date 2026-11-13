//! The database file: reopening, migrations and pruning on disk.

#[cfg(test)]
mod common;

use std::num::NonZeroU32;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use codeskimr_application::ports::{ProjectRepository, SnapshotStore, SymbolFingerprints};
use codeskimr_domain::{Fingerprint, Fingerprints, ScanMode, SymbolId, SymbolKind};
use codeskimr_storage::SqliteStore;
use common::{PROJECT, code_of, data, file, header, project, symbol};
use rusqlite::Connection;

static NEXT: AtomicU32 = AtomicU32::new(0);

/// A database path in a fresh temporary folder that is removed again on drop.
#[cfg(test)]
struct TempDb(PathBuf);

#[cfg(test)]
impl TempDb {
    fn new() -> Self {
        let unique = NEXT.fetch_add(1, Ordering::Relaxed);
        let folder =
            std::env::temp_dir().join(format!("codeskimr-storage-{}-{unique}", std::process::id()));
        Self(folder.join("nested").join("workspace.db"))
    }
}

#[cfg(test)]
impl Drop for TempDb {
    fn drop(&mut self) {
        if let Some(root) = self.0.parent().and_then(std::path::Path::parent) {
            let _ = std::fs::remove_dir_all(root);
        }
    }
}

#[cfg(test)]
fn keep(count: u32) -> NonZeroU32 {
    NonZeroU32::new(count).unwrap()
}

#[test]
fn file_database_is_created_and_survives_reopening() {
    let db = TempDb::new();
    {
        let store = SqliteStore::open(&db.0, keep(3)).unwrap();
        store.save_project(&project(1, "/work/app", &[])).unwrap();
    }
    let store = SqliteStore::open(&db.0, keep(3)).unwrap();
    assert_eq!(store.project(PROJECT).unwrap().name.as_str(), "project 1");
}

#[test]
fn file_database_uses_wal_and_the_current_schema_version() {
    let db = TempDb::new();
    SqliteStore::open(&db.0, keep(3)).unwrap();
    let conn = Connection::open(&db.0).unwrap();
    let mode: String = conn
        .pragma_query_value(None, "journal_mode", |row| row.get(0))
        .unwrap();
    let version: i64 = conn
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .unwrap();
    assert_eq!((mode.as_str(), version), ("wal", 3));
}

#[test]
fn a_version_one_database_gets_projects_without_a_credential() {
    let db = TempDb::new();
    std::fs::create_dir_all(db.0.parent().unwrap()).unwrap();
    let conn = Connection::open(&db.0).unwrap();
    conn.execute_batch(include_str!("../migrations/0001_initial.sql"))
        .unwrap();
    conn.execute(
        "INSERT INTO projects (id, name, source_kind, location, branch) \
         VALUES (1, 'old', 'git', 'https://example.org/a.git', 'main')",
        [],
    )
    .unwrap();
    conn.pragma_update(None, "user_version", 1).unwrap();
    drop(conn);

    let store = SqliteStore::open(&db.0, keep(3)).unwrap();
    let loaded = store.project(PROJECT).unwrap();
    assert!(matches!(
        loaded.source,
        codeskimr_domain::ProjectSource::Git {
            credential: None,
            ..
        }
    ));
}

#[test]
fn database_of_a_newer_version_is_refused() {
    let db = TempDb::new();
    SqliteStore::open(&db.0, keep(3)).unwrap();
    Connection::open(&db.0)
        .unwrap()
        .pragma_update(None, "user_version", 99)
        .unwrap();
    let error = SqliteStore::open(&db.0, keep(3)).unwrap_err();
    assert_eq!(code_of(&error), "STO-301");
}

#[test]
fn a_folder_instead_of_a_file_is_a_database_failure() {
    let db = TempDb::new();
    let folder = db.0.parent().unwrap();
    std::fs::create_dir_all(folder).unwrap();
    let error = SqliteStore::open(folder, keep(3)).unwrap_err();
    assert_eq!(code_of(&error), "STO-601");
}

#[test]
fn pruned_snapshots_leave_no_rows_behind() {
    let db = TempDb::new();
    let store = SqliteStore::open(&db.0, keep(1)).unwrap();
    store.save_project(&project(1, "/work/app", &[])).unwrap();
    for (snapshot, base) in [(10, 100), (20, 200)] {
        let mut scan = data(header(snapshot, ScanMode::Full));
        scan.files = vec![file(snapshot, base, "a.cs", "h")];
        scan.symbols = vec![symbol(base + 1, base, SymbolKind::Class, "App.A", None)];
        scan.fingerprints = vec![SymbolFingerprints {
            symbol: SymbolId::new(base + 1),
            fingerprints: Fingerprints::from_entries(vec![Fingerprint {
                hash: base,
                position: 0,
            }]),
        }];
        store.persist(scan).unwrap();
    }
    let conn = Connection::open(&db.0).unwrap();
    for table in [
        "snapshots",
        "files",
        "symbols",
        "search_index",
        "type_summary",
        "fingerprints",
    ] {
        let rows: i64 = conn
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(rows, 1, "table {table}");
    }
}
