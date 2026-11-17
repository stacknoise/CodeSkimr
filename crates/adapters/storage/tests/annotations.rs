//! Annotations, baselines and alerts of a project (ADR 0068).
#![cfg(test)]

#[cfg(test)]
mod common;

use std::num::NonZeroU32;

use codeskimr_application::ports::{
    AlertDismissal, AlertFilter, AlertPurge, AlertRaise, AnnotationCreation, AnnotationFilter,
    AnnotationStore, AnnotationUpdate, SnapshotStore,
};
use codeskimr_domain::{
    Alert, AlertChange, AlertKind, AlertSighting, AlertState, AlertTrigger, Annotation,
    AnnotationContent, AnnotationKind, AnnotationLimits, AnnotationTarget, Author, CommitHash,
    ContentHash, Loc, NewAnnotation, ProjectId, QualifiedName, ScanMode, SnapshotId,
    SymbolBaseline, SymbolKind, TaskStatus, Timestamp,
};
use codeskimr_storage::SqliteStore;
use common::{PROJECT, TempDb, code_of, data, file, header, project, store, symbol};
use rusqlite::Connection;

const LIMITS: AnnotationLimits = AnnotationLimits {
    max_text_chars: 100,
    max_title_chars: 20,
};
const DAY: i64 = 86_400;

/// A snapshot of the project 1 with the types `App.Order` and `App.Billing` and the method
/// `App.Order.Total`. Ids are derived from the snapshot id so that several snapshots can coexist.
fn scan(store: &SqliteStore, snapshot: u64) {
    let (file_id, first) = (snapshot + 1, snapshot * 10);
    let mut scan = data(header(snapshot, ScanMode::Full));
    scan.files = vec![file(
        snapshot,
        file_id,
        "src/order.cs",
        &format!("h{snapshot}"),
    )];
    scan.symbols = vec![
        symbol(first, file_id, SymbolKind::Class, "App.Order", None),
        symbol(
            first + 1,
            file_id,
            SymbolKind::Method,
            "App.Order.Total",
            Some(first),
        ),
        symbol(first + 2, file_id, SymbolKind::Class, "App.Billing", None),
    ];
    store.persist(scan).unwrap();
}

fn seeded(retention: u32) -> SqliteStore {
    let store = store(retention);
    store.save_project(&project(1, "/work/app", &[])).unwrap();
    scan(&store, 10);
    store
}

fn target(kind: SymbolKind, name: &str) -> AnnotationTarget {
    AnnotationTarget {
        kind,
        name: QualifiedName::new(name).unwrap(),
        parameters: Vec::new(),
    }
}

fn order() -> AnnotationTarget {
    target(SymbolKind::Class, "App.Order")
}

fn content(kind: AnnotationKind) -> AnnotationContent {
    let mut content = AnnotationContent {
        kind,
        title: None,
        text: None,
        status: None,
        due: None,
        assignee: None,
    };
    match kind {
        AnnotationKind::Critical => content.text = Some(LIMITS.text("pays the bills").unwrap()),
        AnnotationKind::Note => content.text = Some(LIMITS.text("check the rounding").unwrap()),
        AnnotationKind::Task => {
            content.title = Some(LIMITS.title("round up").unwrap());
            content.status = Some(TaskStatus::Open);
        }
    }
    content
}

fn creation(target: AnnotationTarget, kind: AnnotationKind) -> AnnotationCreation {
    AnnotationCreation {
        annotation: NewAnnotation {
            project: PROJECT,
            target,
            author: Author::new("anton").unwrap(),
            content: content(kind),
        },
        now: Timestamp(1_000),
    }
}

fn add(store: &SqliteStore, target: AnnotationTarget, kind: AnnotationKind) -> Annotation {
    store.create_annotation(&creation(target, kind)).unwrap()
}

