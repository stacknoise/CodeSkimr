//! The alert step (ADR 0068) over fake ports: baselines, changes, removals, the counter, events and
//! the failure of a store.

use std::sync::Arc;
use std::sync::atomic::Ordering;

#[cfg(test)]
#[path = "common/alerts.rs"]
mod fakes;

use codeskimr_application::alerts::{AlertReport, compare, target};
use codeskimr_application::events::{AlertsChanged, Event, EventBus};
use codeskimr_application::ports::{AlertDismissal, AnnotationStore, SameName};
use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_domain::{
    AlertChange, AlertKind, AlertTrigger, AnnotationTarget, CommitHash, Loc, QualifiedName,
    ScanMode, Snapshot, SnapshotId, SnapshotStats, SymbolBaseline, SymbolKind, Timestamp,
};
use fakes::{MemoryAnnotations, MemoryStates, PROJECT, hash, step, symbol};

const WORK: &str = "Demo.Box.Work";

struct Fixture {
    annotations: Arc<MemoryAnnotations>,
    states: Arc<MemoryStates>,
    bus: EventBus,
}

fn fixture() -> Fixture {
    Fixture {
        annotations: Arc::new(MemoryAnnotations::default()),
        states: Arc::new(MemoryStates::default()),
        bus: EventBus::new(),
    }
}

#[cfg(test)]
/// A fixture with a critical annotation on `WORK` and its baseline taken from the first scan.
#[cfg(test)]
fn watched() -> Fixture {
    let world = fixture();
    world.annotations.critical(SymbolKind::Method, WORK, &[]);
    world.with_work("one");
    world.run(1, AlertTrigger::Scan);
    world
}

#[cfg(test)]
fn snapshot(id: u64) -> Snapshot {
    Snapshot {
        id: SnapshotId::new(id),
        project: PROJECT,
        commit: Some(CommitHash::new("abc123").unwrap()),
        created: Timestamp(1),
        mode: ScanMode::Incremental,
        stats: SnapshotStats::default(),
    }
}

impl Fixture {
    #[cfg(test)]
    fn run(
        &self,
        snapshot: u64,
        trigger: AlertTrigger,
    ) -> codeskimr_application::alerts::AlertReport {
        step(&self.annotations, &self.states, &self.bus)
            .run(&self::snapshot(snapshot), trigger)
            .unwrap()
    }

    fn with_work(&self, content: &str) {
        self.states
            .set(vec![symbol(SymbolKind::Method, WORK, content)]);
    }
}

/// The sighting says `orphaned` and names `Demo.Moved.Work` as the closest symbol.
#[cfg(test)]
fn assert_orphaned_to_moved(sighting: &codeskimr_domain::AlertSighting) {
    assert_eq!(sighting.kind, AlertKind::Orphaned);
    assert_eq!(
        sighting.closest,
        Some(QualifiedName::new("Demo.Moved.Work").unwrap())
    );
}

#[test]
fn the_first_run_takes_a_baseline_and_raises_nothing() {
    let world = fixture();
    let annotation = world.annotations.critical(SymbolKind::Method, WORK, &[]);
    world.with_work("one");
    let report = world.run(1, AlertTrigger::Scan);
    assert_eq!((report.checked, report.baselines, report.raised), (1, 1, 0));
    let baseline = world.annotations.baseline(annotation.id).unwrap().unwrap();
    assert_eq!(baseline.content_hash, hash("one"));
    assert_eq!(world.annotations.open_alerts().len(), 0);
}

#[test]
fn a_changed_symbol_raises_an_alert_that_says_what_changed_and_where_it_was_seen() {
    let world = watched();
    world.states.set(vec![{
        let mut changed = symbol(SymbolKind::Method, WORK, "two");
        changed.loc = Loc(14);
        changed.cc = 5;
        changed
    }]);
    let report = world.run(2, AlertTrigger::Watcher);
    assert_eq!(report.raised, 1);
    let alerts = world.annotations.open_alerts();
    assert_eq!(alerts.len(), 1);
    let sighting = &alerts[0].sighting;
    assert_eq!(sighting.kind, AlertKind::Changed);
    assert_eq!(
        sighting.changes,
        [AlertChange::Content, AlertChange::Cc, AlertChange::Loc]
    );
    assert_eq!(sighting.snapshot, SnapshotId::new(2));
    assert_eq!(sighting.trigger, AlertTrigger::Watcher);
    assert_eq!(sighting.commit, Some(CommitHash::new("abc123").unwrap()));
}

