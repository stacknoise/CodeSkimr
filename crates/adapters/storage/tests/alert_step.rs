//! The alert step over the real store (ADR 0068): content hashes that are stored with the symbols,
//! the lookups of the targets, and the alerts written for real snapshots.
#![cfg(test)]

#[cfg(test)]
mod common;

use std::sync::Arc;

use codeskimr_application::alerts::AlertStep;
use codeskimr_application::error::CodedError;
use codeskimr_application::events::EventBus;
use codeskimr_application::ports::{
    AlertDismissal, AlertFilter, AnnotationCreation, AnnotationFilter, AnnotationStore, Clock,
    ContentDigest, PendingReference, SnapshotStore, SymbolHash, SymbolStates,
};
use codeskimr_domain::{
    Alert, AlertChange, AlertKind, AlertState, AlertTrigger, Annotation, AnnotationContent,
    AnnotationKind, AnnotationTarget, Author, ContentHash, FileId, MetricKind, NewAnnotation,
    QualifiedName, ReferenceKind, ScanMode, SnapshotId, SymbolBaseline, SymbolId, SymbolKind,
    SymbolName, Timestamp,
};
use codeskimr_storage::SqliteStore;
use common::{PROJECT, data, file, header, metric, project, span, store, symbol};

/// Joins its parts: readable, and different for different parts.
struct Joined;

impl ContentDigest for Joined {
    fn digest(&self, parts: &[&str]) -> Result<ContentHash, CodedError> {
        ContentHash::new(format!("d[{}]", parts.join("|"))).map_err(CodedError::caused_by)
    }
}

struct Now;

impl Clock for Now {
    fn now(&self) -> Timestamp {
        Timestamp(5_000)
    }
}

fn step(store: &Arc<SqliteStore>) -> AlertStep {
    AlertStep {
        annotations: store.clone(),
        states: store.clone(),
        digest: Arc::new(Joined),
        clock: Arc::new(Now),
        events: EventBus::new(),
    }
}

/// What a snapshot of the test holds: the hash of the method `App.Order.Total`, the hash of the
/// class `App.Order` and whether the method is still there.
#[derive(Clone)]
struct Version {
    total: &'static str,
    order: &'static str,
    keep_total: bool,
    loc: f64,
}

fn version(total: &'static str) -> Version {
    Version {
        total,
        order: "order-1",
        keep_total: true,
        loc: 12.0,
    }
}

/// Writes snapshot `id` with the class `App.Order`, its method `App.Order.Total` and `App.Billing`.
/// Ids are derived from the snapshot id so that several snapshots can coexist.
fn scan(store: &SqliteStore, id: u64, version: &Version) {
    let (file_id, first) = (id + 1, id * 10);
    let mut scan = data(header(id, ScanMode::Full));
    scan.files = vec![file(id, file_id, "src/order.cs", &format!("h{id}"))];
    scan.symbols = vec![
        symbol(first, file_id, SymbolKind::Class, "App.Order", None),
        symbol(first + 2, file_id, SymbolKind::Class, "App.Billing", None),
    ];
    scan.hashes = vec![
        hash_of(first, version.order),
        hash_of(first + 2, "billing-1"),
    ];
    if version.keep_total {
        scan.symbols.push(symbol(
            first + 1,
            file_id,
            SymbolKind::Method,
            "App.Order.Total",
            Some(first),
        ));
        scan.hashes.push(hash_of(first + 1, version.total));
        scan.metrics = vec![
            metric(first + 1, MetricKind::Loc, version.loc),
            metric(first + 1, MetricKind::CyclomaticComplexity, 3.0),
        ];
    }
    store.persist(scan).unwrap();
}

fn hash_of(symbol: u64, text: &str) -> SymbolHash {
    SymbolHash {
        symbol: SymbolId::new(symbol),
        hash: ContentHash::new(text).unwrap(),
    }
}

fn seeded(version: &Version) -> Arc<SqliteStore> {
    let store = store(5);
    store.save_project(&project(1, "/work/app", &[])).unwrap();
    scan(&store, 10, version);
    Arc::new(store)
}

fn target(kind: SymbolKind, name: &str) -> AnnotationTarget {
    AnnotationTarget {
        kind,
        name: QualifiedName::new(name).unwrap(),
        parameters: Vec::new(),
    }
}

