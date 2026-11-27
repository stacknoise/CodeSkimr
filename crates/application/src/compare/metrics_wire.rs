//! The answer of the metric changes as data that goes over the wire: the same shape, with
//! `"schema": 1`, for the registry command `scans.metrics`, the MCP tool `impact_of_diff` and the
//! command `codeskimr compare --metrics` (ADR 0114, decision 6).
//!
//! The types only copy what [`MetricComparison`] holds into plain fields; no decision is made here.

use codeskimr_codes::ErrorCode;
use codeskimr_domain::{
    Crossing, MetricChange, MetricKind, MetricLevel, MetricMove, MoveDirection, ProjectId,
    SnapshotId, Verdict,
};
use serde::Serialize;

use super::metrics::{MetricComparison, MetricsRequest};
use super::report::Listed;
use super::wire::{ListedOut, ScanOut};
use crate::consts::{
    COMPARE_SCHEMA, CROSSING_INTO_POOR, CROSSING_OUT_OF_POOR, DIRECTION_BETTER, DIRECTION_GREW,
    DIRECTION_WORSE, LEVEL_MEMBER, LEVEL_MODULE, LEVEL_TYPE, VERDICT_BETTER, VERDICT_MIXED,
    VERDICT_WORSE,
};
use crate::error::CodedError;

/// One metric that moved.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MoveOut {
    /// The metric: `loc`, `cc`, `mi`, `ce` or `instability`.
    pub metric: &'static str,
    /// The value in the older scan.
    pub from: f64,
    /// The value in the newer scan.
    pub to: f64,
    /// `worse`, `better` or `grew` (lines, marked only).
    pub direction: &'static str,
    /// `into_poor` or `out_of_poor` if the move crossed the bound of the rating.
    pub crossing: Option<&'static str>,
}

impl From<&MetricMove> for MoveOut {
    fn from(found: &MetricMove) -> Self {
        Self {
            metric: found.kind.label(),
            from: found.from,
            to: found.to,
            direction: direction_label(found.direction),
            crossing: found.crossing.map(crossing_label),
        }
    }
}

const fn direction_label(direction: MoveDirection) -> &'static str {
    match direction {
        MoveDirection::Worse => DIRECTION_WORSE,
        MoveDirection::Better => DIRECTION_BETTER,
        MoveDirection::Grew => DIRECTION_GREW,
    }
}

const fn crossing_label(crossing: Crossing) -> &'static str {
    match crossing {
        Crossing::IntoPoor => CROSSING_INTO_POOR,
        Crossing::OutOfPoor => CROSSING_OUT_OF_POOR,
    }
}

const fn verdict_label(verdict: Verdict) -> &'static str {
    match verdict {
        Verdict::Worse => VERDICT_WORSE,
        Verdict::Better => VERDICT_BETTER,
        Verdict::Mixed => VERDICT_MIXED,
    }
}

const fn level_label(level: MetricLevel) -> &'static str {
    match level {
        MetricLevel::Module => LEVEL_MODULE,
        MetricLevel::Type => LEVEL_TYPE,
        MetricLevel::Member => LEVEL_MEMBER,
    }
}

/// The level a label names (`module`, `type` or `member`); `None` for any other text.
fn parse_level(label: &str) -> Option<MetricLevel> {
    [MetricLevel::Module, MetricLevel::Type, MetricLevel::Member]
        .into_iter()
        .find(|level| level_label(*level) == label)
}

/// The metrics a request may name to ask only for them; `None` for a metric that is not judged.
fn parse_judged_metric(label: &str) -> Option<MetricKind> {
    [
        MetricKind::CyclomaticComplexity,
        MetricKind::MaintainabilityIndex,
        MetricKind::EfferentCoupling,
        MetricKind::Instability,
        MetricKind::Loc,
    ]
    .into_iter()
    .find(|kind| kind.label() == label)
}

