//! The answer of a comparison as data that goes over the wire: the same shape, with `"schema": 1`,
//! for the registry commands `scans.compare` and `scans.impact`, the MCP tool `impact_of_diff`
//! and the commands `codeskimr compare` and `codeskimr change` (ADR 0097, decision 6).
//!
//! The types only copy what [`ScanComparison`] holds into plain fields; no decision is made here.

use codeskimr_codes::i18n::COMPARE_DIRECT_ONLY_NOTE;
use codeskimr_domain::{
    ChangeReason, CommitHash, ImpactCause, ImpactSubject, Matched, SymbolFact, Visibility,
};
use serde::Serialize;

use super::report::{
    CriticalChange, CriticalOutcome, ImpactUsage, LayerChange, Listed, ScanComparison,
    SymbolImpact, UsageStatus,
};
use crate::consts::{
    CAUSE_CHANGED, CAUSE_REMOVED, COMPARE_SCHEMA, OUTCOME_CHANGED, OUTCOME_MOVED, OUTCOME_REMOVED,
    REASON_CONTENT, REASON_SIGNATURE, REASON_VISIBILITY, STATUS_ADAPTED, STATUS_NOT_TOUCHED,
};
use crate::ports::{ScanEntry, SymbolLabel};

/// The lines of the editor start at one, the positions of the model at zero.
const LINE_OFFSET: u32 = 1;

/// One retained scan, as the list of scans and the head of a comparison show it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScanOut {
    /// Id of the snapshot the scan wrote; two of them can be compared.
    pub snapshot: u64,
    /// Identifies the scan itself; several scans that found nothing changed share one snapshot.
    pub scan: u64,
    /// When the scan ended, in seconds since the epoch.
    pub created: i64,
    /// Commit, if the source is a Git repository.
    pub commit: Option<String>,
    /// `full` or `incremental`.
    pub mode: &'static str,
    /// Number of files of the snapshot.
    pub files: u32,
    /// Number of symbols of the snapshot.
    pub symbols: u32,
    /// Wall time of the scan that wrote the snapshot in milliseconds; `None` for a snapshot
    /// written before the time was stored.
    pub duration_ms: Option<u64>,
    /// Files that scan changed against the previous snapshot; `None` for the first scan and for
    /// a snapshot written before this was stored.
    pub files_changed: Option<u32>,
    /// How long each step of writing the snapshot took, in the order of the write; empty for a
    /// scan that wrote nothing and for one stored before the times were kept.
    pub steps: Vec<StepOut>,
}

/// The time one step of writing a snapshot took.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StepOut {
    /// Label of the step (`PersistStep::label`).
    pub step: &'static str,
    /// Wall time of the step in milliseconds.
    pub duration_ms: u64,
}

impl From<&ScanEntry> for ScanOut {
    fn from(entry: &ScanEntry) -> Self {
        Self {
            snapshot: entry.snapshot.get(),
            scan: entry.scan,
            created: entry.created.0,
            commit: entry.commit.as_ref().map(CommitHash::to_string),
            mode: entry.mode.label(),
            files: entry.files,
            symbols: entry.symbols,
            duration_ms: entry.duration_ms,
            files_changed: entry.files_changed,
            steps: entry
                .steps
                .iter()
                .map(|time| StepOut {
                    step: time.step.label(),
                    duration_ms: time.duration_ms,
                })
                .collect(),
        }
    }
}

/// The list of the last scans of a project (`scans.list`, `codeskimr scans`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScanListAnswer {
    /// Version of this format.
    pub schema: u32,
    /// The retained scans, newest first.
    pub scans: Vec<ScanOut>,
    /// How many scans the project keeps (`snapshots.retention_count`).
    pub keep: u32,
    /// Average duration in milliseconds of the scans that have one; `None` if none has.
    pub average_ms: Option<u64>,
}

impl ScanListAnswer {
    /// The list for the retained scans `history`, newest first, of a project that keeps `keep`.
    #[must_use]
    pub fn new(history: &[ScanEntry], keep: u32) -> Self {
        let scans: Vec<ScanOut> = history.iter().map(ScanOut::from).collect();
        Self {
            schema: COMPARE_SCHEMA,
            average_ms: average(&scans),
            keep,
            scans,
        }
    }
}

