//! The comparison of two scans and the impact of the difference: one use case for the screen, the
//! command line and the agent (ADR 0097).
//!
//! The use case reads two retained snapshots of one project through ports. Each read is one short
//! read transaction of its own, so a running scan is never blocked; after the reads the use case
//! checks once more that both scans are still retained, so a scan that retention removed in
//! between is reported as `CMP-201` and never as a half answer.

mod critical;
mod impact;
mod layers;
pub mod metrics;
mod metrics_wire;
pub mod report;
mod select;
mod wire;

use std::sync::Arc;

use codeskimr_codes::ErrorCode;
use codeskimr_domain::{AnnotationKind, Comparison, ProjectId, SnapshotId, compare_symbols};

use crate::config::Compare;
pub use crate::consts::CROSSING_INTO_POOR;
use crate::error::CodedError;
use crate::ports::{
    AnnotationFilter, AnnotationStore, CodeIndex, ComparisonSource, ScanEntry, UsageIndex,
};
pub use metrics::{MetricComparison, MetricRules, MetricsRequest};
pub use metrics_wire::{MetricChangeOut, MetricsAnswer, MetricsWanted, MoveOut, metrics_request};
pub use report::{
    ComparisonRequest, CriticalChange, CriticalOutcome, Impact, ImpactUsage, LayerChange,
    LayerChanges, Listed, ScanComparison, SymbolImpact, UsageStatus,
};
pub use wire::{
    ComparisonAnswer, CriticalOut, HolderOut, LayerEdgeOut, LayersOut, ListedOut, MatchedOut,
    ScanListAnswer, ScanOut, StepOut, SummaryOut, SymbolImpactOut, SymbolOut, UsageOut,
};

/// The limits of the lists of a comparison (`compare.*`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompareLimits {
    /// Most entries of one list of symbols.
    pub max_symbols: usize,
    /// Most usages listed per symbol.
    pub max_usages: u32,
}

impl From<&Compare> for CompareLimits {
    fn from(compare: &Compare) -> Self {
        Self {
            max_symbols: usize::try_from(compare.max_symbols).unwrap_or(usize::MAX),
            max_usages: compare.max_usages,
        }
    }
}

/// The ports the comparison reads through.
#[derive(Clone)]
pub struct ComparePorts {
    /// The retained scans of a project and the layer graph of each.
    pub index: Arc<dyn CodeIndex>,
    /// The symbols and the naming rules of a retained snapshot.
    pub source: Arc<dyn ComparisonSource>,
    /// The usages in a retained snapshot.
    pub usages: Arc<dyn UsageIndex>,
    /// The critical marks of the project.
    pub annotations: Arc<dyn AnnotationStore>,
}

/// What the surfaces ask: the comparison of two scans. [`ScanComparer`] answers it; a surface
/// that only needs the answer takes this trait, so a test can stand in for the stores.
pub trait ScanDiffs: Send + Sync {
    /// Compares the scans the request names; see [`ScanComparer::compare`].
    ///
    /// # Errors
    /// `CMP-201`, `CMP-101`, `CMP-301` or a failure of the `STO` group.
    fn compare(&self, request: &ComparisonRequest) -> Result<ScanComparison, CodedError>;

    /// Compares the metrics of the objects of two scans; see [`ScanComparer::compare_metrics`].
    ///
    /// # Errors
    /// `CMP-201`, `CMP-101`, `CMP-301` or a failure of the `STO` group.
    fn compare_metrics(&self, request: &MetricsRequest) -> Result<MetricComparison, CodedError>;
}

impl ScanDiffs for ScanComparer {
    fn compare(&self, request: &ComparisonRequest) -> Result<ScanComparison, CodedError> {
        Self::compare(self, request)
    }

    fn compare_metrics(&self, request: &MetricsRequest) -> Result<MetricComparison, CodedError> {
        Self::compare_metrics(self, request)
    }
}

/// Compares two retained scans of a project and lists what the difference hits.
pub struct ScanComparer {
    ports: ComparePorts,
    limits: CompareLimits,
    rules: MetricRules,
}

