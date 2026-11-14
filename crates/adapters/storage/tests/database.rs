//! The database file: reopening, migrations and pruning on disk.

#[cfg(test)]
mod common;

use std::num::NonZeroU32;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use codeskimr_application::ports::{
    ProjectRepository, SnapshotData, SnapshotStore, SymbolFingerprints,
};
use codeskimr_domain::{Fingerprint, Fingerprints, MetricKind, ScanMode, SymbolId, SymbolKind};
use codeskimr_storage::SqliteStore;
use common::{PROJECT, code_of, data, file, header, metric, project, reference, symbol};
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
    assert_eq!((mode.as_str(), version), ("wal", 9));
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

/// A scan with one file, a class with a method, a reference, metrics and a fingerprint.
fn scan_with_every_table(snapshot: u64, base: u64) -> SnapshotData {
    let mut scan = data(header(snapshot, ScanMode::Full));
    scan.files = vec![file(snapshot, base, "a.cs", "h")];
    scan.symbols = vec![
        symbol(base + 1, base, SymbolKind::Class, "App.A", None),
        symbol(
            base + 2,
            base,
            SymbolKind::Method,
            "App.A.M",
            Some(base + 1),
        ),
    ];
    scan.references = vec![reference(base + 3, base, base + 2, base + 1)];
    scan.metrics = vec![
        metric(base + 1, MetricKind::Loc, 10.0),
        metric(base + 2, MetricKind::CyclomaticComplexity, 2.0),
        metric(base + 1, MetricKind::EfferentCoupling, 1.0),
    ];
    scan.fingerprints = vec![SymbolFingerprints {
        symbol: SymbolId::new(base + 1),
        fingerprints: Fingerprints::from_entries(vec![Fingerprint {
            hash: base,
            position: 0,
        }]),
    }];
    scan
}

#[test]
fn pruned_snapshots_leave_no_rows_behind() {
    let db = TempDb::new();
    let store = SqliteStore::open(&db.0, keep(1)).unwrap();
    store.save_project(&project(1, "/work/app", &[])).unwrap();
    for (snapshot, base) in [(10, 100), (20, 200)] {
        store
            .persist(scan_with_every_table(snapshot, base))
            .unwrap();
    }
    let conn = Connection::open(&db.0).unwrap();
    for (table, expected) in [
        ("snapshots", 1),
        ("files", 1),
        ("symbols", 2),
        ("symbol_references", 1),
        ("metrics", 3),
        ("symbol_rows", 2),
        ("metric_rows", 2),
        ("coupling_rows", 1),
        ("search_index", 2),
        ("usage_counts", 1),
        ("type_summary", 1),
        ("fingerprints", 1),
    ] {
        let rows: i64 = conn
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(rows, expected, "table {table}");
    }
}

/// Number of rows of a table.
#[cfg(test)]
fn rows_of(conn: &Connection, table: &str) -> i64 {
    conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
        row.get(0)
    })
    .unwrap()
}

/// A scan of one file `a.cs` (id `base`) with two symbols, a reference between them and a
/// fingerprint, followed by an incremental scan that adds `b.cs` with a reference into `a.cs`.
#[cfg(test)]
fn full_then_incremental(store: &SqliteStore) {
    store.persist(scan_with_every_table(10, 100)).unwrap();
    let mut next = data(header(20, ScanMode::Incremental));
    next.files = vec![file(20, 200, "b.cs", "h")];
    next.symbols = vec![symbol(201, 200, SymbolKind::Class, "App.B", None)];
    next.references = vec![reference(202, 200, 201, 101)];
    store.persist(next).unwrap();
}

#[test]
fn an_incremental_scan_shares_references_and_fingerprints_instead_of_copying_them() {
    let db = TempDb::new();
    let store = SqliteStore::open(&db.0, keep(5)).unwrap();
    store.save_project(&project(1, "/work/app", &[])).unwrap();
    full_then_incremental(&store);
    let conn = Connection::open(&db.0).unwrap();
    // One reference of the first scan, one new one; the fingerprint is stored once.
    assert_eq!(rows_of(&conn, "symbol_references"), 2);
    assert_eq!(rows_of(&conn, "fingerprints"), 1);
    // The symbols and metrics of `a.cs` exist once: two of the first scan, one of the second.
    assert_eq!(rows_of(&conn, "symbol_rows"), 3);
    assert_eq!(rows_of(&conn, "metric_rows"), 2);
    let seen = |snapshot: i64| -> i64 {
        conn.query_row(
            "SELECT COUNT(*) FROM snapshot_references WHERE snapshot_id = ?1",
            [snapshot],
            |row| row.get(0),
        )
        .unwrap()
    };
    assert_eq!(seen(10), 1);
    assert_eq!(seen(20), 2);
}