fn baseline(hash: &str, cc: u32) -> SymbolBaseline {
    SymbolBaseline {
        content_hash: ContentHash::new(hash).unwrap(),
        signature_hash: ContentHash::new("sig").unwrap(),
        loc: Loc(12),
        cc,
    }
}

fn sighting(snapshot: u64) -> AlertSighting {
    AlertSighting {
        kind: AlertKind::Changed,
        snapshot: SnapshotId::new(snapshot),
        commit: Some(CommitHash::new("abc123").unwrap()),
        trigger: AlertTrigger::Scan,
        changes: vec![AlertChange::Content, AlertChange::Cc],
        references: None,
        closest: None,
    }
}

fn raise(store: &SqliteStore, annotation: &Annotation, snapshot: u64, now: i64) -> Alert {
    store
        .raise_alert(&AlertRaise {
            annotation: annotation.id,
            sighting: sighting(snapshot),
            now: Timestamp(now),
        })
        .unwrap()
}

fn alerts(store: &SqliteStore, state: Option<AlertState>) -> Vec<Alert> {
    store
        .list_alerts(&AlertFilter {
            project: PROJECT,
            state,
        })
        .unwrap()
}

fn all(store: &SqliteStore) -> Vec<Annotation> {
    store
        .list_annotations(&AnnotationFilter {
            project: PROJECT,
            target: None,
            kind: None,
        })
        .unwrap()
}

/// A critical annotation on `App.Order` with the baseline `h1` and an open alert.
fn watched_and_alerted(store: &SqliteStore) -> Annotation {
    let critical = add(store, order(), AnnotationKind::Critical);
    store
        .store_baseline(critical.id, &baseline("h1", 4))
        .unwrap();
    raise(store, &critical, 10, 2_000);
    critical
}

#[test]
fn every_kind_is_stored_and_listed_in_order() {
    let store = seeded(3);
    let critical = add(&store, order(), AnnotationKind::Critical);
    let note = add(&store, order(), AnnotationKind::Note);
    let mut task = creation(
        target(SymbolKind::Method, "App.Order.Total"),
        AnnotationKind::Task,
    );
    task.annotation.content.due = Some(Timestamp(5_000));
    task.annotation.content.assignee = Some(codeskimr_domain::Assignee::new("berta").unwrap());
    let task = store.create_annotation(&task).unwrap();

    assert_eq!(all(&store), vec![critical, note, task]);
}

#[test]
fn the_list_can_be_narrowed_by_symbol_and_kind() {
    let store = seeded(3);
    let critical = add(&store, order(), AnnotationKind::Critical);
    let note = add(&store, order(), AnnotationKind::Note);
    add(
        &store,
        target(SymbolKind::Class, "App.Billing"),
        AnnotationKind::Note,
    );
    let filter = |target, kind| AnnotationFilter {
        project: PROJECT,
        target,
        kind,
    };

    let of_order = store
        .list_annotations(&filter(Some(order()), None))
        .unwrap();
    assert_eq!(of_order, vec![critical, note.clone()]);
    let notes = store
        .list_annotations(&filter(Some(order()), Some(AnnotationKind::Note)))
        .unwrap();
    assert_eq!(notes, vec![note]);
    let tasks = store
        .list_annotations(&filter(None, Some(AnnotationKind::Task)))
        .unwrap();
    assert_eq!(tasks.len(), 0);
}

#[test]
fn parameter_types_tell_overloads_apart() {
    let store = seeded(3);
    let mut with_int = target(SymbolKind::Method, "App.Order.Total");
    with_int.parameters = vec!["int".to_owned(), "Map<K, V>".to_owned()];
    let first = add(&store, with_int.clone(), AnnotationKind::Critical);
    // Another overload of the same method is another symbol, so it can be critical as well.
    let second = add(
        &store,
        target(SymbolKind::Method, "App.Order.Total"),
        AnnotationKind::Critical,
    );

    assert_eq!(first.target.parameters, with_int.parameters);
    let found = store
        .list_annotations(&AnnotationFilter {
            project: PROJECT,
            target: Some(with_int),
            kind: None,
        })
        .unwrap();
    assert_eq!(found, vec![first]);
    assert_eq!(second.target.parameters.len(), 0);
}