fn mark(store: &SqliteStore, kind: AnnotationKind, target: AnnotationTarget) -> Annotation {
    let limits = codeskimr_domain::AnnotationLimits {
        max_text_chars: 100,
        max_title_chars: 20,
    };
    let mut content = AnnotationContent {
        kind,
        title: None,
        text: None,
        status: None,
        due: None,
        assignee: None,
    };
    match kind {
        AnnotationKind::Task => {
            content.title = Some(limits.title("fix").unwrap());
            content.status = Some(codeskimr_domain::TaskStatus::Open);
        }
        AnnotationKind::Note => content.text = Some(limits.text("look at it").unwrap()),
        AnnotationKind::Critical => {}
    }
    store
        .create_annotation(&AnnotationCreation {
            annotation: NewAnnotation {
                project: PROJECT,
                target,
                author: Author::new("anton").unwrap(),
                content,
            },
            now: Timestamp(1),
        })
        .unwrap()
}

fn open_alerts(store: &SqliteStore) -> Vec<Alert> {
    store
        .list_alerts(&AlertFilter {
            project: PROJECT,
            state: Some(AlertState::Open),
        })
        .unwrap()
}

fn latest(id: u64) -> SnapshotId {
    SnapshotId::new(id)
}

fn run(store: &Arc<SqliteStore>, snapshot: u64, trigger: AlertTrigger) {
    let header = header(snapshot, ScanMode::Full);
    step(store).run(&header, trigger).unwrap();
}

#[test]
fn the_hash_is_stored_with_the_symbol_and_found_with_its_signature_lines_and_complexity() {
    let store = seeded(&version("total-1"));
    let names = [QualifiedName::new("App.Order.Total").unwrap()];
    let found = store.states(latest(10), &names).unwrap();
    assert_eq!(found.len(), 1);
    assert_eq!(
        found[0].content_hash,
        Some(ContentHash::new("total-1").unwrap())
    );
    assert_eq!(found[0].signature.as_deref(), Some("Total()"));
    assert_eq!((found[0].loc.0, found[0].cc), (12, 3));
    // A symbol without a hash and the empty list are no problem.
    assert_eq!(store.states(latest(10), &[]).unwrap().len(), 0);
}

#[test]
fn an_incremental_snapshot_keeps_the_hashes_of_the_files_it_did_not_analyse() {
    let store = seeded(&version("total-1"));
    let mut next = data(header(20, ScanMode::Incremental));
    next.files = vec![file(20, 21, "src/other.cs", "h-other")];
    next.symbols = vec![symbol(200, 21, SymbolKind::Class, "App.Other", None)];
    next.hashes = vec![hash_of(200, "other-1")];
    store.persist(next).unwrap();
    let names = [
        QualifiedName::new("App.Order.Total").unwrap(),
        QualifiedName::new("App.Other").unwrap(),
    ];
    let found = store.states(latest(20), &names).unwrap();
    let hashes: Vec<&str> = found
        .iter()
        .filter_map(|state| state.content_hash.as_ref().map(ContentHash::as_str))
        .collect();
    assert_eq!(hashes, ["total-1", "other-1"]);
}

/// Marks the method `Total` critical and takes its first baseline with a scan.
fn watch_total(store: &Arc<SqliteStore>) -> Annotation {
    let critical = mark(
        store,
        AnnotationKind::Critical,
        target(SymbolKind::Method, "App.Order.Total"),
    );
    run(store, 10, AlertTrigger::Scan);
    critical
}

#[test]
fn a_critical_symbol_gets_a_baseline_and_a_change_of_its_tokens_an_alert() {
    let store = seeded(&version("total-1"));
    let critical = watch_total(&store);
    let baseline = store.baseline(critical.id).unwrap().unwrap();
    assert_eq!(baseline.content_hash.as_str(), "total-1");
    assert_eq!(baseline.signature_hash.as_str(), "d[Total()]");
    assert_eq!((baseline.loc.0, baseline.cc), (12, 3));
    assert_eq!(open_alerts(&store).len(), 0);

    scan(
        &store,
        20,
        &Version {
            loc: 15.0,
            ..version("total-2")
        },
    );
    run(&store, 20, AlertTrigger::Watcher);
    let alerts = open_alerts(&store);
    assert_eq!(alerts.len(), 1);
    assert_eq!(alerts[0].sighting.kind, AlertKind::Changed);
    assert_eq!(
        alerts[0].sighting.changes,
        [AlertChange::Content, AlertChange::Loc]
    );
    assert_eq!(alerts[0].sighting.trigger, AlertTrigger::Watcher);
    assert_eq!(alerts[0].sighting.snapshot, latest(20));

    // The same state again, and another save of the same symbol.
    scan(&store, 30, &version("total-2"));
    run(&store, 30, AlertTrigger::Watcher);
    assert_eq!(open_alerts(&store)[0].counter, 1);
    scan(&store, 40, &version("total-3"));
    run(&store, 40, AlertTrigger::Watcher);
    let alerts = open_alerts(&store);
    assert_eq!((alerts.len(), alerts[0].counter), (1, 2));
}

