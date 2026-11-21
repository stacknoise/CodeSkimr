//! The retention of dismissed alerts is wired: the purge runs at the start of the server and the
//! deletion follows `annotations.dismissed_retention_days` (ADR 0068).

use std::sync::Arc;
use std::time::{Duration, Instant};

use codeskimr_application::events::EventBus;
use codeskimr_application::ports::{AlertDismissal, AlertFilter, AnnotationStore, Clock};
use codeskimr_cli::GlobalOptions;
use codeskimr_domain::Timestamp as Time;
use codeskimr_domain::{AlertState, ScanMode, SymbolKind, Timestamp};
use codeskimr_storage::SqliteStore;

use crate::alert_tests::{critical, open_alerts, watched};
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
    // `store` and `catalog` are two handles on the same database file under `data`.
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

/// A clock that stands still, to try the edge of the retention.
struct Fixed(i64);

impl Clock for Fixed {
    fn now(&self) -> Timestamp {
        Timestamp(self.0)
    }
}

const DAY: i64 = 86_400;

#[test]
fn the_retention_keeps_29_days_and_30_and_deletes_31_and_spares_open_alerts() {
    let root = tempfile::tempdir().unwrap();
    let (folder, session, store) = watched(&root, SymbolKind::Class, "Demo.Target");
    critical(&store, SymbolKind::Class, "Demo.Other");
    std::fs::remove_file(folder.join("Target.cs")).unwrap();
    std::fs::remove_file(folder.join("Other.cs")).unwrap();
    session.scan_with(ScanMode::Full, |_| {}).unwrap();
    let alerts = open_alerts(&store);
    assert_eq!(alerts.len(), 2);
    let at = 100 * DAY;
    store
        .dismiss_alert(&AlertDismissal {
            alert: alerts[0].id,
            baseline: None,
            now: Time(at),
        })
        .unwrap();

    assert_eq!(
        purge_expired(&*store, &Fixed(at + 29 * DAY), 30).unwrap(),
        0
    );
    assert_eq!(
        purge_expired(&*store, &Fixed(at + 30 * DAY), 30).unwrap(),
        0
    );
    assert_eq!(dismissed(&store), 1);
    assert_eq!(
        purge_expired(&*store, &Fixed(at + 31 * DAY), 30).unwrap(),
        1
    );
    assert_eq!(dismissed(&store), 0);
    // The open alert is still there, however old the clock says it is.
    assert_eq!(open_alerts(&store).len(), 1);
}