#[test]
fn annotations_and_baselines_survive_new_snapshots_and_retention() {
    let store = seeded(2);
    let critical = watched_and_alerted(&store);
    let note = add(&store, order(), AnnotationKind::Note);

    scan(&store, 20);
    scan(&store, 30);
    scan(&store, 40);

    assert_eq!(
        store.latest_snapshot(PROJECT).unwrap().map(|s| s.id),
        Some(SnapshotId::new(40))
    );
    assert_eq!(all(&store), vec![critical.clone(), note]);
    assert_eq!(
        store.baseline(critical.id).unwrap(),
        Some(baseline("h1", 4))
    );
    assert_eq!(alerts(&store, Some(AlertState::Open)).len(), 1);
}

#[test]
fn deleting_a_project_deletes_its_annotations_baselines_and_alerts() {
    let db = TempDb::new();
    let store = SqliteStore::open(&db.0, NonZeroU32::new(3).unwrap()).unwrap();
    store.save_project(&project(1, "/work/app", &[])).unwrap();
    scan(&store, 10);
    watched_and_alerted(&store);

    let conn = Connection::open(&db.0).unwrap();
    conn.pragma_update(None, "foreign_keys", true).unwrap();
    conn.execute("DELETE FROM projects WHERE id = 1", [])
        .unwrap();

    for table in ["annotation", "annotation_baseline", "alert"] {
        let rows: i64 = conn
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(rows, 0, "{table}");
    }
}

#[test]
fn a_target_must_be_in_the_newest_snapshot() {
    let store = seeded(3);
    let error = store
        .create_annotation(&creation(
            target(SymbolKind::Class, "App.Gone"),
            AnnotationKind::Note,
        ))
        .unwrap_err();
    assert_eq!(code_of(&error), "ANN-101");
    // The same name with another kind is another symbol.
    let error = store
        .create_annotation(&creation(
            target(SymbolKind::Method, "App.Order"),
            AnnotationKind::Note,
        ))
        .unwrap_err();
    assert_eq!(code_of(&error), "ANN-101");
}

#[test]
fn an_unknown_project_is_refused() {
    let store = seeded(3);
    let mut other = creation(order(), AnnotationKind::Note);
    other.annotation.project = ProjectId::new(9);
    assert_eq!(
        code_of(&store.create_annotation(&other).unwrap_err()),
        "PRJ-201"
    );
}

#[test]
fn the_fields_must_fit_the_kind() {
    let store = seeded(3);
    let mut note = creation(order(), AnnotationKind::Note);
    note.annotation.content.text = None;
    assert_eq!(
        code_of(&store.create_annotation(&note).unwrap_err()),
        "ANN-103"
    );
    let mut critical = creation(order(), AnnotationKind::Critical);
    critical.annotation.content.due = Some(Timestamp(1));
    assert_eq!(
        code_of(&store.create_annotation(&critical).unwrap_err()),
        "ANN-103"
    );
    assert_eq!(all(&store).len(), 0);
}

#[test]
fn a_symbol_has_at_most_one_critical_annotation() {
    let store = seeded(3);
    add(&store, order(), AnnotationKind::Critical);

    let error = store
        .create_annotation(&creation(order(), AnnotationKind::Critical))
        .unwrap_err();
    assert_eq!(code_of(&error), "ANN-301");

    // Notes and tasks are not limited, and another symbol can be critical too.
    add(&store, order(), AnnotationKind::Note);
    add(&store, order(), AnnotationKind::Note);
    add(&store, order(), AnnotationKind::Task);
    let billing = add(
        &store,
        target(SymbolKind::Class, "App.Billing"),
        AnnotationKind::Critical,
    );
    assert_eq!(all(&store).len(), 5);

    // Moving a critical annotation onto a symbol that has one is refused as well.
    let mut moved = billing;
    moved.target = order();
    let error = store
        .update_annotation(&AnnotationUpdate {
            annotation: moved,
            now: Timestamp(2_000),
        })
        .unwrap_err();
    assert_eq!(code_of(&error), "ANN-301");
}