#[test]
fn pruning_keeps_the_shared_rows_another_snapshot_still_contains() {
    let db = TempDb::new();
    let store = SqliteStore::open(&db.0, keep(1)).unwrap();
    store.save_project(&project(1, "/work/app", &[])).unwrap();
    full_then_incremental(&store);
    let conn = Connection::open(&db.0).unwrap();
    // Only snapshot 20 is left: it still contains the carried file with its reference and
    // fingerprint.
    assert_eq!(rows_of(&conn, "snapshots"), 1);
    assert_eq!(rows_of(&conn, "symbol_references"), 2);
    assert_eq!(rows_of(&conn, "fingerprints"), 1);
    store.persist(scan_with_every_table(30, 300)).unwrap();
    // A full scan shares nothing with the snapshot it replaces.
    assert_eq!(rows_of(&conn, "symbol_references"), 1);
    assert_eq!(rows_of(&conn, "fingerprints"), 1);
}

/// A database that has run only `migrations`, at the schema version they make.
#[cfg(test)]
fn database_at_version(db: &TempDb, migrations: &[&str]) -> Connection {
    std::fs::create_dir_all(db.0.parent().unwrap()).unwrap();
    let conn = Connection::open(&db.0).unwrap();
    for migration in migrations {
        conn.execute_batch(migration).unwrap();
    }
    let version = i64::try_from(migrations.len()).unwrap();
    conn.pragma_update(None, "user_version", version).unwrap();
    conn
}

/// A version three database in which both snapshots hold the same file, symbols, reference and
/// fingerprint, as a carry copied them.
#[cfg(test)]
fn version_three_database(db: &TempDb) {
    let conn = database_at_version(
        db,
        &[
            include_str!("../migrations/0001_initial.sql"),
            include_str!("../migrations/0002_credential_reference.sql"),
            include_str!("../migrations/0003_fingerprints.sql"),
        ],
    );
    conn.execute_batch(
        "INSERT INTO projects (id, name, source_kind, location, branch)
           VALUES (1, 'old', 'folder', '/work/app', '');
         INSERT INTO snapshots (id, project_id, created, mode, files, symbols, refs)
           VALUES (10, 1, 0, 'full', 1, 2, 1), (20, 1, 1, 'incremental', 1, 2, 1);",
    )
    .unwrap();
    for snapshot in [10, 20] {
        conn.execute_batch(&format!(
            "INSERT INTO files (snapshot_id, id, path, language, content_hash, loc)
               VALUES ({snapshot}, 100, 'a.cs', 'csharp', 'h', 1);
             INSERT INTO symbols (snapshot_id, id, file_id, kind, name, qualified_name, parent_id,
               visibility, is_static, start_line, start_column, end_line, end_column, signature, doc)
               VALUES ({snapshot}, 101, 100, 'class', 'A', 'App.A', NULL, NULL, 0, 1, 0, 1, 1, NULL, NULL),
                      ({snapshot}, 102, 100, 'method', 'M', 'App.A.M', 101, NULL, 0, 1, 0, 1, 1, NULL, NULL);
             INSERT INTO symbol_references (snapshot_id, id, from_symbol, to_symbol, file_id,
               start_line, start_column, end_line, end_column, kind, confidence)
               VALUES ({snapshot}, 103, 102, 101, 100, 1, 0, 1, 1, 'call', 'certain');
             INSERT INTO fingerprints (snapshot_id, symbol_id, position, hash)
               VALUES ({snapshot}, 101, 0, 7);"
        ))
        .unwrap();
    }
    version_three_second_scan(&conn);
}

/// What only snapshot 20 of the version three database has: a new file with a class, a reference,
/// and a different value of a metric the two snapshots share.
#[cfg(test)]
fn version_three_second_scan(conn: &Connection) {
    conn.execute_batch(
        "INSERT INTO metrics (snapshot_id, symbol_id, kind, value)
           VALUES (10, 101, 'loc', 1.0), (20, 101, 'loc', 2.0);
         INSERT INTO files (snapshot_id, id, path, language, content_hash, loc)
           VALUES (20, 200, 'b.cs', 'csharp', 'h', 1);
         INSERT INTO symbols (snapshot_id, id, file_id, kind, name, qualified_name, parent_id,
           visibility, is_static, start_line, start_column, end_line, end_column, signature, doc)
           VALUES (20, 201, 200, 'class', 'B', 'App.B', NULL, NULL, 0, 1, 0, 1, 1, NULL, NULL);
         INSERT INTO symbol_references (snapshot_id, id, from_symbol, to_symbol, file_id,
           start_line, start_column, end_line, end_column, kind, confidence)
           VALUES (20, 204, 201, 101, 200, 1, 0, 1, 1, 'call', 'certain');",
    )
    .unwrap();
}

