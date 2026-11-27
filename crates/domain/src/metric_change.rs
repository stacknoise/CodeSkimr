//! Which objects got worse and which got better by their metrics between two scans (ADR 0114).
//!
//! Pure logic on facts that a store hands over: the same object is found in both scans by the key
//! of the comparison (kind, qualified name, parameter types), and each metric that has a direction
//! is judged against a threshold, so that noise is not listed.

use std::collections::HashMap;

use crate::compare::SymbolIdentity;
use crate::consts::MOVE_TOLERANCE;
use crate::file::RelativePath;
use crate::metric::MetricKind;
use crate::qualified_name::QualifiedName;
use crate::symbol::SymbolKind;

/// The measured values of one object; `None` where the metric does not exist for it.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct MetricValues {
    /// Lines of code.
    pub loc: Option<f64>,
    /// Cyclomatic complexity.
    pub cc: Option<f64>,
    /// Highest complexity of a callable below a type; shown, never judged.
    pub cc_max: Option<f64>,
    /// Maintainability index.
    pub mi: Option<f64>,
    /// Efferent coupling.
    pub ce: Option<f64>,
    /// Afferent coupling; shown, never judged.
    pub ca: Option<f64>,
    /// Instability, 0 to 1.
    pub instability: Option<f64>,
}

impl MetricValues {
    /// The value of `kind`.
    #[must_use]
    pub const fn get(&self, kind: MetricKind) -> Option<f64> {
        match kind {
            MetricKind::Loc => self.loc,
            MetricKind::CyclomaticComplexity => self.cc,
            MetricKind::CyclomaticComplexityMax => self.cc_max,
            MetricKind::MaintainabilityIndex => self.mi,
            MetricKind::EfferentCoupling => self.ce,
            MetricKind::AfferentCoupling => self.ca,
            MetricKind::Instability => self.instability,
        }
    }

    /// Sets the value of `kind`.
    pub fn set(&mut self, kind: MetricKind, value: f64) {
        let slot = match kind {
            MetricKind::Loc => &mut self.loc,
            MetricKind::CyclomaticComplexity => &mut self.cc,
            MetricKind::CyclomaticComplexityMax => &mut self.cc_max,
            MetricKind::MaintainabilityIndex => &mut self.mi,
            MetricKind::EfferentCoupling => &mut self.ce,
            MetricKind::AfferentCoupling => &mut self.ca,
            MetricKind::Instability => &mut self.instability,
        };
        *slot = Some(value);
    }
}

/// What a store hands over of one object of one snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct MetricFact {
    /// Kind of the symbol.
    pub kind: SymbolKind,
    /// Fully qualified name.
    pub qualified_name: QualifiedName,
    /// Signature as written; the parameter types tell overloads apart.
    pub signature: Option<String>,
    /// The file that declares it.
    pub path: RelativePath,
    /// Its measured values.
    pub values: MetricValues,
}

impl MetricFact {
    /// The key that names the object across snapshots.
    #[must_use]
    pub fn identity(&self) -> SymbolIdentity {
        SymbolIdentity::of(self.kind, &self.qualified_name, self.signature.as_deref())
    }
}

/// The level an object is listed on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricLevel {
    /// A namespace, module or package.
    Module,
    /// A type.
    Type,
    /// A callable, property, field, constant, event or enum member.
    Member,
}

impl MetricLevel {
    /// The level of a symbol kind; `None` for kinds that are not compared.
    #[must_use]
    pub const fn of(kind: SymbolKind) -> Option<Self> {
        if matches!(kind, SymbolKind::Namespace | SymbolKind::Module) {
            Some(Self::Module)
        } else if kind.is_type() {
            Some(Self::Type)
        } else if crate::compare::compares_kind(kind) {
            Some(Self::Member)
        } else {
            None
        }
    }
}

