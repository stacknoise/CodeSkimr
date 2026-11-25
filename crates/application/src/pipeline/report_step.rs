//! Stores what a scan says about itself, after its snapshot is written (ADR 0072, decision 9).

use std::sync::Arc;

use codeskimr_codes::ErrorCode;
use codeskimr_domain::{ScanLevel, ScanOrigin, Snapshot};

use crate::level2::{LanguageReport, Level2Outcome, Skipped};
use crate::ports::{
    Clock, LanguageLine, LanguageOutcome, LastScan, Level2Report, ReportWrite, ScanReportStore,
};
use codeskimr_domain::SkipReason;

/// Writes the report of a finished scan. A failure never undoes the snapshot: the caller turns it
/// into the warning `PRJ-502`.
pub struct ReportStep {
    store: Arc<dyn ScanReportStore>,
    clock: Arc<dyn Clock>,
}

impl ReportStep {
    /// A step that writes to `store` and reads the time of the pass from `clock`.
    #[must_use]
    pub fn new(store: Arc<dyn ScanReportStore>, clock: Arc<dyn Clock>) -> Self {
        Self { store, clock }
    }

    /// Stores the report of the scan that wrote `snapshot`. `level2` is `Some` only if the second
    /// pass ran, so a scan without it leaves the earlier level 2 report as it is.
    ///
    /// # Errors
    /// `PRJ-502` if the report could not be stored.
    pub fn store(
        &self,
        snapshot: &Snapshot,
        origin: ScanOrigin,
        level2: Option<&[LanguageReport]>,
    ) -> Result<(), ErrorCode> {
        self.store_lines(snapshot, origin, level2.map(lines_of))
    }

    /// Like [`ReportStep::store`], with the lines of the second pass already made.
    ///
    /// # Errors
    /// `PRJ-502` if the report could not be stored.
    pub fn store_lines(
        &self,
        snapshot: &Snapshot,
        origin: ScanOrigin,
        lines: Option<Vec<LanguageLine>>,
    ) -> Result<(), ErrorCode> {
        let checked = lines.as_deref().is_some_and(any_checked);
        let write = ReportWrite {
            project: snapshot.project,
            last: LastScan {
                at: snapshot.created,
                level: if checked {
                    ScanLevel::Two
                } else {
                    ScanLevel::One
                },
                origin,
            },
            level2: lines.map(|languages| Level2Report {
                at: self.clock.now(),
                origin,
                languages,
            }),
        };
        self.store
            .store_report(&write)
            .map_err(|_| ErrorCode::ScanReportNotStored)
    }
}

fn any_checked(lines: &[LanguageLine]) -> bool {
    lines
        .iter()
        .any(|line| matches!(line.outcome, LanguageOutcome::Checked { .. }))
}

fn lines_of(reports: &[LanguageReport]) -> Vec<LanguageLine> {
    reports
        .iter()
        .map(|report| LanguageLine {
            language: report.language.clone(),
            outcome: outcome_of(&report.outcome),
        })
        .collect()
}

/// The stored form of an outcome: counts and keys only, no tool problem text.
fn outcome_of(outcome: &Level2Outcome) -> LanguageOutcome {
    match outcome {
        Level2Outcome::Checked { files, stale, .. } => LanguageOutcome::Checked {
            files: *files,
            stale: *stale,
        },
        Level2Outcome::Skipped(skipped) => skipped_outcome(skipped),
    }
}