#[test]
fn an_update_changes_content_and_keeps_identity() {
    let store = seeded(3);
    let stored = add(&store, order(), AnnotationKind::Task);
    let mut changed = stored.clone();
    changed.content.status = Some(TaskStatus::Done);
    changed.content.assignee = Some(codeskimr_domain::Assignee::new("berta").unwrap());
    changed.author = Author::new("someone else").unwrap();

    let updated = store
        .update_annotation(&AnnotationUpdate {
            annotation: changed,
            now: Timestamp(3_000),
        })
        .unwrap();

    assert_eq!(updated.author, stored.author);
    assert_eq!(updated.created, stored.created);
    assert_eq!(updated.updated, Timestamp(3_000));
    assert_eq!(all(&store), vec![updated]);
}

#[test]
fn the_kind_of_an_annotation_cannot_change() {
    let store = seeded(3);
    let mut changed = add(&store, order(), AnnotationKind::Note);
    changed.content = content(AnnotationKind::Task);
    let error = store
        .update_annotation(&AnnotationUpdate {
            annotation: changed,
            now: Timestamp(2_000),
        })
        .unwrap_err();
    assert_eq!(code_of(&error), "ANN-103");
}

#[test]
fn reassigning_a_critical_annotation_drops_the_baseline_of_the_old_symbol() {
    let store = seeded(3);
    let mut critical = add(&store, order(), AnnotationKind::Critical);
    store
        .store_baseline(critical.id, &baseline("h1", 4))
        .unwrap();
    critical.target = target(SymbolKind::Class, "App.Billing");

    let moved = store
        .update_annotation(&AnnotationUpdate {
            annotation: critical,
            now: Timestamp(2_000),
        })
        .unwrap();

    assert_eq!(moved.target.name.as_str(), "App.Billing");
    assert_eq!(store.baseline(moved.id).unwrap(), None);
}

#[test]
fn an_orphaned_note_stays_editable_without_its_symbol() {
    let store = seeded(3);
    let mut note = add(&store, order(), AnnotationKind::Note);
    let mut next = data(header(20, ScanMode::Full));
    next.files = vec![file(20, 21, "src/other.cs", "h20")];
    next.symbols = vec![symbol(200, 21, SymbolKind::Class, "App.Billing", None)];
    store.persist(next).unwrap();

    note.content.text = Some(LIMITS.text("symbol is gone").unwrap());
    let updated = store
        .update_annotation(&AnnotationUpdate {
            annotation: note,
            now: Timestamp(2_000),
        })
        .unwrap();
    assert_eq!(updated.target, order());
}

#[test]
fn unknown_annotations_and_alerts_are_reported() {
    let store = seeded(3);
    let missing = codeskimr_domain::AnnotationId::new(77);
    assert_eq!(
        code_of(&store.delete_annotation(missing).unwrap_err()),
        "ANN-201"
    );
    assert_eq!(code_of(&store.baseline(missing).unwrap_err()), "ANN-201");
    let raised = store.raise_alert(&AlertRaise {
        annotation: missing,
        sighting: sighting(10),
        now: Timestamp(1),
    });
    assert_eq!(code_of(&raised.unwrap_err()), "ANN-201");
    let dismissal = AlertDismissal {
        alert: codeskimr_domain::AlertId::new(5),
        baseline: None,
        now: Timestamp(1),
    };
    assert_eq!(
        code_of(&store.dismiss_alert(&dismissal).unwrap_err()),
        "ANN-202"
    );
}