#[test]
fn a_cosmetic_edit_gives_no_alert() {
    let world = watched();
    // The same tokens in another layout: the hash is the same, the lines of code differ.
    world.states.set(vec![{
        let mut laid_out = symbol(SymbolKind::Method, WORK, "one");
        laid_out.loc = Loc(25);
        laid_out
    }]);
    assert_eq!(world.run(2, AlertTrigger::Scan).raised, 0);
    assert_eq!(world.annotations.open_alerts().len(), 0);
}

#[test]
fn a_changed_signature_alone_raises_an_alert() {
    let world = watched();
    world.states.set(vec![{
        let mut other = symbol(SymbolKind::Method, WORK, "one");
        other.signature = Some("Work(int x)".to_owned());
        other
    }]);
    world.run(2, AlertTrigger::Scan);
    let alerts = world.annotations.open_alerts();
    assert_eq!(alerts[0].sighting.changes, [AlertChange::Signature]);
}

#[test]
fn several_saves_give_one_alert_with_a_counter_and_a_scan_that_changes_nothing_adds_nothing() {
    let world = watched();
    world.with_work("two");
    world.run(2, AlertTrigger::Watcher);
    world.with_work("three");
    world.run(3, AlertTrigger::Watcher);
    // A save elsewhere: the symbol looks as it did at the last sighting.
    world.run(4, AlertTrigger::Watcher);
    let alerts = world.annotations.open_alerts();
    assert_eq!(alerts.len(), 1);
    assert_eq!(alerts[0].counter, 2);
    assert_eq!(alerts[0].sighting.snapshot, SnapshotId::new(3));
}

#[test]
fn dismissing_takes_the_current_state_as_the_baseline_and_a_later_change_is_a_new_alert() {
    let world = fixture();
    let annotation = world.annotations.critical(SymbolKind::Method, WORK, &[]);
    world.with_work("one");
    world.run(1, AlertTrigger::Scan);
    world.with_work("two");
    world.run(2, AlertTrigger::Scan);
    let alert = world.annotations.open_alerts().remove(0);
    let now = SymbolBaseline {
        content_hash: hash("two"),
        signature_hash: hash("digest(Demo.Box.Work())"),
        loc: Loc(10),
        cc: 2,
    };
    world
        .annotations
        .dismiss_alert(&AlertDismissal {
            alert: alert.id,
            baseline: Some(now),
            now: Timestamp(5),
        })
        .unwrap();
    assert_eq!(world.run(3, AlertTrigger::Scan).raised, 0);
    world.with_work("three");
    assert_eq!(world.run(4, AlertTrigger::Scan).raised, 1);
    let open = world.annotations.open_alerts();
    assert_eq!(open.len(), 1);
    assert_eq!(open[0].counter, 1);
    assert_ne!(open[0].id, alert.id);
    assert_eq!(world.annotations.held().alerts.len(), 2);
    assert_eq!(annotation.id, open[0].annotation);
}

#[test]
fn a_symbol_that_is_gone_without_a_namesake_is_removed_with_the_names_that_still_point_at_it() {
    let world = watched();
    world.states.set(Vec::new());
    world.states.references.store(3, Ordering::SeqCst);
    world.run(2, AlertTrigger::Scan);
    let sighting = world.annotations.open_alerts().remove(0).sighting;
    assert_eq!(sighting.kind, AlertKind::Removed);
    assert_eq!(sighting.references, Some(3));
    assert_eq!(sighting.closest, None);
    // The next scan does not count the same loss again.
    world.run(3, AlertTrigger::Scan);
    assert_eq!(world.annotations.open_alerts()[0].counter, 1);
}

#[test]
fn a_symbol_that_is_gone_with_a_namesake_is_orphaned_and_names_the_closest_one() {
    let world = watched();
    world.states.set(Vec::new());
    *world.states.same_name.lock().unwrap() = vec![
        SameName {
            kind: SymbolKind::Method,
            name: QualifiedName::new("Other.Thing.Work").unwrap(),
        },
        SameName {
            kind: SymbolKind::Method,
            name: QualifiedName::new("Demo.Moved.Work").unwrap(),
        },
    ];
    world.run(2, AlertTrigger::Scan);
    let sighting = world.annotations.open_alerts().remove(0).sighting;
    assert_orphaned_to_moved(&sighting);
    assert_eq!(sighting.references, None);
}