#[test]
fn a_change_in_the_layout_only_gives_no_alert() {
    let store = seeded(&version("total-1"));
    watch_total(&store);
    scan(
        &store,
        20,
        &Version {
            loc: 30.0,
            ..version("total-1")
        },
    );
    run(&store, 20, AlertTrigger::Scan);
    assert_eq!(open_alerts(&store).len(), 0);
}

#[test]
fn dismissing_an_alert_and_changing_the_symbol_again_gives_a_new_alert() {
    let store = seeded(&version("total-1"));
    let critical = watch_total(&store);
    scan(&store, 20, &version("total-2"));
    run(&store, 20, AlertTrigger::Scan);
    let first = open_alerts(&store).remove(0);
    let now = SymbolBaseline {
        content_hash: ContentHash::new("total-2").unwrap(),
        signature_hash: ContentHash::new("d[Total()]").unwrap(),
        loc: codeskimr_domain::Loc(12),
        cc: 3,
    };
    store
        .dismiss_alert(&AlertDismissal {
            alert: first.id,
            baseline: Some(now),
            now: Timestamp(6_000),
        })
        .unwrap();
    scan(&store, 30, &version("total-2"));
    run(&store, 30, AlertTrigger::Scan);
    assert_eq!(open_alerts(&store).len(), 0);
    scan(&store, 40, &version("total-4"));
    run(&store, 40, AlertTrigger::Scan);
    let open = open_alerts(&store);
    assert_eq!(open.len(), 1);
    assert_eq!((open[0].counter, open[0].annotation), (1, critical.id));
    assert_ne!(open[0].id, first.id);
}

#[test]
fn a_gone_symbol_is_removed_with_the_names_that_still_point_at_it() {
    let store = seeded(&version("total-1"));
    watch_total(&store);
    // Snapshot 20 no longer has the method, but two names in `App.Billing` still use `Total`.
    let mut next = data(header(20, ScanMode::Full));
    next.files = vec![file(20, 21, "src/order.cs", "h20")];
    next.symbols = vec![
        symbol(200, 21, SymbolKind::Class, "App.Order", None),
        symbol(202, 21, SymbolKind::Class, "App.Billing", None),
    ];
    next.pending = ["Total", "App.Order.Total", "Other"]
        .into_iter()
        .map(|target| PendingReference {
            from: SymbolId::new(202),
            file: FileId::new(21),
            span: span(3),
            kind: ReferenceKind::Call,
            target: QualifiedName::new(target).unwrap(),
            receiver: None,
        })
        .collect();
    store.persist(next).unwrap();
    run(&store, 20, AlertTrigger::Scan);
    let alert = open_alerts(&store).remove(0);
    assert_eq!(alert.sighting.kind, AlertKind::Removed);
    assert_eq!(alert.sighting.references, Some(2));
}

#[test]
fn a_symbol_with_the_same_name_elsewhere_makes_the_annotation_orphaned() {
    let store = seeded(&version("total-1"));
    watch_total(&store);
    let mut next = data(header(20, ScanMode::Full));
    next.files = vec![file(20, 21, "src/billing.cs", "h20")];
    next.symbols = vec![
        symbol(200, 21, SymbolKind::Class, "App.Billing", None),
        symbol(201, 21, SymbolKind::Method, "App.Billing.Total", Some(200)),
        symbol(202, 21, SymbolKind::Method, "Other.Total", None),
    ];
    store.persist(next).unwrap();
    run(&store, 20, AlertTrigger::Scan);
    let alert = open_alerts(&store).remove(0);
    assert_eq!(alert.sighting.kind, AlertKind::Orphaned);
    assert_eq!(
        alert.sighting.closest,
        Some(QualifiedName::new("App.Billing.Total").unwrap())
    );
}