#[test]
fn only_critical_annotations_have_baselines_and_alerts() {
    let store = seeded(3);
    let note = add(&store, order(), AnnotationKind::Note);
    let error = store
        .store_baseline(note.id, &baseline("h1", 1))
        .unwrap_err();
    assert_eq!(code_of(&error), "ANN-103");
    let error = store
        .raise_alert(&AlertRaise {
            annotation: note.id,
            sighting: sighting(10),
            now: Timestamp(1),
        })
        .unwrap_err();
    assert_eq!(code_of(&error), "ANN-103");
}

#[test]
fn a_critical_annotation_without_baseline_reads_none_and_a_new_one_replaces_the_old() {
    let store = seeded(3);
    let critical = add(&store, order(), AnnotationKind::Critical);
    assert_eq!(store.baseline(critical.id).unwrap(), None);
    store
        .store_baseline(critical.id, &baseline("h1", 4))
        .unwrap();
    store
        .store_baseline(critical.id, &baseline("h2", 7))
        .unwrap();
    assert_eq!(
        store.baseline(critical.id).unwrap(),
        Some(baseline("h2", 7))
    );
}

#[test]
fn there_is_one_open_alert_per_annotation_and_it_counts() {
    let store = seeded(3);
    let critical = add(&store, order(), AnnotationKind::Critical);

    let first = raise(&store, &critical, 10, 2_000);
    assert_eq!(
        (first.counter, first.first_seen, first.last_seen),
        (1, Timestamp(2_000), Timestamp(2_000))
    );
    assert_eq!(first.state, AlertState::Open);

    let mut again = AlertRaise {
        annotation: critical.id,
        sighting: sighting(20),
        now: Timestamp(3_000),
    };
    again.sighting.kind = AlertKind::Removed;
    again.sighting.changes = Vec::new();
    again.sighting.references = Some(3);
    again.sighting.trigger = AlertTrigger::Watcher;
    let second = store.raise_alert(&again).unwrap();

    assert_eq!(second.id, first.id);
    assert_eq!(second.counter, 2);
    assert_eq!(
        (second.first_seen, second.last_seen),
        (Timestamp(2_000), Timestamp(3_000))
    );
    assert_eq!(second.sighting, again.sighting);
    assert_eq!(alerts(&store, None), vec![second]);
}

#[test]
fn an_orphaned_alert_names_the_closest_symbol() {
    let store = seeded(3);
    let critical = add(&store, order(), AnnotationKind::Critical);
    let mut orphaned = AlertRaise {
        annotation: critical.id,
        sighting: sighting(10),
        now: Timestamp(2_000),
    };
    orphaned.sighting.kind = AlertKind::Orphaned;
    orphaned.sighting.changes = Vec::new();
    orphaned.sighting.commit = None;
    orphaned.sighting.closest = Some(QualifiedName::new("Other.Order").unwrap());

    let stored = store.raise_alert(&orphaned).unwrap();
    assert_eq!(stored.sighting, orphaned.sighting);
}

#[test]
fn dismissing_sets_the_state_and_the_baseline_and_a_later_change_alerts_again() {
    let store = seeded(3);
    let critical = watched_and_alerted(&store);
    let alert = alerts(&store, None).remove(0);

    let dismissed = store
        .dismiss_alert(&AlertDismissal {
            alert: alert.id,
            baseline: Some(baseline("new", 9)),
            now: Timestamp(2_500),
        })
        .unwrap();

    assert_eq!(dismissed.state, AlertState::Dismissed);
    assert_eq!(dismissed.dismissed, Some(Timestamp(2_500)));
    assert_eq!(
        store.baseline(critical.id).unwrap(),
        Some(baseline("new", 9))
    );
    assert_eq!(alerts(&store, Some(AlertState::Open)).len(), 0);
    assert_eq!(alerts(&store, Some(AlertState::Dismissed)), vec![dismissed]);

    let next = raise(&store, &critical, 20, 4_000);
    assert_ne!(next.id, alert.id);
    assert_eq!(next.counter, 1);
    assert_eq!(alerts(&store, None).len(), 2);
}