#[test]
fn an_open_alert_follows_the_state_of_the_symbol_without_counting_the_way_back() {
    let world = watched();
    world.with_work("two");
    world.run(2, AlertTrigger::Scan);
    let counter = || world.annotations.open_alerts()[0].counter;
    assert_eq!(counter(), 1);

    // Gone without a namesake: the alert turns to `removed` and counts the new event.
    world.states.set(Vec::new());
    world.states.references.store(2, Ordering::SeqCst);
    world.run(3, AlertTrigger::Scan);
    let sighting = world.annotations.open_alerts().remove(0).sighting;
    assert_eq!(
        (sighting.kind, sighting.references),
        (AlertKind::Removed, Some(2))
    );
    assert_eq!(counter(), 2);

    // A namesake appears: `removed` becomes `orphaned` and names it, without another count.
    *world.states.same_name.lock().unwrap() = vec![SameName {
        kind: SymbolKind::Method,
        name: QualifiedName::new("Demo.Moved.Work").unwrap(),
    }];
    world.run(4, AlertTrigger::Scan);
    let sighting = world.annotations.open_alerts().remove(0).sighting;
    assert_orphaned_to_moved(&sighting);
    assert_eq!(counter(), 2);

    // The same scan again says nothing new.
    assert_eq!(world.run(5, AlertTrigger::Scan).raised, 0);

    // Back in the state it was seen last: `changed` again and no additional count.
    world.with_work("two");
    world.run(6, AlertTrigger::Scan);
    let alert = world.annotations.open_alerts().remove(0);
    assert_eq!(alert.sighting.kind, AlertKind::Changed);
    assert_eq!(alert.sighting.closest, None);
    assert_eq!(alert.counter, 2);

    // Back but different from what was seen: a new change counts.
    world.states.set(Vec::new());
    world.run(7, AlertTrigger::Scan);
    world.with_work("three");
    world.run(8, AlertTrigger::Scan);
    assert_eq!(world.annotations.open_alerts()[0].counter, 4);
}

#[test]
fn a_namespace_changes_when_a_type_below_it_changes_whatever_the_order() {
    let world = fixture();
    world
        .annotations
        .critical(SymbolKind::Namespace, "Demo", &[]);
    world
        .states
        .set(vec![symbol(SymbolKind::Namespace, "Demo", "none")]);
    *world.states.children.lock().unwrap() = vec![hash("b"), hash("a")];
    world.run(1, AlertTrigger::Scan);
    *world.states.children.lock().unwrap() = vec![hash("a"), hash("b")];
    assert_eq!(world.run(2, AlertTrigger::Scan).raised, 0);
    *world.states.children.lock().unwrap() = vec![hash("a"), hash("c")];
    assert_eq!(world.run(3, AlertTrigger::Scan).raised, 1);
    *world.states.children.lock().unwrap() = vec![hash("a"), hash("c"), hash("d")];
    world.run(4, AlertTrigger::Scan);
    assert_eq!(world.annotations.open_alerts()[0].counter, 2);
}

#[test]
fn overloads_are_told_apart_by_the_parameter_types_of_the_target() {
    let world = fixture();
    let mut int = symbol(SymbolKind::Method, WORK, "int-version");
    int.signature = Some("Work(int x)".to_owned());
    let mut text = symbol(SymbolKind::Method, WORK, "text-version");
    text.signature = Some("Work(string s)".to_owned());
    world.states.set(vec![int, text]);
    let annotation = world
        .annotations
        .critical(SymbolKind::Method, WORK, &["string"]);
    world.run(1, AlertTrigger::Scan);
    let baseline = world.annotations.baseline(annotation.id).unwrap().unwrap();
    assert_eq!(baseline.content_hash, hash("text-version"));
}

#[test]
fn an_overload_with_more_parameters_is_not_taken_for_the_shorter_one() {
    let world = fixture();
    let mut long = symbol(SymbolKind::Method, WORK, "long-version");
    long.signature = Some("Work(int x, string s)".to_owned());
    let mut short = symbol(SymbolKind::Method, WORK, "short-version");
    short.signature = Some("Work(int x)".to_owned());
    world.states.set(vec![long, short]);
    let annotation = world
        .annotations
        .critical(SymbolKind::Method, WORK, &["int x"]);
    world.run(1, AlertTrigger::Scan);
    let baseline = world.annotations.baseline(annotation.id).unwrap().unwrap();
    assert_eq!(baseline.content_hash, hash("short-version"));
}

#[test]
fn an_overload_that_no_longer_matches_the_parameter_types_is_gone() {
    let world = fixture();
    let mut int = symbol(SymbolKind::Method, WORK, "int-version");
    int.signature = Some("Work(int x)".to_owned());
    let mut long = symbol(SymbolKind::Method, WORK, "long-version");
    long.signature = Some("Work(long x)".to_owned());
    world.states.set(vec![int, long]);
    world
        .annotations
        .critical(SymbolKind::Method, WORK, &["string"]);
    world.run(1, AlertTrigger::Scan);
    assert_eq!(
        world.annotations.open_alerts()[0].sighting.kind,
        AlertKind::Removed
    );
}