/// The size from which a move counts, and the bounds beyond which a value is rated poor.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MoveLimits {
    /// Smallest move of the complexity.
    pub cc: f64,
    /// Smallest move of the maintainability index.
    pub mi: f64,
    /// Smallest move of the efferent coupling.
    pub ce: f64,
    /// Smallest move of the instability (0 to 1).
    pub instability: f64,
    /// Growth in lines from which an object is marked.
    pub loc_growth: f64,
    /// How many dependents make a module count as used a lot: a rising instability is worse then.
    pub busy_dependents: f64,
    /// A complexity above this is rated poor.
    pub poor_cc_above: f64,
    /// A maintainability index below this is rated poor.
    pub poor_mi_below: f64,
    /// An instability above this is rated poor.
    pub poor_instability_above: f64,
}

/// Whether a move is for the better or the worse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveDirection {
    /// The metric got worse.
    Worse,
    /// The metric got better.
    Better,
    /// Lines grew by at least the growth limit: marked, never judged.
    Grew,
}

/// Whether a move crossed the bound that rates a value poor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Crossing {
    /// The value was not poor and is now.
    IntoPoor,
    /// The value was poor and is not any more.
    OutOfPoor,
}

/// One metric of one object that moved by at least its threshold.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MetricMove {
    /// Which metric.
    pub kind: MetricKind,
    /// The value in the base scan.
    pub from: f64,
    /// The value in the target scan.
    pub to: f64,
    /// For the better or the worse.
    pub direction: MoveDirection,
    /// Whether the move crossed the bound of the rating.
    pub crossing: Option<Crossing>,
}

/// The verdict on one object.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// Only moves to the worse.
    Worse,
    /// Only moves to the better.
    Better,
    /// Moves to both sides; the object is listed on both.
    Mixed,
}

/// One object that exists in both scans and whose metrics moved.
#[derive(Debug, Clone, PartialEq)]
pub struct MetricChange {
    /// The object in the base scan.
    pub base: MetricFact,
    /// The object in the target scan.
    pub target: MetricFact,
    /// The metrics that moved, directed ones first.
    pub moves: Vec<MetricMove>,
    /// The verdict.
    pub verdict: Verdict,
}

impl MetricChange {
    /// Whether a move to `direction` crosses the bound of the rating: such an object ranks first
    /// on the list of that side. A move to the worse can only enter the poor range and a move to
    /// the better can only leave it, so a crossing of the other side does not count here.
    #[must_use]
    pub fn crosses_on(&self, direction: MoveDirection) -> bool {
        self.moves
            .iter()
            .any(|m| m.direction == direction && m.crossing.is_some())
    }

    /// The moves of one side; the marks of grown lines go with every side.
    #[must_use]
    pub fn moves_of(&self, direction: MoveDirection) -> Vec<MetricMove> {
        self.moves
            .iter()
            .copied()
            .filter(|m| m.direction == direction || m.direction == MoveDirection::Grew)
            .collect()
    }

    /// How far the directed moves of `direction` go, in multiples of their thresholds.
    #[must_use]
    pub fn weight(&self, direction: MoveDirection, limits: &MoveLimits) -> f64 {
        self.moves
            .iter()
            .filter(|m| m.direction == direction)
            .map(|m| (m.to - m.from).abs() / threshold(m.kind, limits))
            .sum()
    }
}

/// The smallest move that counts for `kind`.
const fn threshold(kind: MetricKind, limits: &MoveLimits) -> f64 {
    match kind {
        MetricKind::CyclomaticComplexity => limits.cc,
        MetricKind::MaintainabilityIndex => limits.mi,
        MetricKind::EfferentCoupling => limits.ce,
        MetricKind::Instability => limits.instability,
        _ => limits.loc_growth,
    }
}

/// Whether a higher value of `kind` is the worse one; `None` for metrics without a direction.
const fn higher_is_worse(kind: MetricKind) -> Option<bool> {
    match kind {
        MetricKind::CyclomaticComplexity
        | MetricKind::EfferentCoupling
        | MetricKind::Instability => Some(true),
        MetricKind::MaintainabilityIndex => Some(false),
        _ => None,
    }
}

