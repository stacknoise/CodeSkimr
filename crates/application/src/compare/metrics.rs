//! The metric changes between two scans: which objects got worse and which got better (ADR 0114).
//!
//! One use case for the screen, the command line and the agent. The domain judges the pairs; this
//! module reads the two scans through the port, leaves out what the request does not ask for, and
//! sorts and cuts the lists.

use std::cmp::Ordering;

use codeskimr_domain::{
    MetricChange, MetricFact, MetricKind, MetricLevel, MoveDirection, MoveLimits, ProjectId,
    SnapshotId, Verdict, compare_metrics,
};

use super::{ScanComparer, report::Listed, select};
use crate::config::Config;
use crate::consts::PERMILLE;
use crate::conventions::checker::in_test_folder;
use crate::error::CodedError;
use crate::ports::ScanEntry;

/// What decides whether a move counts and which objects are looked at (`compare.metric_*`, the
/// rating bounds of `metrics.*`, `graph.risk_min_incoming` and `conventions.test_folders`).
#[derive(Debug, Clone, PartialEq)]
pub struct MetricRules {
    /// The thresholds and the bounds of the rating.
    pub limits: MoveLimits,
    /// Names of the folders that hold test code.
    pub test_folders: Vec<String>,
}

impl MetricRules {
    /// The rules of a configuration.
    #[must_use]
    pub fn from_config(config: &Config) -> Self {
        let (compare, rating) = (&config.compare, &config.metrics);
        let per_mille = PERMILLE;
        Self {
            limits: MoveLimits {
                cc: f64::from(compare.metric_min_cc),
                mi: f64::from(compare.metric_min_mi),
                ce: f64::from(compare.metric_min_ce),
                instability: f64::from(compare.metric_min_instability) / per_mille,
                loc_growth: f64::from(compare.metric_loc_growth),
                busy_dependents: f64::from(config.graph.risk_min_incoming),
                poor_cc_above: f64::from(rating.cyclomatic_medium_max),
                poor_mi_below: f64::from(rating.maintainability_medium_min),
                poor_instability_above: rating.instability_medium_max,
            },
            test_folders: config.conventions.test_folders.clone(),
        }
    }
}

/// Which two scans to compare and what to look at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricsRequest {
    /// The project.
    pub project: ProjectId,
    /// The older scan; `None` takes the one before the target.
    pub base: Option<SnapshotId>,
    /// The newer scan; `None` takes the newest.
    pub target: Option<SnapshotId>,
    /// The level to list: modules, types or members.
    pub level: MetricLevel,
    /// Whether test code is looked at too.
    pub include_tests: bool,
    /// The metrics that must have moved; empty means every judged metric.
    pub only: Vec<MetricKind>,
}

/// The answer: the objects that got worse and those that got better, worst and best first.
#[derive(Debug, Clone, PartialEq)]
pub struct MetricComparison {
    /// The project.
    pub project: ProjectId,
    /// The older scan.
    pub base: ScanEntry,
    /// The newer scan.
    pub target: ScanEntry,
    /// The level that was listed.
    pub level: MetricLevel,
    /// How many objects of the level the target scan has; the rest did not move enough.
    pub looked_at: usize,
    /// Objects with a move to the worse; a mixed object is on both lists.
    pub worse: Listed<MetricChange>,
    /// Objects with a move to the better.
    pub better: Listed<MetricChange>,
}

impl ScanComparer {
    /// Compares the metrics of the objects that exist in both scans the request names.
    ///
    /// # Errors
    /// `CMP-201` if a scan is not retained or there is no scan to compare with, `CMP-101` if the
    /// base is not older than the target, `CMP-301` if the scans used different naming rules, or a
    /// failure of the `STO` group.
    pub fn compare_metrics(
        &self,
        request: &MetricsRequest,
    ) -> Result<MetricComparison, CodedError> {
        let history = self.ports.index.scan_history(request.project)?;
        let (base, target) = select::choose(&history, (request.base, request.target))?;
        self.check_rules(base.snapshot, target.snapshot)?;
        let facts = |snapshot| -> Result<Vec<MetricFact>, CodedError> {
            Ok(self.keep(self.ports.source.metric_facts(snapshot)?, request))
        };
        let (before, after) = (facts(base.snapshot)?, facts(target.snapshot)?);
        let changes = compare_metrics(&before, &after, &self.rules.limits);
        self.still_retained(request.project, (base.snapshot, target.snapshot))?;
        let side = |direction| self.side(&changes, direction, &request.only);
        Ok(MetricComparison {
            project: request.project,
            base,
            target,
            level: request.level,
            looked_at: after.len(),
            worse: side(MoveDirection::Worse),
            better: side(MoveDirection::Better),
        })
    }

    /// The facts of the level the request asks for, without test code unless it asks for it.
    fn keep(&self, facts: Vec<MetricFact>, request: &MetricsRequest) -> Vec<MetricFact> {
        let folders = &self.rules.test_folders;
        facts
            .into_iter()
            .filter(|fact| MetricLevel::of(fact.kind) == Some(request.level))
            .filter(|fact| request.include_tests || !in_test_folder(&fact.path, folders))
            .collect()
    }

    /// The list of one side: the changes with a move that way, the most telling first, cut.
    fn side(
        &self,
        changes: &[MetricChange],
        direction: MoveDirection,
        only: &[MetricKind],
    ) -> Listed<MetricChange> {
        let wanted = |change: &&MetricChange| {
            let on_side = matches!(
                (change.verdict, direction),
                (Verdict::Mixed, _)
                    | (Verdict::Worse, MoveDirection::Worse)
                    | (Verdict::Better, MoveDirection::Better)
            );
            on_side
                && (only.is_empty()
                    || change
                        .moves
                        .iter()
                        .any(|m| m.direction == direction && only.contains(&m.kind)))
        };
        let mut listed: Vec<MetricChange> = changes.iter().filter(wanted).cloned().collect();
        let limits = &self.rules.limits;
        listed.sort_by(|a, b| order(a, b, direction, limits));
        Listed::cut(listed, self.limits.max_symbols)
    }
}

/// Crossing the bound of the rating first, then the bigger move in multiples of its threshold,
/// then the name, so that equal changes come in the same order every time.
fn order(
    a: &MetricChange,
    b: &MetricChange,
    direction: MoveDirection,
    limits: &MoveLimits,
) -> Ordering {
    b.crosses()
        .cmp(&a.crosses())
        .then_with(|| {
            b.weight(direction, limits)
                .total_cmp(&a.weight(direction, limits))
        })
        .then_with(|| a.target.qualified_name.cmp(&b.target.qualified_name))
}