#[test]
fn without_critical_annotations_the_step_reads_no_snapshot() {
    let world = fixture();
    world.with_work("one");
    let report = world.run(1, AlertTrigger::Scan);
    assert_eq!(report, AlertReport::default());
    assert_eq!(world.states.reads(), 0);
    assert_eq!(world.annotations.calls_named("list_annotations"), 1);
    assert_eq!(world.annotations.calls_named("baseline"), 0);
    assert_eq!(world.annotations.calls_named("list_alerts"), 0);
}

#[test]
fn an_event_tells_the_number_of_open_alerts_and_only_when_one_was_raised() {
    let world = fixture();
    let events = world.bus.subscribe();
    world.annotations.critical(SymbolKind::Method, WORK, &[]);
    world.with_work("one");
    world.run(1, AlertTrigger::Scan);
    assert_eq!(events.try_iter().count(), 0);
    world.with_work("two");
    world.run(2, AlertTrigger::Scan);
    assert_eq!(
        events.try_iter().collect::<Vec<_>>(),
        [Event::AlertsChanged(AlertsChanged {
            project: PROJECT,
            open: 1
        })]
    );
}

#[test]
fn a_failure_of_the_store_is_returned() {
    let world = fixture();
    world.annotations.critical(SymbolKind::Method, WORK, &[]);
    *world.annotations.fail.lock().unwrap() = Some(ErrorCode::DiskFull);
    let error = step(&world.annotations, &world.states, &world.bus)
        .run(&snapshot(1), AlertTrigger::Scan)
        .unwrap_err();
    assert_eq!(error.code(), ErrorCode::DiskFull);
}

#[test]
fn one_annotation_that_fails_does_not_keep_the_others_from_being_checked() {
    let world = fixture();
    let first = world.annotations.critical(SymbolKind::Method, WORK, &[]);
    world
        .annotations
        .critical(SymbolKind::Method, "Demo.Box.Other", &[]);
    *world.annotations.fail_baseline_of.lock().unwrap() = Some(first.id);
    world.states.set(vec![
        symbol(SymbolKind::Method, WORK, "one"),
        symbol(SymbolKind::Method, "Demo.Box.Other", "other"),
    ]);
    let result =
        step(&world.annotations, &world.states, &world.bus).run(&snapshot(1), AlertTrigger::Scan);
    assert!(result.is_err());
    assert_eq!(world.annotations.held().baselines.len(), 1);
}

#[test]
fn the_closest_namesake_prefers_the_kind_then_the_longest_shared_start_then_the_name() {
    let target = AnnotationTarget {
        kind: SymbolKind::Method,
        name: QualifiedName::new("A.B.C.Run").unwrap(),
        parameters: Vec::new(),
    };
    let same = |kind, name: &str| SameName {
        kind,
        name: QualifiedName::new(name).unwrap(),
    };
    let found = target::closest(
        &target,
        &[
            same(SymbolKind::Function, "A.B.C.D.Run"),
            same(SymbolKind::Method, "X.Run"),
            same(SymbolKind::Method, "A.B.Z.Run"),
            same(SymbolKind::Method, "A.B.Y.Run"),
        ],
    );
    assert_eq!(found, Some(QualifiedName::new("A.B.Y.Run").unwrap()));
    assert_eq!(target::closest(&target, &[]), None);
}

#[test]
fn the_simple_name_is_the_last_segment_of_a_dotted_or_scoped_name() {
    let simple = |text: &str| {
        target::simple_name(&QualifiedName::new(text).unwrap()).map(|name| name.to_string())
    };
    assert_eq!(simple("Demo.Box.Work").as_deref(), Some("Work"));
    assert_eq!(simple("crate::module::run").as_deref(), Some("run"));
}

#[test]
fn only_a_difference_of_content_or_signature_counts_and_it_names_the_numbers_too() {
    let base = SymbolBaseline {
        content_hash: hash("a"),
        signature_hash: hash("s"),
        loc: Loc(5),
        cc: 1,
    };
    let numbers = SymbolBaseline {
        loc: Loc(9),
        cc: 4,
        ..base.clone()
    };
    assert_eq!(compare::differences(&base, &numbers), []);
    let content = SymbolBaseline {
        content_hash: hash("b"),
        ..numbers
    };
    assert_eq!(
        compare::differences(&base, &content),
        [AlertChange::Content, AlertChange::Cc, AlertChange::Loc]
    );
}
