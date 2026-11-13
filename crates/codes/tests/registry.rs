//! Tests of the generated error code API against the real registry.

use std::collections::HashSet;

use codeskimr_codes::{Class, Coded, ErrorCode, Group, Severity};

#[test]
fn ids_are_unique_and_round_trip() {
    let mut seen = HashSet::new();
    for code in ErrorCode::ALL {
        assert!(seen.insert(code.id()), "duplicate id {}", code.id());
        assert_eq!(ErrorCode::from_id(code.id()), Some(code));
    }
    assert_eq!(ErrorCode::from_id("XXX-999"), None);
}

#[test]
fn id_matches_group_and_class() {
    for code in ErrorCode::ALL {
        let expected = format!("{}-{}", code.group().prefix(), code.class().digit());
        assert!(
            code.id().starts_with(&expected),
            "{} vs {expected}",
            code.id()
        );
        assert_eq!(code.message_key(), format!("error-{}", code.id()));
    }
}

#[test]
fn known_codes_carry_their_data() {
    let code = ErrorCode::SourcePathNotFound;
    assert_eq!(code.id(), "SRC-201");
    assert_eq!(code.group(), Group::Src);
    assert_eq!(code.class(), Class::NotFound);
    assert_eq!(code.severity(), Severity::Error);
    assert!(!code.retryable());
    assert!(ErrorCode::GitFetchFailed.retryable());
    assert_eq!(ErrorCode::AmbiguousReference.severity(), Severity::Warning);
    assert_eq!(ErrorCode::ResponseLimitExceeded.severity(), Severity::Info);
    assert_eq!(Severity::Warning.label(), "warning");
}

#[test]
fn appendix_a_is_complete() {
    // 47 codes from blueprint Appendix A plus CFG-103 (WP-17), STO-601 and STO-901 (WP-21),
    // MCP-202 and MCP-203 (WP-26), AUTH-401 (WP-59), CLI-102 (WP-33), API-403, API-601, APP-202 and APP-601 (WP-42), EXP-201 (WP-75).
    assert_eq!(ErrorCode::ALL.len(), 64);
    assert_eq!(Group::ALL.len(), 18);
}

struct Sample;

impl Coded for Sample {
    fn code(&self) -> ErrorCode {
        ErrorCode::UnexpectedInternalError
    }
}

#[test]
fn coded_trait_returns_the_code() {
    assert_eq!(Sample.code().id(), "APP-901");
}