fn average(scans: &[ScanOut]) -> Option<u64> {
    let times: Vec<u64> = scans.iter().filter_map(|scan| scan.duration_ms).collect();
    let count = u64::try_from(times.len()).ok().filter(|count| *count > 0)?;
    Some(times.iter().sum::<u64>() / count)
}

/// A list that was cut to its limit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ListedOut<T> {
    /// How many entries there were before the cut.
    pub total: usize,
    /// Whether entries were left out.
    pub is_cut: bool,
    /// The entries that are listed.
    pub items: Vec<T>,
}

fn listed<T, U: for<'a> From<&'a T>>(list: &Listed<T>) -> ListedOut<U> {
    ListedOut {
        total: list.total,
        is_cut: list.is_cut(),
        items: list.items.iter().map(U::from).collect(),
    }
}

/// A symbol of one of the two scans.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SymbolOut {
    /// Id of the symbol within its snapshot.
    pub id: u64,
    /// Kind, for instance `class` or `method`.
    pub kind: &'static str,
    /// Fully qualified name.
    pub qualified_name: String,
    /// Signature as written, if any.
    pub signature: Option<String>,
    /// Visibility, if the language has one.
    pub visibility: Option<&'static str>,
    /// File that declares it.
    pub path: String,
}

impl From<&SymbolFact> for SymbolOut {
    fn from(fact: &SymbolFact) -> Self {
        Self {
            id: fact.id.get(),
            kind: fact.kind.label(),
            qualified_name: fact.qualified_name.as_str().to_owned(),
            signature: fact.signature.clone(),
            visibility: fact.visibility.map(Visibility::label),
            path: fact.path.as_str().to_owned(),
        }
    }
}

/// A symbol of both scans, as the base and as the target have it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MatchedOut {
    /// The symbol in the older scan.
    pub base: SymbolOut,
    /// The symbol in the newer scan.
    pub target: SymbolOut,
    /// `signature`, `visibility` or `content`; empty for a symbol that only moved.
    pub reasons: Vec<&'static str>,
    /// Whether the visibility got narrower.
    pub narrowed: bool,
}

const fn reason_label(reason: ChangeReason) -> &'static str {
    match reason {
        ChangeReason::Signature => REASON_SIGNATURE,
        ChangeReason::Visibility { .. } => REASON_VISIBILITY,
        ChangeReason::Content => REASON_CONTENT,
    }
}

fn is_narrowed(reasons: &[ChangeReason]) -> bool {
    reasons
        .iter()
        .any(|reason| matches!(reason, ChangeReason::Visibility { narrowed: true }))
}

fn reasons_out(reasons: &[ChangeReason]) -> Vec<&'static str> {
    reasons.iter().copied().map(reason_label).collect()
}

impl From<&Matched> for MatchedOut {
    fn from(matched: &Matched) -> Self {
        Self {
            base: SymbolOut::from(&matched.base),
            target: SymbolOut::from(&matched.target),
            reasons: reasons_out(&matched.reasons),
            narrowed: is_narrowed(&matched.reasons),
        }
    }
}

/// The symbol that holds a usage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HolderOut {
    /// Id of the symbol in the base scan.
    pub id: u64,
    /// Kind of the symbol.
    pub kind: &'static str,
    /// Fully qualified name.
    pub qualified_name: String,
}

impl From<&SymbolLabel> for HolderOut {
    fn from(label: &SymbolLabel) -> Self {
        Self {
            id: label.id.get(),
            kind: label.kind.label(),
            qualified_name: label.qualified_name.as_str().to_owned(),
        }
    }
}

/// One direct usage of a symbol, in the base scan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UsageOut {
    /// The symbol that holds the usage.
    pub from: HolderOut,
    /// The file of the usage.
    pub path: String,
    /// Line, counted from 1.
    pub line: u32,
    /// Column, counted from 1.
    pub column: u32,
    /// How the symbol is used, for instance `call`.
    pub kind: &'static str,
    /// `certain` or `probable`.
    pub confidence: &'static str,
    /// `adapted` if the holder is changed, moved or removed too, `not_touched` otherwise.
    pub status: &'static str,
}