/// `App` holds `App.Order` and the nested namespace `App.Sub` with `App.Sub.Deep`.
fn namespaced(store: &SqliteStore, id: u64, deep: &str) {
    let (file_id, first) = (id + 1, id * 10);
    let mut scan = data(header(id, ScanMode::Full));
    scan.files = vec![file(id, file_id, "src/app.cs", &format!("h{id}"))];
    scan.symbols = vec![
        symbol(first, file_id, SymbolKind::Namespace, "App", None),
        symbol(
            first + 1,
            file_id,
            SymbolKind::Class,
            "App.Order",
            Some(first),
        ),
        symbol(
            first + 2,
            file_id,
            SymbolKind::Namespace,
            "App.Sub",
            Some(first),
        ),
        symbol(
            first + 3,
            file_id,
            SymbolKind::Class,
            "App.Sub.Deep",
            Some(first + 2),
        ),
    ];
    scan.hashes = vec![hash_of(first + 1, "order-1"), hash_of(first + 3, deep)];
    store.persist(scan).unwrap();
}

#[test]
fn a_namespace_hashes_the_types_below_it_also_in_nested_namespaces() {
    let store = store(5);
    store.save_project(&project(1, "/work/app", &[])).unwrap();
    namespaced(&store, 10, "deep-1");
    let store = Arc::new(store);
    let name = QualifiedName::new("App").unwrap();
    let mut hashes: Vec<String> = store
        .child_hashes(latest(10), &name)
        .unwrap()
        .iter()
        .map(ToString::to_string)
        .collect();
    hashes.sort();
    assert_eq!(hashes, ["deep-1", "order-1"]);

    mark(
        &store,
        AnnotationKind::Critical,
        target(SymbolKind::Namespace, "App"),
    );
    run(&store, 10, AlertTrigger::Scan);
    namespaced(&store, 20, "deep-2");
    run(&store, 20, AlertTrigger::Scan);
    let alert = open_alerts(&store).remove(0);
    assert_eq!(alert.sighting.changes, [AlertChange::Content]);
}

fn flags(store: &SqliteStore) -> Vec<bool> {
    store
        .list_annotations(&AnnotationFilter {
            project: PROJECT,
            target: None,
            kind: None,
        })
        .unwrap()
        .iter()
        .map(|annotation| annotation.without_target)
        .collect()
}

#[test]
fn a_note_without_its_symbol_is_marked_and_unmarked_when_the_symbol_returns() {
    let store = seeded(&version("total-1"));
    let note = mark(
        &store,
        AnnotationKind::Note,
        target(SymbolKind::Method, "App.Order.Total"),
    );
    let task = mark(
        &store,
        AnnotationKind::Task,
        target(SymbolKind::Class, "App.Billing"),
    );
    assert_eq!(flags(&store), [false, false]);
    let store = Arc::new(store);

    scan(
        &store,
        20,
        &Version {
            keep_total: false,
            ..version("x")
        },
    );
    assert_eq!(store.refresh_targets(PROJECT, latest(20)).unwrap(), 1);
    assert_eq!(flags(&store), [true, false]);
    // No alert for a note, whatever the step does.
    run(&store, 20, AlertTrigger::Scan);
    assert_eq!(open_alerts(&store).len(), 0);

    scan(&store, 30, &version("total-1"));
    assert_eq!(store.refresh_targets(PROJECT, latest(30)).unwrap(), 1);
    assert_eq!(flags(&store), [false, false]);
    assert_eq!((note.id.get(), task.id.get()), (1, 2));
}

#[test]
fn without_annotations_the_refresh_touches_nothing() {
    let store = seeded(&version("total-1"));
    assert_eq!(store.refresh_targets(PROJECT, latest(10)).unwrap(), 0);
    let report = step(&store)
        .run(&header(10, ScanMode::Full), AlertTrigger::Scan)
        .unwrap();
    assert_eq!((report.checked, report.raised, report.baselines), (0, 0, 0));
}

#[test]
fn the_symbols_of_a_simple_name_are_found_with_their_kinds() {
    let store = seeded(&version("total-1"));
    let names = [QualifiedName::new("App.Billing").unwrap()];
    let found = store.states(latest(10), &names).unwrap();
    assert_eq!(
        found[0].content_hash.as_ref().map(ContentHash::as_str),
        Some("billing-1")
    );
    let simple = SymbolName::new("Total").unwrap();
    assert_eq!(
        store.same_simple_name(latest(10), &simple).unwrap().len(),
        1
    );
}