/// Rows of a view or table that a snapshot contains.
#[cfg(test)]
fn rows_in(conn: &Connection, table: &str, snapshot: i64) -> i64 {
    conn.query_row(
        &format!("SELECT COUNT(*) FROM {table} WHERE snapshot_id = ?1"),
        [snapshot],
        |row| row.get(0),
    )
    .unwrap()
}

#[test]
fn a_version_three_database_keeps_what_each_snapshot_contained() {
    let db = TempDb::new();
    version_three_database(&db);
    drop(SqliteStore::open(&db.0, keep(5)).unwrap());
    let conn = Connection::open(&db.0).unwrap();
    // Each row exists once; the reference of the second scan is not seen by the first.
    assert_eq!(rows_of(&conn, "symbol_references"), 2);
    assert_eq!(rows_of(&conn, "symbol_rows"), 3);
    assert_eq!(rows_of(&conn, "fingerprints"), 1);
    assert_eq!(rows_in(&conn, "snapshot_references", 10), 1);
    assert_eq!(rows_in(&conn, "snapshot_references", 20), 2);
    assert_eq!(rows_in(&conn, "symbols", 10), 2);
    assert_eq!(rows_in(&conn, "symbols", 20), 3);
    // The metric the snapshots shared has the value of the newest one.
    let loc: f64 = conn
        .query_row(
            "SELECT value FROM metrics WHERE snapshot_id = 10 AND symbol_id = 101",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!((loc - 2.0).abs() < f64::EPSILON);
}

#[test]
fn pruning_right_after_a_migration_leaves_only_the_newest_snapshot() {
    let db = TempDb::new();
    version_three_database(&db);
    let store = SqliteStore::open(&db.0, keep(1)).unwrap();
    store.persist(scan_with_every_table(30, 300)).unwrap();
    let conn = Connection::open(&db.0).unwrap();
    for (table, expected) in [
        ("snapshots", 1),
        ("symbol_rows", 2),
        ("metric_rows", 2),
        ("symbol_references", 1),
        ("fingerprints", 1),
    ] {
        assert_eq!(rows_of(&conn, table), expected, "table {table}");
    }
}

#[test]
fn a_rebuilt_reference_of_a_carried_file_is_not_seen_by_the_older_snapshot() {
    let db = TempDb::new();
    let store = SqliteStore::open(&db.0, keep(5)).unwrap();
    store.save_project(&project(1, "/work/app", &[])).unwrap();
    store.persist(scan_with_every_table(10, 100)).unwrap();
    let mut next = data(header(20, ScanMode::Incremental));
    next.files = vec![file(20, 200, "b.cs", "h")];
    next.symbols = vec![symbol(201, 200, SymbolKind::Class, "App.B", None)];
    next.references = vec![reference(204, 100, 102, 101)];
    store.persist(next).unwrap();
    let conn = Connection::open(&db.0).unwrap();
    assert_eq!(rows_in(&conn, "snapshot_references", 10), 1);
    assert_eq!(rows_in(&conn, "snapshot_references", 20), 2);
}

/// The ways a scan can point at rows that it does not hold.
#[cfg(test)]
fn corruptions() -> Vec<fn(&mut SnapshotData)> {
    vec![
        |scan| {
            scan.symbols
                .push(symbol(900, 999, SymbolKind::Class, "X", None));
        },
        |scan| {
            scan.symbols
                .push(symbol(900, 100, SymbolKind::Method, "X.M", Some(999)));
        },
        |scan| scan.references.push(reference(901, 100, 998, 101)),
        |scan| scan.references.push(reference(901, 999, 102, 101)),
        |scan| scan.metrics.push(metric(997, MetricKind::Loc, 1.0)),
        |scan| {
            scan.fingerprints.push(SymbolFingerprints {
                symbol: SymbolId::new(996),
                fingerprints: Fingerprints::from_entries(vec![Fingerprint {
                    hash: 1,
                    position: 0,
                }]),
            });
        },
    ]
}

#[test]
fn a_snapshot_with_a_row_that_points_nowhere_is_refused_and_leaves_nothing_behind() {
    for (index, corrupt) in corruptions().into_iter().enumerate() {
        let db = TempDb::new();
        let store = SqliteStore::open(&db.0, keep(5)).unwrap();
        store.save_project(&project(1, "/work/app", &[])).unwrap();
        let mut scan = scan_with_every_table(10, 100);
        corrupt(&mut scan);
        let error = store.persist(scan).unwrap_err();
        assert_eq!(code_of(&error), "STO-901", "corruption {index}");
        let conn = Connection::open(&db.0).unwrap();
        for table in [
            "symbol_rows",
            "metric_rows",
            "symbol_references",
            "fingerprints",
        ] {
            assert_eq!(
                rows_of(&conn, table),
                0,
                "corruption {index}, table {table}"
            );
        }
    }
}

#[test]
fn the_verified_mark_of_a_reference_survives_storing_and_loading() {
    let db = TempDb::new();
    let store = SqliteStore::open(&db.0, keep(3)).unwrap();
    store.save_project(&project(1, "/work/app", &[])).unwrap();
    let mut scan = scan_with_every_table(10, 100);
    scan.references[0].verified = true;
    store.persist(scan).unwrap();
    let conn = Connection::open(&db.0).unwrap();
    let verified: i64 = conn
        .query_row(
            "SELECT verified FROM snapshot_references WHERE snapshot_id = 10",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(verified, 1);
}

#[test]
fn a_version_five_database_gets_the_tool_report_tables() {
    use codeskimr_application::ports::ToolReportStore;
    let db = TempDb::new();
    let conn = database_at_version(
        &db,
        &[
            include_str!("../migrations/0001_initial.sql"),
            include_str!("../migrations/0002_credential_reference.sql"),
            include_str!("../migrations/0003_fingerprints.sql"),
            include_str!("../migrations/0004_shared_rows.sql"),
            include_str!("../migrations/0005_reference_verified.sql"),
        ],
    );
    drop(conn);

    let store = SqliteStore::open(&db.0, keep(3)).unwrap();
    store.save_project(&project(1, "/work/app", &[])).unwrap();
    assert_eq!(store.load(PROJECT).unwrap(), None);
}

#[test]
fn a_version_seven_database_has_no_naming_rules_for_its_old_snapshots() {
    let db = TempDb::new();
    let conn = database_at_version(
        &db,
        &[
            include_str!("../migrations/0001_initial.sql"),
            include_str!("../migrations/0002_credential_reference.sql"),
            include_str!("../migrations/0003_fingerprints.sql"),
            include_str!("../migrations/0004_shared_rows.sql"),
            include_str!("../migrations/0005_reference_verified.sql"),
            include_str!("../migrations/0006_level2_consent.sql"),
            include_str!("../migrations/0007_tool_reports.sql"),
        ],
    );
    conn.execute(
        "INSERT INTO projects (id, name, source_kind, location, branch) VALUES (?1, ?2, ?3, ?4, ?5)",
        (1, "older", "folder", "/work/legacy", ""),
    )
    .unwrap();
    conn.execute(
        "INSERT INTO snapshots (id, project_id, created, mode, files, symbols, refs) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?5, ?5)",
        (10, 1, 0, "full", 0),
    )
    .unwrap();
    drop(conn);
    let store = SqliteStore::open(&db.0, keep(3)).unwrap();
    assert_eq!(store.latest_naming_rules(PROJECT).unwrap(), None);
    assert_eq!(
        store.latest_snapshot(PROJECT).unwrap().unwrap().id.get(),
        10
    );
}

#[test]
fn connections_registering_at_once_never_share_a_project_id() {
    let db = TempDb::new();
    let workers: Vec<_> = (0..4)
        .map(|worker| {
            let path = db.0.clone();
            std::thread::spawn(move || {
                let store = SqliteStore::open(&path, keep(3)).unwrap();
                (0..5)
                    .map(|round| {
                        let spec = project(0, &format!("/work/{worker}/{round}"), &[]);
                        store.register_project(spec.name, spec.source).unwrap().id
                    })
                    .collect::<Vec<_>>()
            })
        })
        .collect();
    let mut ids: Vec<_> = workers
        .into_iter()
        .flat_map(|worker| worker.join().unwrap())
        .collect();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), 20, "every source got its own id");
}

#[test]
fn connections_opening_a_new_database_at_once_all_succeed() {
    let db = TempDb::new();
    let workers: Vec<_> = (0..6)
        .map(|_| {
            let path = db.0.clone();
            std::thread::spawn(move || SqliteStore::open(&path, keep(3)).map(drop))
        })
        .collect();
    for worker in workers {
        worker.join().unwrap().unwrap();
    }
}
