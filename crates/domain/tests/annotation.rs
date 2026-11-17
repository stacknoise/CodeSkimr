//! Tests of annotations, alerts and their validated texts (ADR 0068).

use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_domain::{
    AlertChange, AlertKind, AlertState, AlertTrigger, AnnotationContent, AnnotationKind,
    AnnotationLimits, AnnotationText, ModelError, TaskStatus, Timestamp,
};

const LIMITS: AnnotationLimits = AnnotationLimits {
    max_text_chars: 10,
    max_title_chars: 4,
};

fn content(kind: AnnotationKind) -> AnnotationContent {
    AnnotationContent {
        kind,
        title: None,
        text: None,
        status: None,
        due: None,
        assignee: None,
    }
}

fn round_trip<T>(all: &[T], label: fn(T) -> &'static str)
where
    T: Copy + std::fmt::Debug + PartialEq + std::str::FromStr<Err = ModelError>,
{
    for variant in all {
        assert_eq!(label(*variant).parse::<T>(), Ok(*variant));
    }
}

#[test]
fn vocabularies_round_trip_and_match_the_adr() {
    assert_eq!(AnnotationKind::ALL.len(), 3);
    assert_eq!(TaskStatus::InProgress.label(), "in_progress");
    assert_eq!(AlertKind::ALL.len(), 3);
    assert_eq!(AlertState::ALL.len(), 2);
    round_trip(AnnotationKind::ALL, AnnotationKind::label);
    round_trip(TaskStatus::ALL, TaskStatus::label);
    round_trip(AlertKind::ALL, AlertKind::label);
    round_trip(AlertState::ALL, AlertState::label);
    round_trip(AlertTrigger::ALL, AlertTrigger::label);
    round_trip(AlertChange::ALL, AlertChange::label);
}

#[test]
fn a_text_may_be_as_long_as_the_limit_and_not_longer() {
    assert!(LIMITS.text("0123456789").is_ok());
    assert_eq!(LIMITS.text("01234567890"), Err(ModelError::TooLong));
    assert_eq!(LIMITS.title("abcde"), Err(ModelError::TooLong));
    assert!(LIMITS.title("abcd").is_ok());
}

#[test]
fn the_limit_counts_characters_not_bytes() {
    assert!(LIMITS.text("äöüäöüäöüä").is_ok());
}

#[test]
fn an_empty_text_is_refused_and_a_stored_text_ignores_the_limit() {
    assert_eq!(LIMITS.text("  \n"), Err(ModelError::EmptyText));
    assert_eq!(LIMITS.title(" "), Err(ModelError::EmptyText));
    assert_eq!(AnnotationText::restore(""), Err(ModelError::EmptyText));
    let stored = AnnotationText::restore("longer than any limit that was lowered later").unwrap();
    assert!(stored.as_str().starts_with("longer"));
}

#[test]
fn user_input_errors_have_their_own_codes() {
    assert_eq!(ModelError::TooLong.code(), ErrorCode::AnnotationTextTooLong);
    assert_eq!(
        ModelError::Inconsistent.code(),
        ErrorCode::AnnotationInvalid
    );
    assert_eq!(ModelError::EmptyText.code(), ErrorCode::AnnotationInvalid);
    assert_eq!(ModelError::Empty.code(), ErrorCode::UnexpectedInternalError);
}

#[test]
fn the_fields_must_fit_the_kind() {
    let text = LIMITS.text("why").unwrap();
    let title = LIMITS.title("do").unwrap();

    assert_eq!(content(AnnotationKind::Critical).check(), Ok(()));
    let mut critical = content(AnnotationKind::Critical);
    critical.text = Some(text.clone());
    assert_eq!(critical.check(), Ok(()));
    critical.due = Some(Timestamp(1));
    assert_eq!(critical.check(), Err(ModelError::Inconsistent));

    assert_eq!(
        content(AnnotationKind::Note).check(),
        Err(ModelError::Inconsistent)
    );
    let mut note = content(AnnotationKind::Note);
    note.text = Some(text);
    assert_eq!(note.check(), Ok(()));
    note.status = Some(TaskStatus::Open);
    assert_eq!(note.check(), Err(ModelError::Inconsistent));

    let mut task = content(AnnotationKind::Task);
    assert_eq!(task.check(), Err(ModelError::Inconsistent));
    task.title = Some(title);
    task.status = Some(TaskStatus::Open);
    assert_eq!(task.check(), Ok(()));
}