#[test]
fn dismissing_a_removed_symbol_keeps_its_baseline_and_dismissing_twice_changes_nothing() {
    let store = seeded(3);
    let critical = watched_and_alerted(&store);
    let alert = alerts(&store, None).remove(0);
    let dismissal = |baseline, now| AlertDismissal {
        alert: alert.id,
        baseline,
        now: Timestamp(now),
    };

    let first = store.dismiss_alert(&dismissal(None, 2_500)).unwrap();
    assert_eq!(
        store.baseline(critical.id).unwrap(),
        Some(baseline("h1", 4))
    );
    let second = store
        .dismiss_alert(&dismissal(Some(baseline("late", 1)), 9_000))
        .unwrap();
    assert_eq!(second, first);
    assert_eq!(
        store.baseline(critical.id).unwrap(),
        Some(baseline("h1", 4))
    );
}

#[test]
fn alerts_are_listed_by_state_newest_first_and_only_for_their_project() {
    let store = seeded(3);
    store.save_project(&project(2, "/work/other", &[])).unwrap();
    let billing = add(
        &store,
        target(SymbolKind::Class, "App.Billing"),
        AnnotationKind::Critical,
    );
    let critical = add(&store, order(), AnnotationKind::Critical);
    let old = raise(&store, &billing, 10, 2_000);
    let new = raise(&store, &critical, 10, 3_000);
    store
        .dismiss_alert(&AlertDismissal {
            alert: old.id,
            baseline: None,
            now: Timestamp(3_500),
        })
        .unwrap();

    let open = alerts(&store, Some(AlertState::Open));
    assert_eq!(
        open.iter().map(|alert| alert.id).collect::<Vec<_>>(),
        vec![new.id]
    );
    let everything = alerts(&store, None);
    assert_eq!(
        everything.iter().map(|alert| alert.id).collect::<Vec<_>>(),
        vec![new.id, old.id]
    );
    let other = store
        .list_alerts(&AlertFilter {
            project: ProjectId::new(2),
            state: None,
        })
        .unwrap();
    assert_eq!(other.len(), 0);
}

#[test]
fn only_dismissed_alerts_older_than_the_retention_are_purged() {
    let store = seeded(3);
    let billing = add(
        &store,
        target(SymbolKind::Class, "App.Billing"),
        AnnotationKind::Critical,
    );
    let critical = add(&store, order(), AnnotationKind::Critical);
    let dismissed = raise(&store, &billing, 10, 1_000);
    raise(&store, &critical, 10, 1_000);
    let at = 10 * DAY;
    store
        .dismiss_alert(&AlertDismissal {
            alert: dismissed.id,
            baseline: None,
            now: Timestamp(at),
        })
        .unwrap();
    let purge = |days_later: i64| AlertPurge {
        now: Timestamp(at + days_later * DAY),
        retention_days: 30,
    };

    assert_eq!(store.purge_dismissed(&purge(29)).unwrap(), 0);
    assert_eq!(store.purge_dismissed(&purge(30)).unwrap(), 0);
    assert_eq!(store.purge_dismissed(&purge(31)).unwrap(), 1);
    // The open alert is never purged, however old, and nothing is left to purge.
    assert_eq!(store.purge_dismissed(&purge(1_000)).unwrap(), 0);
    assert_eq!(alerts(&store, None).len(), 1);
}

#[test]
fn deleting_an_annotation_deletes_its_baseline_and_alerts() {
    let store = seeded(3);
    let critical = watched_and_alerted(&store);

    store.delete_annotation(critical.id).unwrap();

    assert_eq!(all(&store).len(), 0);
    assert_eq!(alerts(&store, None).len(), 0);
    assert_eq!(
        code_of(&store.baseline(critical.id).unwrap_err()),
        "ANN-201"
    );
    // The symbol can be marked again.
    add(&store, order(), AnnotationKind::Critical);
}
