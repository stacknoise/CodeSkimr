//! `list_annotations` and `get_alerts` against fake ports (ADR 0068): read-only, paged, compact,
//! and open to a caller with `read` only.

#[cfg(test)]
mod common;

use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_domain::{
    Alert, AlertChange, AlertId, AlertKind, AlertSighting, AlertState, AlertTrigger, Annotation,
    AnnotationContent, AnnotationId, AnnotationKind, AnnotationLimits, AnnotationTarget, Author,
    Permission, PermissionSet, Principal, QualifiedName, SnapshotId, SymbolKind, TaskStatus,
    Timestamp,
};
use codeskimr_mcp::{AlertsArgs, AnnotationsArgs};
use common::{FakeAnnotations, NOW, PROJECT, SCANNED_AT, tools_annotated};

#[cfg(test)]
const LIMITS: AnnotationLimits = AnnotationLimits {
    max_text_chars: 100,
    max_title_chars: 20,
};

#[cfg(test)]
fn owner() -> Principal {
    Principal::local_owner()
}

#[cfg(test)]
fn target(kind: SymbolKind, name: &str) -> AnnotationTarget {
    AnnotationTarget {
        kind,
        name: QualifiedName::new(name).unwrap(),
        parameters: Vec::new(),
    }
}

#[cfg(test)]
fn content(kind: AnnotationKind, text: &str) -> AnnotationContent {
    let task = kind == AnnotationKind::Task;
    AnnotationContent {
        kind,
        title: task.then(|| LIMITS.title("Do it").unwrap()),
        text: Some(LIMITS.text(text).unwrap()),
        status: task.then_some(TaskStatus::Open),
        due: None,
        assignee: None,
    }
}

#[cfg(test)]
fn annotation(id: u64, on: AnnotationTarget, content: AnnotationContent) -> Annotation {
    Annotation {
        id: AnnotationId::new(id),
        project: PROJECT,
        target: on,
        author: Author::new("local").unwrap(),
        content,
        created: Timestamp(SCANNED_AT),
        updated: Timestamp(SCANNED_AT),
        without_target: false,
    }
}

#[cfg(test)]
fn alert(id: u64, annotation: u64, state: AlertState, kind: AlertKind) -> Alert {
    Alert {
        id: AlertId::new(id),
        annotation: AnnotationId::new(annotation),
        state,
        counter: 2,
        first_seen: Timestamp(SCANNED_AT),
        last_seen: Timestamp(NOW),
        dismissed: None,
        sighting: AlertSighting {
            kind,
            snapshot: SnapshotId::new(10),
            commit: None,
            trigger: AlertTrigger::Scan,
            changes: if kind == AlertKind::Changed {
                vec![AlertChange::Content, AlertChange::Signature]
            } else {
                Vec::new()
            },
            references: (kind == AlertKind::Removed).then_some(3),
            closest: None,
            observed: None,
        },
    }
}

#[cfg(test)]
fn fixture() -> FakeAnnotations {
    let order = || target(SymbolKind::Class, "App.Domain.Order");
    FakeAnnotations {
        annotations: vec![
            annotation(
                1,
                order(),
                content(AnnotationKind::Critical, "Billing relies on it"),
            ),
            annotation(
                2,
                order(),
                content(AnnotationKind::Note, "Keep the rounding"),
            ),
            annotation(
                3,
                target(SymbolKind::Method, "App.Order.Total"),
                content(AnnotationKind::Task, "Split it"),
            ),
            annotation(
                4,
                target(SymbolKind::Class, "App.OrderService"),
                content(AnnotationKind::Critical, "Entry point"),
            ),
        ],
        alerts: vec![
            alert(1, 1, AlertState::Open, AlertKind::Changed),
            alert(2, 4, AlertState::Open, AlertKind::Removed),
            alert(3, 1, AlertState::Dismissed, AlertKind::Changed),
        ],
    }
}

#[cfg(test)]
fn annotations_args() -> AnnotationsArgs {
    AnnotationsArgs {
        symbol: None,
        kind: None,
        status: None,
        project: None,
        limit: None,
        offset: None,
    }
}

#[cfg(test)]
fn alerts_args() -> AlertsArgs {
    AlertsArgs {
        state: None,
        kind: None,
        project: None,
        limit: None,
        offset: None,
    }
}

#[test]
fn annotations_come_in_pages_that_name_the_snapshot_and_its_age() {
    let tools = tools_annotated(fixture());
    let first = tools
        .list_annotations(&owner(), &annotations_args())
        .unwrap();
    assert_eq!(first["snapshot"]["id"], 10);
    assert_eq!(first["snapshot"]["age_seconds"], NOW - SCANNED_AT);
    assert_eq!(first["total"], 4);
    assert_eq!(
        first["annotations"].as_array().unwrap().len(),
        2,
        "default limit"
    );
    assert_eq!(first["next_offset"], 2);
    let mut next = annotations_args();
    next.offset = Some(2);
    let second = tools.list_annotations(&owner(), &next).unwrap();
    assert_eq!(second["annotations"].as_array().unwrap().len(), 2);
    assert!(
        second.get("next_offset").is_none(),
        "the last page has none"
    );
}

