//! The report of a scan as the screens and the MCP server show it (ADR 0072, decisions 10 and 11).

use codeskimr_domain::{OutcomeKind, SkipReason};
use serde::Serialize;

use codeskimr_codes::Coded;
use codeskimr_domain::ProjectId;

use crate::ports::{LanguageLine, LanguageOutcome, Level2Report, ScanReport, ScanReportStore};

/// The newest scan of a project.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LastScanView {
    /// When the snapshot was written, in seconds since the Unix epoch.
    pub at: i64,
    /// `1` or `2`.
    pub level: &'static str,
    /// `terminal`, `browser`, `watch` or `mcp`.
    pub origin: &'static str,
}

/// What level 2 did for one language. Counts, keys and a code only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LanguageView {
    /// Id of the language pack.
    pub language: String,
    /// `checked`, `skipped` or `failed`.
    pub outcome: &'static str,
    /// Files whose references come from the index, for `checked`.
    pub files: Option<u64>,
    /// Files that changed meanwhile and kept level 1, for `checked`.
    pub stale: Option<u64>,
    /// Why the language was not checked, for `skipped`.
    pub reason: Option<&'static str>,
    /// The registered error code, for `failed`.
    pub code: Option<&'static str>,
}

/// The newest second pass of a project.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Level2View {
    /// When the pass ended, in seconds since the Unix epoch.
    pub at: i64,
    /// `terminal`, `browser`, `watch` or `mcp`.
    pub origin: &'static str,
    /// One entry per language.
    pub languages: Vec<LanguageView>,
}

/// Both parts of the report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReportView {
    /// The newest scan.
    pub last_scan: LastScanView,
    /// The newest second pass, if a scan ran one.
    pub level2: Option<Level2View>,
}

impl LanguageView {
    fn of(line: &LanguageLine) -> Self {
        let none = Self {
            language: line.language.to_string(),
            outcome: OutcomeKind::Skipped.label(),
            files: None,
            stale: None,
            reason: None,
            code: None,
        };
        match line.outcome {
            LanguageOutcome::Checked { files, stale } => Self {
                outcome: OutcomeKind::Checked.label(),
                files: Some(files),
                stale: Some(stale),
                ..none
            },
            LanguageOutcome::Skipped(reason) => Self {
                reason: Some(reason.label()),
                ..none
            },
            LanguageOutcome::Failed(code) => Self {
                outcome: OutcomeKind::Failed.label(),
                code: Some(code.id()),
                ..none
            },
        }
    }
}

impl Level2View {
    fn of(report: &Level2Report, keep: impl Fn(&LanguageLine) -> bool) -> Self {
        Self {
            at: report.at.0,
            origin: report.origin.label(),
            languages: report
                .languages
                .iter()
                .filter(|line| keep(line))
                .map(LanguageView::of)
                .collect(),
        }
    }
}

impl ReportView {
    /// The stored report of `project` for a screen. One that cannot be read is logged and left
    /// out: the answer of the screen is worth more than the line about the last scan.
    #[must_use]
    pub fn stored(store: &dyn ScanReportStore, project: ProjectId) -> Option<Self> {
        match store.report(project) {
            Ok(report) => report.as_ref().map(Self::of),
            Err(error) => {
                tracing::warn!(project = project.get(), code = error.code().id());
                None
            }
        }
    }

    /// The whole report, for the screens of the local owner.
    #[must_use]
    pub fn of(report: &ScanReport) -> Self {
        Self::filtered(report, |_| true)
    }

    /// The report for the MCP server: the reason `tools_unavailable` stays with the tool report,
    /// so a client does not learn which tool is missing from here.
    #[must_use]
    pub fn for_remote(report: &ScanReport) -> Self {
        Self::filtered(report, |line| {
            line.outcome != LanguageOutcome::Skipped(SkipReason::ToolsUnavailable)
        })
    }

    fn filtered(report: &ScanReport, keep: impl Fn(&LanguageLine) -> bool) -> Self {
        Self {
            last_scan: LastScanView {
                at: report.last.at.0,
                level: report.last.level.label(),
                origin: report.last.origin.label(),
            },
            level2: report
                .level2
                .as_ref()
                .map(|level2| Level2View::of(level2, keep)),
        }
    }
}