/// Whether `value` of `kind` is rated poor.
fn is_poor(kind: MetricKind, value: f64, limits: &MoveLimits) -> bool {
    match kind {
        MetricKind::CyclomaticComplexity => value > limits.poor_cc_above,
        MetricKind::MaintainabilityIndex => value < limits.poor_mi_below,
        MetricKind::Instability => value > limits.poor_instability_above,
        _ => false,
    }
}

/// The crossing of the rating bound by a move, if there is one.
fn crossing(kind: MetricKind, from: f64, to: f64, limits: &MoveLimits) -> Option<Crossing> {
    match (is_poor(kind, from, limits), is_poor(kind, to, limits)) {
        (false, true) => Some(Crossing::IntoPoor),
        (true, false) => Some(Crossing::OutOfPoor),
        _ => None,
    }
}

/// The move of one metric, if it moved by at least its threshold in a direction that counts.
fn judge_metric(
    kind: MetricKind,
    (base, target): (&MetricValues, &MetricValues),
    limits: &MoveLimits,
) -> Option<MetricMove> {
    let (from, to) = (base.get(kind)?, target.get(kind)?);
    let delta = to - from;
    if kind == MetricKind::Loc {
        return (delta >= limits.loc_growth).then_some(MetricMove {
            kind,
            from,
            to,
            direction: MoveDirection::Grew,
            crossing: None,
        });
    }
    let up_is_worse = higher_is_worse(kind)?;
    if delta.abs() + MOVE_TOLERANCE < threshold(kind, limits) {
        return None;
    }
    let worse = delta.is_sign_positive() == up_is_worse;
    let busy = target.ca.unwrap_or_default() >= limits.busy_dependents;
    if kind == MetricKind::Instability && worse && !busy {
        return None;
    }
    Some(MetricMove {
        kind,
        from,
        to,
        direction: if worse {
            MoveDirection::Worse
        } else {
            MoveDirection::Better
        },
        crossing: crossing(kind, from, to, limits),
    })
}

/// The metrics of one object that moved, and the verdict; `None` if no directed metric moved.
#[must_use]
pub fn judge(
    base: &MetricValues,
    target: &MetricValues,
    limits: &MoveLimits,
) -> Option<(Vec<MetricMove>, Verdict)> {
    const JUDGED: &[MetricKind] = &[
        MetricKind::CyclomaticComplexity,
        MetricKind::MaintainabilityIndex,
        MetricKind::EfferentCoupling,
        MetricKind::Instability,
        MetricKind::Loc,
    ];
    let moves: Vec<MetricMove> = JUDGED
        .iter()
        .filter_map(|kind| judge_metric(*kind, (base, target), limits))
        .collect();
    let any = |d: MoveDirection| moves.iter().any(|m| m.direction == d);
    let verdict = match (any(MoveDirection::Worse), any(MoveDirection::Better)) {
        (true, true) => Verdict::Mixed,
        (true, false) => Verdict::Worse,
        (false, true) => Verdict::Better,
        (false, false) => return None,
    };
    Some((moves, verdict))
}

/// The objects that exist in both scans and whose metrics moved, by the key of the comparison.
/// An object that exists in one scan only is not compared. The order is that of `target`.
#[must_use]
pub fn compare_metrics(
    base: &[MetricFact],
    target: &[MetricFact],
    limits: &MoveLimits,
) -> Vec<MetricChange> {
    let mut by_key: HashMap<SymbolIdentity, &MetricFact> = HashMap::with_capacity(base.len());
    for fact in base {
        by_key.entry(fact.identity()).or_insert(fact);
    }
    target
        .iter()
        .filter_map(|after| {
            let before = by_key.get(&after.identity())?;
            let (moves, verdict) = judge(&before.values, &after.values, limits)?;
            Some(MetricChange {
                base: (*before).clone(),
                target: after.clone(),
                moves,
                verdict,
            })
        })
        .collect()
}
