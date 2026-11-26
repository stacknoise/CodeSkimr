//! What a comparison of two scans answers (ADR 0097): the same result for the screen, the command
//! line and the agent.

use codeskimr_domain::{
    AnnotationId, ImpactSubject, LayerName, Matched, Moved, ProjectId, QualifiedName, Reference,
    RelativePath, SnapshotId, SymbolFact, SymbolKind,
};

use crate::ports::{ScanEntry, SymbolLabel};

/// A list cut to a limit: the total tells how many there were before.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Listed<T> {
    /// How many entries there were before the cut.
    pub total: usize,
    /// The first entries, at most the limit.
    pub items: Vec<T>,
}

impl<T> Listed<T> {
    /// The first `limit` of `items`, with the total of all.
    #[must_use]
    pub fn cut(mut items: Vec<T>, limit: usize) -> Self {
        let total = items.len();
        items.truncate(limit);
        Self { total, items }
    }

    /// Whether entries were left out.
    #[must_use]
    pub fn is_cut(&self) -> bool {
        self.total > self.items.len()
    }
}

/// Which two scans of a project to compare. A snapshot left out is chosen: the target is the
/// newest scan, the base the scan before the target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComparisonRequest {
    /// The project.
    pub project: ProjectId,
    /// The older scan; `None` takes the one before the target.
    pub base: Option<SnapshotId>,
    /// The newer scan; `None` takes the newest.
    pub target: Option<SnapshotId>,
    /// Whether to read the usages of the symbols a change can break. Without them the answer is
    /// the plain comparison, which costs no usage reads.
    pub with_impact: bool,
}

/// Whether a usage sits in a symbol that the comparison changes, moves or removes itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsageStatus {
    /// The symbol around the usage is changed, moved or removed too: the usage is adapted.
    Adapted,
    /// The symbol around the usage is untouched: the usage is on the checklist.
    NotTouched,
}

/// One usage of a symbol of the base scan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImpactUsage {
    /// The symbol that holds the usage, as the base scan has it.
    pub from: SymbolLabel,
    /// The file of the usage.
    pub path: RelativePath,
    /// The reference with its kind, place and confidence (sure or probable).
    pub reference: Reference,
    /// Adapted or not touched.
    pub status: UsageStatus,
}

/// The direct usages of one symbol that a change can break, in the base scan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolImpact {
    /// The symbol and why its usages are looked at.
    pub subject: ImpactSubject,
    /// Its direct usages, cut to `compare.max_usages`.
    pub usages: Listed<ImpactUsage>,
}

impl SymbolImpact {
    /// How many of the listed usages are not touched: the checklist.
    #[must_use]
    pub fn not_touched(&self) -> usize {
        self.usages
            .items
            .iter()
            .filter(|usage| usage.status == UsageStatus::NotTouched)
            .count()
    }
}

/// The usages that the changes of a comparison hit. Only direct usages are known: calls by
/// reflection, in strings or from other repositories are not seen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Impact {
    /// The symbols whose usages can break, cut to `compare.max_symbols`.
    pub symbols: Listed<SymbolImpact>,
}

/// A dependency between two layers that is new in the target scan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayerChange {
    /// The depending layer.
    pub from: LayerName,
    /// The layer it depends on.
    pub to: LayerName,
    /// How many references are behind the edge in the target scan.
    pub count: u32,
}

/// The layer rules that the target breaks and the base did not.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LayerChanges {
    /// Edges that break the declared layering in the target and not in the base.
    pub new_violations: Vec<LayerChange>,
    /// Edges that lie on a dependency cycle in the target and not in the base.
    pub new_cycles: Vec<LayerChange>,
}

/// What happened to a symbol that carries a critical mark.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CriticalOutcome {
    /// It is gone.
    Removed,
    /// It has another name now.
    Renamed,
    /// It is in another file now.
    Moved,
    /// Its signature, visibility or content changed.
    Changed,
}

/// A critical symbol that a change touches (ADR 0068).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CriticalChange {
    /// The critical annotation.
    pub annotation: AnnotationId,
    /// Kind of the marked symbol.
    pub kind: SymbolKind,
    /// Its qualified name in the base scan.
    pub name: QualifiedName,
    /// What happened to it.
    pub outcome: CriticalOutcome,
}

/// The answer to a comparison of two scans of one project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanComparison {
    /// The project.
    pub project: ProjectId,
    /// The older scan.
    pub base: ScanEntry,
    /// The newer scan.
    pub target: ScanEntry,
    /// Symbols only the target has, cut to `compare.max_symbols`.
    pub added: Listed<SymbolFact>,
    /// Symbols only the base has, cut likewise.
    pub removed: Listed<SymbolFact>,
    /// Symbols that changed in signature, visibility or content, cut likewise.
    pub changed: Listed<Matched>,
    /// Symbols that moved to another file or were renamed, cut likewise.
    pub moved: Listed<Moved>,
    /// Critical symbols a change touches, all of them.
    pub critical: Vec<CriticalChange>,
    /// New layer violations and cycles.
    pub layers: LayerChanges,
    /// The usages a change hits; `None` if the request did not ask for them.
    pub impact: Option<Impact>,
}

impl ScanComparison {
    /// Whether any of the lists was cut to its limit.
    #[must_use]
    pub fn is_cut(&self) -> bool {
        self.added.is_cut()
            || self.removed.is_cut()
            || self.changed.is_cut()
            || self.moved.is_cut()
            || self.impact.as_ref().is_some_and(|impact| {
                impact.symbols.is_cut()
                    || impact.symbols.items.iter().any(|one| one.usages.is_cut())
            })
    }
}
