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
    // MCP-202 and MCP-203 (WP-26), AUTH-401 (WP-59), CLI-102 (WP-33), API-403, API-601, APP-202 and APP-601 (WP-42), SEC-202 and CLI-103 (WP-73), EXP-201 (WP-75), PRJ-303 (cancelling a scan, #168), RES-101, RES-302 and RES-501 (SCIP import, ADR 0033), RES-303 (symbols created by level 2, ADR 0055), RES-201, RES-502, RES-503, RES-504, RES-601, SEC-301 and SEC-403 (indexer runner, ADR 0033), API-502 and API-503 (server limits, #88), PRJ-501 (parallel scans, #125), CHK-106 (parameter limit, ADR 0048), APP-602 (embedded language packs, #327), CFG-104 (layers written from the UI, ADR 0065), APP-603 (tray start, ADR 0067), ANN-101, ANN-102, ANN-103, ANN-104, ANN-105, ANN-201, ANN-202 and ANN-301 (annotations and alerts, ADR 0068), CFG-105, APP-604 and API-504 (level 2 switch, ADR 0071), PRJ-502 (scan report, ADR 0072), PRS-102 (a source file that cannot be read is left out of the scan, #621), RES-304 (candidate limit, ADR 0084), PRJ-503 (memory limit of a scan, ADR 0087), EXP-501 and EXP-502 (one export per project, size limit, ADR 0090), PRJ-102 (display name of a project), PRJ-103 (scope of a project, ADR 0102), CHK-107 (architecture rules, ADR 0104), CMP-101, CMP-201 and CMP-301 (scan comparison, ADR 0097), APP-605 (open the project file, ADR 0111), CFG-106 (project file changed while it was edited, #811), APP-606 and APP-607 (log folder and log file, ADR 0115), PRJ-304 (one convention check per project, ADR 0116).
    assert_eq!(ErrorCode::ALL.len(), 113);
    assert_eq!(Group::ALL.len(), 20);
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