impl From<&ImpactUsage> for UsageOut {
    fn from(usage: &ImpactUsage) -> Self {
        let start = usage.reference.span.start();
        Self {
            from: HolderOut::from(&usage.from),
            path: usage.path.as_str().to_owned(),
            line: start.line + LINE_OFFSET,
            column: start.column + LINE_OFFSET,
            kind: usage.reference.kind.label(),
            confidence: usage.reference.confidence.label(),
            status: match usage.status {
                UsageStatus::Adapted => STATUS_ADAPTED,
                UsageStatus::NotTouched => STATUS_NOT_TOUCHED,
            },
        }
    }
}

/// A symbol whose usages a change can break, with those usages.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SymbolImpactOut {
    /// The symbol as the base scan has it.
    pub symbol: SymbolOut,
    /// `removed` or `changed`.
    pub cause: &'static str,
    /// For `changed`: `signature` and/or `visibility`.
    pub reasons: Vec<&'static str>,
    /// Whether the visibility got narrower.
    pub narrowed: bool,
    /// Its direct usages, cut to `compare.max_usages`.
    pub usages: ListedOut<UsageOut>,
    /// How many of the listed usages are not touched: the checklist.
    pub not_touched: usize,
}

fn subject_parts(subject: &ImpactSubject) -> (&'static str, &[ChangeReason]) {
    match &subject.cause {
        ImpactCause::Removed => (CAUSE_REMOVED, &[]),
        ImpactCause::Changed(reasons) => (CAUSE_CHANGED, reasons),
    }
}

impl From<&SymbolImpact> for SymbolImpactOut {
    fn from(impact: &SymbolImpact) -> Self {
        let (cause, reasons) = subject_parts(&impact.subject);
        Self {
            symbol: SymbolOut::from(&impact.subject.symbol),
            cause,
            reasons: reasons_out(reasons),
            narrowed: is_narrowed(reasons),
            usages: listed(&impact.usages),
            not_touched: impact.not_touched(),
        }
    }
}

/// A critical symbol that a change touches.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CriticalOut {
    /// Id of the critical annotation.
    pub annotation: u64,
    /// Kind of the marked symbol.
    pub kind: &'static str,
    /// Its qualified name in the base scan.
    pub qualified_name: String,
    /// `removed`, `moved` or `changed`.
    pub outcome: &'static str,
}

impl From<&CriticalChange> for CriticalOut {
    fn from(change: &CriticalChange) -> Self {
        Self {
            annotation: change.annotation.get(),
            kind: change.kind.label(),
            qualified_name: change.name.as_str().to_owned(),
            outcome: match change.outcome {
                CriticalOutcome::Removed => OUTCOME_REMOVED,
                CriticalOutcome::Moved => OUTCOME_MOVED,
                CriticalOutcome::Changed => OUTCOME_CHANGED,
            },
        }
    }
}

/// A dependency between two layers that is new in the target scan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LayerEdgeOut {
    /// The depending layer.
    pub from: String,
    /// The layer it depends on.
    pub to: String,
    /// How many references are behind the edge.
    pub count: u32,
}

impl From<&LayerChange> for LayerEdgeOut {
    fn from(edge: &LayerChange) -> Self {
        Self {
            from: edge.from.as_str().to_owned(),
            to: edge.to.as_str().to_owned(),
            count: edge.count,
        }
    }
}

/// The layer rules the target breaks and the base did not.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LayersOut {
    /// Edges that break the declared layering.
    pub new_violations: Vec<LayerEdgeOut>,
    /// Edges that lie on a dependency cycle.
    pub new_cycles: Vec<LayerEdgeOut>,
}