fn skipped_outcome(skipped: &Skipped) -> LanguageOutcome {
    match (skipped.reason(), skipped) {
        (Some(reason), _) => LanguageOutcome::Skipped(reason),
        (None, Skipped::Failed(code)) => LanguageOutcome::Failed(*code),
        // `reason` is `None` only for a failure.
        (None, _) => LanguageOutcome::Skipped(SkipReason::Cancelled),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use codeskimr_domain::{LanguageId, ProjectId, ScanMode, SnapshotId, SnapshotStats, Timestamp};

    use super::*;
    use crate::error::CodedError;
    use crate::ports::{ReportWrite, ScanReport, ToolIssue, ToolProblem};

    struct Keep(Mutex<Vec<ReportWrite>>);

    impl ScanReportStore for Keep {
        fn store_report(&self, write: &ReportWrite) -> Result<(), CodedError> {
            self.0.lock().unwrap().push(write.clone());
            Ok(())
        }

        fn report(&self, _: ProjectId) -> Result<Option<ScanReport>, CodedError> {
            Ok(None)
        }
    }

    struct Noon;

    impl Clock for Noon {
        fn now(&self) -> Timestamp {
            Timestamp(1200)
        }
    }

    fn snapshot() -> Snapshot {
        Snapshot {
            id: SnapshotId::new(3),
            project: ProjectId::new(1),
            commit: None,
            created: Timestamp(1000),
            mode: ScanMode::Full,
            stats: SnapshotStats::default(),
        }
    }

    fn report(language: &str, outcome: Level2Outcome) -> LanguageReport {
        LanguageReport {
            language: LanguageId::new(language).unwrap(),
            outcome,
        }
    }

    fn stored(level2: Option<&[LanguageReport]>) -> ReportWrite {
        let keep = Arc::new(Keep(Mutex::default()));
        let step = ReportStep::new(keep.clone(), Arc::new(Noon));
        step.store(&snapshot(), ScanOrigin::Browser, level2)
            .unwrap();
        let writes = keep.0.lock().unwrap();
        writes[0].clone()
    }

    #[test]
    fn a_scan_without_the_pass_is_level_one_and_carries_no_second_report() {
        let write = stored(None);
        assert_eq!(write.last.level, ScanLevel::One);
        assert_eq!(write.last.at, Timestamp(1000));
        assert_eq!(write.level2, None);
    }

    #[test]
    fn one_checked_language_makes_the_scan_level_two_and_the_pass_gets_the_time_of_the_clock() {
        let checked = Level2Outcome::Checked {
            files: 5,
            stale: 1,
            mapped: 40,
            dropped: 2,
            outside_scan: 0,
        };
        let write = stored(Some(&[
            report("csharp", checked),
            report("go", Level2Outcome::Skipped(Skipped::Stale)),
        ]));
        assert_eq!(write.last.level, ScanLevel::Two);
        let level2 = write.level2.unwrap();
        assert_eq!(level2.at, Timestamp(1200));
        assert_eq!(level2.origin, ScanOrigin::Browser);
        assert_eq!(
            level2.languages[0].outcome,
            LanguageOutcome::Checked { files: 5, stale: 1 }
        );
        assert_eq!(
            level2.languages[1].outcome,
            LanguageOutcome::Skipped(SkipReason::Stale)
        );
    }

    #[test]
    fn nothing_checked_stays_level_one_and_keeps_reasons_and_codes_but_no_tool_text() {
        let problem = ToolProblem {
            program: "scip-dotnet".to_owned(),
            issue: ToolIssue::Missing,
        };
        let write = stored(Some(&[
            report(
                "csharp",
                Level2Outcome::Skipped(Skipped::Unavailable(vec![problem])),
            ),
            report("java", Level2Outcome::Skipped(Skipped::NotConfirmed)),
            report("go", Level2Outcome::Skipped(Skipped::NoProjectFile)),
            report(
                "rust",
                Level2Outcome::Skipped(Skipped::Failed(ErrorCode::ScanReportNotStored)),
            ),
        ]));
        assert_eq!(write.last.level, ScanLevel::One);
        let outcomes: Vec<_> = write
            .level2
            .unwrap()
            .languages
            .into_iter()
            .map(|l| l.outcome)
            .collect();
        assert_eq!(
            outcomes,
            [
                LanguageOutcome::Skipped(SkipReason::ToolsUnavailable),
                LanguageOutcome::Skipped(SkipReason::NotConfirmed),
                LanguageOutcome::Skipped(SkipReason::NoProjectFile),
                LanguageOutcome::Failed(ErrorCode::ScanReportNotStored),
            ]
        );
    }
}
