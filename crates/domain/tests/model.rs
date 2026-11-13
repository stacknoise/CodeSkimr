//! Tests of the public domain model.

use std::str::FromStr;

use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_domain::{
    Confidence, FileId, MetricKind, MetricValue, ModelError, Position, ReferenceKind, ScanMode,
    Span, SymbolId, SymbolKind, SymbolName, Visibility,
};

fn at(line: u32, column: u32) -> Position {
    Position { line, column }
}

#[test]
fn symbol_kinds_match_blueprint_and_round_trip() {
    assert_eq!(SymbolKind::ALL.len(), 19);
    for kind in SymbolKind::ALL {
        assert_eq!(SymbolKind::from_str(kind.label()), Ok(*kind));
    }
    assert_eq!(SymbolKind::EnumMember.label(), "enum_member");
}

#[test]
fn reference_kinds_and_confidence_round_trip() {
    assert_eq!(ReferenceKind::ALL.len(), 9);
    for kind in ReferenceKind::ALL {
        assert_eq!(kind.to_string().parse::<ReferenceKind>(), Ok(*kind));
    }
    assert!(Confidence::Probable.is_probable());
    assert!(!Confidence::Certain.is_probable());
    assert_eq!("probable".parse::<Confidence>(), Ok(Confidence::Probable));
}

#[test]
fn other_vocabularies_round_trip() {
    for kind in MetricKind::ALL {
        assert_eq!(kind.label().parse::<MetricKind>(), Ok(*kind));
    }
    for mode in ScanMode::ALL {
        assert_eq!(mode.label().parse::<ScanMode>(), Ok(*mode));
    }
    for visibility in Visibility::ALL {
        assert_eq!(visibility.label().parse::<Visibility>(), Ok(*visibility));
    }
}

#[test]
fn unknown_label_is_rejected() {
    assert_eq!("klass".parse::<SymbolKind>(), Err(ModelError::UnknownLabel));
}

#[test]
fn labels_are_unique_per_enumeration() {
    let mut labels: Vec<_> = SymbolKind::ALL.iter().map(|kind| kind.label()).collect();
    labels.sort_unstable();
    labels.dedup();
    assert_eq!(labels.len(), SymbolKind::ALL.len());
}

#[test]
fn ids_keep_their_value_and_display() {
    assert_eq!(SymbolId::new(7).get(), 7);
    assert_eq!(FileId::new(3).to_string(), "3");
}

#[test]
fn names_reject_blank_text() {
    assert_eq!(SymbolName::new(""), Err(ModelError::Empty));
    assert_eq!(SymbolName::new("  \t"), Err(ModelError::Empty));
    assert_eq!(
        SymbolName::new("main").map(|n| n.to_string()),
        Ok("main".to_string())
    );
    assert_eq!("x".parse::<SymbolName>().map(|n| n.as_str().len()), Ok(1));
}

#[test]
fn span_must_not_be_inverted() {
    assert_eq!(Span::new(at(4, 0), at(3, 9)), Err(ModelError::InvertedSpan));
    let span = Span::new(at(2, 4), at(5, 0)).expect("valid span");
    assert_eq!(span.start(), at(2, 4));
    assert_eq!(span.end(), at(5, 0));
    assert_eq!(span.line_count(), 4);
    assert!(Span::new(at(1, 1), at(1, 1)).is_ok());
}

#[test]
fn metric_value_must_be_finite() {
    assert_eq!(MetricValue::new(f64::NAN), Err(ModelError::NotFinite));
    assert_eq!(MetricValue::new(f64::INFINITY), Err(ModelError::NotFinite));
    assert_eq!(MetricValue::new(2.5).map(MetricValue::get), Ok(2.5));
}

#[test]
fn every_model_error_has_a_registered_code() {
    for error in [
        ModelError::Empty,
        ModelError::InvertedSpan,
        ModelError::UnknownLabel,
        ModelError::NotFinite,
    ] {
        assert_eq!(error.code(), ErrorCode::UnexpectedInternalError);
        assert!(!error.to_string().is_empty());
    }
}
