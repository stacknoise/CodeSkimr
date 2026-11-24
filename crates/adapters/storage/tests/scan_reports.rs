//! The stored report of the last scan: header, level 2 lines, and what a later scan leaves alone.
#![cfg(test)]

#[cfg(test)]
mod common;

use codeskimr_application::ports::{
    LanguageLine, LanguageOutcome, LastScan, Level2Report, ReportWrite, ScanReport, ScanReportStore,
};
use codeskimr_codes::ErrorCode;
use codeskimr_domain::{LanguageId, ScanLevel, ScanOrigin, SkipReason, Timestamp};
use common::{PROJECT, code_of, project, store};

fn line(language: &str, outcome: LanguageOutcome) -> LanguageLine {
    LanguageLine {
        language: LanguageId::new(language).unwrap(),
        outcome,
    }
}

fn last(at: i64, level: ScanLevel, origin: ScanOrigin) -> LastScan {
    LastScan {
        at: Timestamp(at),
        level,
        origin,
    }
}

fn second_pass(at: i64, origin: ScanOrigin, languages: Vec<LanguageLine>) -> Level2Report {
    Level2Report {
        at: Timestamp(at),
        origin,
        languages,
    }
}

fn write(last: LastScan, level2: Option<Level2Report>) -> ReportWrite {
    ReportWrite {
        project: PROJECT,
        last,
        level2,
    }
}

fn mixed() -> Vec<LanguageLine> {
    vec![
        line(
            "csharp",
            LanguageOutcome::Checked {
                files: 328,
                stale: 3,
            },
        ),
        line(
            "go",
            LanguageOutcome::Failed(ErrorCode::ScanReportNotStored),
        ),
        line("rust", LanguageOutcome::Skipped(SkipReason::NotConfirmed)),
    ]
}

#[test]
fn a_project_without_a_stored_scan_has_no_report() {
    let store = store(3);
    store.save_project(&project(1, "/work/app", &[])).unwrap();
    assert_eq!(store.report(PROJECT).unwrap(), None);
}

#[test]
fn a_level_2_scan_is_stored_with_every_outcome_and_read_back_in_the_order_of_the_language() {
    let store = store(3);
    store.save_project(&project(1, "/work/app", &[])).unwrap();
    let level2 = second_pass(20, ScanOrigin::Browser, mixed());
    store
        .store_report(&write(
            last(10, ScanLevel::Two, ScanOrigin::Browser),
            Some(level2),
        ))
        .unwrap();
    let report = store.report(PROJECT).unwrap().unwrap();
    assert_eq!(report.last, last(10, ScanLevel::Two, ScanOrigin::Browser));
    let lines = report.level2.unwrap().languages;
    let languages: Vec<&str> = lines.iter().map(|l| l.language.as_str()).collect();
    assert_eq!(languages, ["csharp", "go", "rust"]);
    assert_eq!(
        lines[0].outcome,
        LanguageOutcome::Checked {
            files: 328,
            stale: 3
        }
    );
    assert_eq!(
        lines[1].outcome,
        LanguageOutcome::Failed(ErrorCode::ScanReportNotStored)
    );
    assert_eq!(
        lines[2].outcome,
        LanguageOutcome::Skipped(SkipReason::NotConfirmed)
    );
}

#[test]
fn a_level_1_scan_replaces_the_header_but_keeps_what_the_last_level_2_run_said() {
    let store = store(3);
    store.save_project(&project(1, "/work/app", &[])).unwrap();
    let level2 = second_pass(20, ScanOrigin::Terminal, mixed());
    store
        .store_report(&write(
            last(10, ScanLevel::Two, ScanOrigin::Terminal),
            Some(level2.clone()),
        ))
        .unwrap();
    store
        .store_report(&write(last(30, ScanLevel::One, ScanOrigin::Watch), None))
        .unwrap();
    assert_eq!(
        store.report(PROJECT).unwrap(),
        Some(ScanReport {
            last: last(30, ScanLevel::One, ScanOrigin::Watch),
            level2: Some(level2),
        })
    );
}

#[test]
fn a_new_level_2_run_replaces_the_lines_of_the_old_one() {
    let store = store(3);
    store.save_project(&project(1, "/work/app", &[])).unwrap();
    for (at, languages) in [
        (20, mixed()),
        (
            40,
            vec![line("go", LanguageOutcome::Skipped(SkipReason::Busy))],
        ),
    ] {
        store
            .store_report(&write(
                last(at, ScanLevel::One, ScanOrigin::Browser),
                Some(second_pass(at, ScanOrigin::Browser, languages)),
            ))
            .unwrap();
    }
    let level2 = store.report(PROJECT).unwrap().unwrap().level2.unwrap();
    assert_eq!(level2.at, Timestamp(40));
    assert_eq!(
        level2.languages,
        vec![line("go", LanguageOutcome::Skipped(SkipReason::Busy))]
    );
}

#[test]
fn a_pass_without_languages_is_still_a_report() {
    let store = store(3);
    store.save_project(&project(1, "/work/app", &[])).unwrap();
    store
        .store_report(&write(
            last(10, ScanLevel::One, ScanOrigin::Terminal),
            Some(second_pass(11, ScanOrigin::Terminal, Vec::new())),
        ))
        .unwrap();
    let level2 = store.report(PROJECT).unwrap().unwrap().level2.unwrap();
    assert_eq!(level2.languages, Vec::new());
}

#[test]
fn a_report_of_an_unknown_project_is_refused() {
    let store = store(3);
    let error = store
        .store_report(&write(last(10, ScanLevel::One, ScanOrigin::Terminal), None))
        .unwrap_err();
    assert_eq!(code_of(&error), "PRJ-201");
}