impl ScanComparer {
    /// A use case over the ports and the limits of `compare.*`.
    #[must_use]
    pub const fn new(ports: ComparePorts, limits: CompareLimits, rules: MetricRules) -> Self {
        Self {
            ports,
            limits,
            rules,
        }
    }

    /// Compares the two scans the request names (or the newest and the one before it) and, if
    /// the request asks for it, lists the direct usages in the base scan of every symbol a
    /// change can break, as adapted or not touched.
    ///
    /// # Errors
    /// `CMP-201` if a scan is not retained or there is no scan to compare with, `CMP-101` if the
    /// base is not older than the target, `CMP-301` if the scans used different naming rules (a
    /// full scan makes them comparable), or a failure of the `STO` group.
    pub fn compare(&self, request: &ComparisonRequest) -> Result<ScanComparison, CodedError> {
        let history = self.ports.index.scan_history(request.project)?;
        let (base, target) = select::choose(&history, (request.base, request.target))?;
        self.check_rules(base.snapshot, target.snapshot)?;
        let comparison = compare_symbols(
            &self.ports.source.comparison_facts(base.snapshot)?,
            &self.ports.source.comparison_facts(target.snapshot)?,
        );
        let impact = request
            .with_impact
            .then(|| {
                impact::read(
                    &*self.ports.usages,
                    base.snapshot,
                    &comparison,
                    &self.limits,
                )
            })
            .transpose()?;
        let layers = self.layer_changes(base.snapshot, target.snapshot)?;
        let critical = self.critical(request.project, &comparison)?;
        self.still_retained(request.project, (base.snapshot, target.snapshot))?;
        Ok(self.answer(
            request.project,
            (base, target),
            comparison,
            (layers, critical, impact),
        ))
    }

    /// `CMP-301` unless both scans named their symbols by the same rules.
    fn check_rules(&self, base: SnapshotId, target: SnapshotId) -> Result<(), CodedError> {
        let rules = |snapshot| self.ports.source.snapshot_naming_rules(snapshot);
        select::same_rules(rules(base)?.as_deref(), rules(target)?.as_deref())
    }

    /// The new layer violations and cycles of the target against the base.
    fn layer_changes(
        &self,
        base: SnapshotId,
        target: SnapshotId,
    ) -> Result<LayerChanges, CodedError> {
        Ok(layers::changes(
            &self.ports.index.layer_edges(base)?,
            &self.ports.index.layer_edges(target)?,
        ))
    }

    /// The critical symbols the comparison touches.
    fn critical(
        &self,
        project: ProjectId,
        comparison: &Comparison,
    ) -> Result<Vec<CriticalChange>, CodedError> {
        let marks = self.ports.annotations.list_annotations(&AnnotationFilter {
            project,
            target: None,
            kind: Some(AnnotationKind::Critical),
        })?;
        Ok(critical::touched(&marks, comparison))
    }

    /// `CMP-201` if retention removed one of the scans while they were read.
    fn still_retained(
        &self,
        project: ProjectId,
        (base, target): (SnapshotId, SnapshotId),
    ) -> Result<(), CodedError> {
        let history = self.ports.index.scan_history(project)?;
        let kept = |wanted: SnapshotId| history.iter().any(|entry| entry.snapshot == wanted);
        if kept(base) && kept(target) {
            Ok(())
        } else {
            Err(CodedError::new(ErrorCode::SnapshotNotFound))
        }
    }

    /// The answer, with every list cut to `compare.max_symbols`.
    fn answer(
        &self,
        project: ProjectId,
        (base, target): (ScanEntry, ScanEntry),
        comparison: Comparison,
        (layers, critical, impact): (LayerChanges, Vec<CriticalChange>, Option<Impact>),
    ) -> ScanComparison {
        let limit = self.limits.max_symbols;
        ScanComparison {
            project,
            base,
            target,
            added: Listed::cut(comparison.added, limit),
            removed: Listed::cut(comparison.removed, limit),
            changed: Listed::cut(comparison.changed, limit),
            moved: Listed::cut(comparison.moved, limit),
            critical,
            layers,
            impact,
        }
    }
}