/// The numbers at the head of the answer, counted over the listed entries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SummaryOut {
    /// Symbols the change touches in their signature, visibility or content, or removes.
    pub changed_symbols: usize,
    /// Of those, the public ones in the listed entries.
    pub public_symbols: usize,
    /// Listed usages that are certain; `None` without the impact.
    pub usages_certain: Option<usize>,
    /// Listed usages that are probable; `None` without the impact.
    pub usages_probable: Option<usize>,
    /// Listed usages that are not touched: the checklist; `None` without the impact.
    pub usages_not_touched: Option<usize>,
}

/// The answer to a comparison of two scans of one project.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ComparisonAnswer {
    /// Version of this format.
    pub schema: u32,
    /// Id of the project.
    pub project: u64,
    /// The older scan.
    pub base: ScanOut,
    /// The newer scan.
    pub target: ScanOut,
    /// Counts for the head of the answer.
    pub summary: SummaryOut,
    /// Symbols only the target has.
    pub added: ListedOut<SymbolOut>,
    /// Symbols only the base has.
    pub removed: ListedOut<SymbolOut>,
    /// Symbols that changed in signature, visibility or content.
    pub changed: ListedOut<MatchedOut>,
    /// Symbols that have the same key in another file.
    pub moved: ListedOut<MatchedOut>,
    /// Critical symbols a change touches.
    pub critical: Vec<CriticalOut>,
    /// New layer violations and cycles.
    pub layers: LayersOut,
    /// The symbols whose usages a change can break; `None` for a plain comparison.
    pub impact: Option<ListedOut<SymbolImpactOut>>,
    /// Whether any list was cut to its limit.
    pub is_cut: bool,
    /// Always `true`: only direct usages are known.
    pub direct_usages_only: bool,
    /// Key of the Fluent text that says what the comparison does not see.
    pub note_key: &'static str,
    /// That text, if the surface that answers has a language to say it in.
    pub note: Option<String>,
}

fn edges(list: &[LayerChange]) -> Vec<LayerEdgeOut> {
    list.iter().map(LayerEdgeOut::from).collect()
}

fn summary(comparison: &ScanComparison) -> SummaryOut {
    let public = |fact: &SymbolFact| fact.visibility == Some(Visibility::Public);
    let public_symbols = comparison
        .removed
        .items
        .iter()
        .filter(|f| public(f))
        .count()
        + comparison
            .changed
            .items
            .iter()
            .filter(|matched| public(&matched.base))
            .count();
    let usages = comparison.impact.as_ref().map(|impact| {
        impact
            .symbols
            .items
            .iter()
            .flat_map(|one| one.usages.items.iter())
    });
    let count = |wanted: fn(&ImpactUsage) -> bool| {
        usages
            .clone()
            .map(|all| all.filter(|usage| wanted(usage)).count())
    };
    SummaryOut {
        changed_symbols: comparison.removed.total + comparison.changed.total,
        public_symbols,
        usages_certain: count(|usage| !usage.reference.confidence.is_probable()),
        usages_probable: count(|usage| usage.reference.confidence.is_probable()),
        usages_not_touched: count(|usage| usage.status == UsageStatus::NotTouched),
    }
}

impl ComparisonAnswer {
    /// The answer for `comparison`; `note` is the text that says what the comparison does not see,
    /// in the language of the surface that answers.
    #[must_use]
    pub fn new(comparison: &ScanComparison, note: Option<String>) -> Self {
        Self {
            schema: COMPARE_SCHEMA,
            project: comparison.project.get(),
            base: ScanOut::from(&comparison.base),
            target: ScanOut::from(&comparison.target),
            summary: summary(comparison),
            added: listed(&comparison.added),
            removed: listed(&comparison.removed),
            changed: listed(&comparison.changed),
            moved: listed(&comparison.moved),
            critical: comparison.critical.iter().map(CriticalOut::from).collect(),
            layers: LayersOut {
                new_violations: edges(&comparison.layers.new_violations),
                new_cycles: edges(&comparison.layers.new_cycles),
            },
            impact: comparison
                .impact
                .as_ref()
                .map(|impact| listed(&impact.symbols)),
            is_cut: comparison.is_cut(),
            direct_usages_only: true,
            note_key: COMPARE_DIRECT_ONLY_NOTE,
            note,
        }
    }
}