#[test]
fn the_limit_is_held_to_the_configured_maximum() {
    let mut args = annotations_args();
    args.limit = Some(1_000);
    let found = tools_annotated(fixture())
        .list_annotations(&owner(), &args)
        .unwrap();
    assert_eq!(
        found["annotations"].as_array().unwrap().len(),
        3,
        "max_limit"
    );
}

#[test]
fn a_symbol_or_a_kind_narrows_the_list_and_empty_fields_are_left_out() {
    let tools = tools_annotated(fixture());
    let mut args = annotations_args();
    args.symbol = Some("App.Domain.Order".to_owned());
    let of_symbol = tools.list_annotations(&owner(), &args).unwrap();
    assert_eq!(of_symbol["total"], 2);
    let note = &of_symbol["annotations"][1];
    assert_eq!(note["kind"], "note");
    assert_eq!(note["text"], "Keep the rounding");
    for absent in [
        "title",
        "status",
        "due",
        "assignee",
        "parameters",
        "without_target",
    ] {
        assert!(note.get(absent).is_none(), "{absent} is empty and left out");
    }
    let mut tasks = annotations_args();
    tasks.kind = Some("task".to_owned());
    tasks.status = Some("open".to_owned());
    let found = tools.list_annotations(&owner(), &tasks).unwrap();
    assert_eq!(found["total"], 1);
    assert_eq!(found["annotations"][0]["status"], "open");
    assert_eq!(found["annotations"][0]["title"], "Do it");
}

#[test]
fn an_unknown_label_or_symbol_is_an_invalid_argument() {
    let tools = tools_annotated(fixture());
    let mut bad_kind = annotations_args();
    bad_kind.kind = Some("memo".to_owned());
    let error = tools.list_annotations(&owner(), &bad_kind).unwrap_err();
    assert_eq!(error.code(), ErrorCode::ToolArgumentsInvalid);
    let mut missing = annotations_args();
    missing.symbol = Some("No.Such.Symbol".to_owned());
    let error = tools.list_annotations(&owner(), &missing).unwrap_err();
    assert_eq!(error.code(), ErrorCode::SymbolNotFound);
}

#[test]
fn open_alerts_are_the_default_and_say_what_changed() {
    let tools = tools_annotated(fixture());
    let found = tools.get_alerts(&owner(), &alerts_args()).unwrap();
    assert_eq!(found["open"], 2);
    assert_eq!(found["total"], 2);
    let changed = &found["alerts"][0];
    assert_eq!(changed["kind"], "changed");
    assert_eq!(changed["symbol"], "App.Domain.Order");
    assert_eq!(
        changed["changes"],
        serde_json::json!(["content", "signature"])
    );
    assert_eq!(changed["count"], 2);
    assert_eq!(changed["reason"], "Billing relies on it");
    assert!(changed.get("references").is_none());
    let removed = &found["alerts"][1];
    assert_eq!(removed["references"], 3);
    assert!(removed.get("changes").is_none());
    assert_eq!(found["snapshot"]["age_seconds"], NOW - SCANNED_AT);
}

#[test]
fn alerts_can_be_filtered_by_state_and_kind() {
    let tools = tools_annotated(fixture());
    let mut dismissed = alerts_args();
    dismissed.state = Some("dismissed".to_owned());
    let found = tools.get_alerts(&owner(), &dismissed).unwrap();
    assert_eq!(found["total"], 1);
    assert_eq!(found["open"], 2, "the open count ignores the filter");
    let mut removed = alerts_args();
    removed.kind = Some("removed".to_owned());
    let found = tools.get_alerts(&owner(), &removed).unwrap();
    assert_eq!(found["total"], 1);
    assert_eq!(found["alerts"][0]["symbol"], "App.OrderService");
    let mut bad = alerts_args();
    bad.state = Some("gone".to_owned());
    assert_eq!(
        tools.get_alerts(&owner(), &bad).unwrap_err().code(),
        ErrorCode::ToolArgumentsInvalid
    );
}

#[test]
fn reading_needs_read_and_nothing_else_and_a_denial_gives_nothing_away() {
    let tools = tools_annotated(fixture());
    let reader = Principal::limited(PermissionSet::none().with(Permission::Read));
    assert!(tools.list_annotations(&reader, &annotations_args()).is_ok());
    assert!(tools.get_alerts(&reader, &alerts_args()).is_ok());
    let writer = Principal::limited(
        PermissionSet::none()
            .with(Permission::Annotate)
            .with(Permission::ReadSource),
    );
    let denied = [
        tools.list_annotations(&writer, &annotations_args()),
        tools.get_alerts(&writer, &alerts_args()),
    ];
    for result in denied {
        let error = result.unwrap_err();
        assert_eq!(error.code(), ErrorCode::PermissionDenied);
        assert!(error.report().params.is_empty());
    }
}
