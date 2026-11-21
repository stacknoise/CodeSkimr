//! The retention of dismissed alerts is wired: the purge runs at the start of the server and the
//! deletion follows `annotations.dismissed_retention_days` (ADR 0068).

use std::sync::Arc;
use std::time::{Duration, Instant};

use codeskimr_application::events::EventBus;
use codeskimr_application::ports::{AlertDismissal, AlertFilter, AnnotationStore, Clock};
use codeskimr_cli::GlobalOptions;
use codeskimr_domain::{AlertState, ScanMode, SymbolKind, Timestamp};
use codeskimr_storage::SqliteStore;

use crate::alert_tests::{open_alerts, watched};
use crate::clock::SystemClock;
use crate::config_files;
use crate::mcp_http_tests::locations;
use crate::retention::{DailyPurge, purge_expired};
use crate::running::RunningScans;
use crate::serve::serving_parts;
use crate::wiring::open_store;

const PROJECT: codeskimr_domain::ProjectId = codeskimr_domain::ProjectId::new(1);
const WAIT: Duration = Duration::from_secs(20);

fn dismissed(store: &SqliteStore) -> usize {
    store
        .list_alerts(&AlertFilter {
            project: PROJECT,
            state: Some(AlertState::Dismissed),
        })
        .unwrap()
        .len()
}

/// A removed critical class whose alert was dismissed at `at`.
fn dismissed_at(root: &tempfile::TempDir, at: i64) -> Arc<SqliteStore> {
    let (folder, session, store) = watched(root, SymbolKind::Class, "Demo.Target");
    std::fs::remove_file(folder.join("Target.cs")).unwrap();
    session.scan_with(ScanMode::Full, |_| {}).unwrap();
    let alert = open_alerts(&store).remove(0);
    store
        .dismiss_alert(&AlertDismissal {
            alert: alert.id,
            baseline: None,
            now: Timestamp(at),
        })
        .unwrap();
    assert_eq!(dismissed(&store), 1);
    store
}

fn wait_until(condition: impl Fn() -> bool) {
    let start = Instant::now();
    while !condition() {
        assert!(start.elapsed() < WAIT, "the purge did not run");
        std::thread::sleep(Duration::from_millis(20));
    }
}

#[test]
fn only_alerts_dismissed_before_the_retention_are_deleted() {
    let root = tempfile::tempdir().unwrap();
    let store = dismissed_at(&root, SystemClock.now().0);
    assert_eq!(purge_expired(&*store, &SystemClock, 30).unwrap(), 0);
    assert_eq!(dismissed(&store), 1);
    // Dismissed at the epoch: far older than 30 days.
    let old = tempfile::tempdir().unwrap();
    let store = dismissed_at(&old, 1);
    assert_eq!(purge_expired(&*store, &SystemClock, 30).unwrap(), 1);
    assert_eq!(dismissed(&store), 0);
}

#[test]
fn the_server_purges_expired_alerts_when_it_starts() {
    let root = tempfile::tempdir().unwrap();
    let store = dismissed_at(&root, 1);
    let data = locations(&root.path().join("data"));
    let global = GlobalOptions::default();
    let config = config_files::load(&data, &global, None).unwrap();
    let catalog = open_store(&data, &config).unwrap();
    let parts = serving_parts(
        &catalog,
        &config,
        (&global, &data),
        (EventBus::new(), &crate::memory_keystore::secrets()),
        RunningScans::default(),
    )
    .unwrap();
    wait_until(|| dismissed(&store) == 0);
    drop(parts);
}

#[test]
fn the_timer_purges_again_after_its_interval() {
    let root = tempfile::tempdir().unwrap();
    let store = dismissed_at(&root, 1);
    let _timer = DailyPurge::start(store.clone(), 30, Duration::from_millis(10));
    wait_until(|| dismissed(&store) == 0);
}