/// What a caller names in text to narrow the metric changes: the same for the registry command,
/// the command line and the MCP tool.
#[derive(Debug, Clone, Copy, Default)]
pub struct MetricsWanted<'a> {
    /// `module`, `type` or `member`; `member` if left out.
    pub level: Option<&'a str>,
    /// Only objects in which one of these metrics moved; every judged metric if empty.
    pub only: &'a [String],
    /// Whether test code is looked at too.
    pub include_tests: bool,
}

/// The request for the scans `(base, target)` of `project` as the caller named it.
///
/// # Errors
/// `API-101` for a level or a metric that is not known.
pub fn metrics_request(
    project: ProjectId,
    (base, target): (Option<SnapshotId>, Option<SnapshotId>),
    wanted: MetricsWanted<'_>,
) -> Result<MetricsRequest, CodedError> {
    let invalid = || CodedError::new(ErrorCode::RequestInvalid);
    let level = wanted.level.map_or(Ok(MetricLevel::Member), |text| {
        parse_level(text).ok_or_else(invalid)
    })?;
    let only = wanted
        .only
        .iter()
        .map(|text| parse_judged_metric(text).ok_or_else(invalid))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(MetricsRequest {
        project,
        base,
        target,
        level,
        include_tests: wanted.include_tests,
        only,
    })
}

/// One object whose metrics moved.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MetricChangeOut {
    /// Fully qualified name.
    pub name: String,
    /// Kind of the symbol, such as `method` or `class`.
    pub kind: &'static str,
    /// The file that declares it in the newer scan.
    pub path: String,
    /// `worse`, `better` or `mixed`; a mixed object is on both lists.
    pub verdict: &'static str,
    /// Whether a move crossed the bound of the rating: such an object comes first.
    pub crosses: bool,
    /// The moves of the side of the list the object is on, and the marks of grown lines.
    pub moves: Vec<MoveOut>,
}

impl MetricChangeOut {
    fn new(change: &MetricChange, side: MoveDirection) -> Self {
        Self {
            name: change.target.qualified_name.as_str().to_owned(),
            kind: change.target.kind.label(),
            path: change.target.path.as_str().to_owned(),
            verdict: verdict_label(change.verdict),
            crosses: change.crosses_on(side),
            moves: change.moves_of(side).iter().map(MoveOut::from).collect(),
        }
    }
}

fn side(list: &Listed<MetricChange>, side: MoveDirection) -> ListedOut<MetricChangeOut> {
    ListedOut {
        total: list.total,
        is_cut: list.is_cut(),
        items: list
            .items
            .iter()
            .map(|change| MetricChangeOut::new(change, side))
            .collect(),
    }
}

/// The answer to the comparison of the metrics of two scans of one project.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MetricsAnswer {
    /// Version of this format.
    pub schema: u32,
    /// Id of the project.
    pub project: u64,
    /// The older scan.
    pub base: ScanOut,
    /// The newer scan.
    pub target: ScanOut,
    /// `module`, `type` or `member`.
    pub level: &'static str,
    /// How many objects of the level the newer scan has; the rest did not move enough.
    pub looked_at: usize,
    /// Objects with a move to the worse, the worst first.
    pub worse: ListedOut<MetricChangeOut>,
    /// Objects with a move to the better, the best first.
    pub better: ListedOut<MetricChangeOut>,
    /// Whether a list was cut to its limit.
    pub is_cut: bool,
}

impl MetricsAnswer {
    /// The answer for a comparison of the metrics.
    #[must_use]
    pub fn new(found: &MetricComparison) -> Self {
        Self {
            schema: COMPARE_SCHEMA,
            project: found.project.get(),
            base: ScanOut::from(&found.base),
            target: ScanOut::from(&found.target),
            level: level_label(found.level),
            looked_at: found.looked_at,
            worse: side(&found.worse, MoveDirection::Worse),
            better: side(&found.better, MoveDirection::Better),
            is_cut: found.worse.is_cut() || found.better.is_cut(),
        }
    }
}
