//! Metrics of the symbols of analysed files from the counts of the language pack (ADR 0015).
//!
//! The pack counts per callable; this module applies the pure functions of `metrics` and rolls the
//! values up to types, namespaces and modules: `cc` is summed, `cc_max` is the highest value and
//! `mi` is the mean weighted by the lines of code of the callables.

use std::collections::HashMap;

use crate::ModelError;
use crate::consts::CALLABLE_METRICS;
use crate::file::Loc;
use crate::metric::{Metric, MetricKind, MetricValue};
use crate::metrics::{HalsteadCounts, cyclomatic_complexity, maintainability_index};
use crate::symbol::{Symbol, SymbolId};

/// What the language pack counted in the body of one callable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BodyCounts {
    /// Decision points of the body itself, without nested named functions.
    pub decisions: u32,
    /// Operators and operands of the body.
    pub halstead: HalsteadCounts,
}

/// The counts of one measurable symbol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SymbolCounts {
    /// The callable, type, namespace or module.
    pub symbol: SymbolId,
    /// Code lines of the symbol (ADR 0015).
    pub loc: Loc,
    /// Counts of the body; `None` for anything but a callable with a body, and for a language
    /// pack without `metrics.scm`.
    pub body: Option<BodyCounts>,
    /// Whether the language pack counts decision points at all (it has `metrics.scm`). Without it
    /// a type or module has no `cc`, because 0 would be a measurement that did not happen.
    pub counted: bool,
}

/// Running totals of the callables below one type, namespace or module.
#[derive(Default)]
struct Rollup {
    cc: u32,
    cc_max: u32,
    weighted_mi: f64,
    weight: u32,
    callables: u32,
}

impl Rollup {
    fn add(&mut self, cc: u32, mi: f64, loc: Loc) {
        self.cc = self.cc.saturating_add(cc);
        self.cc_max = self.cc_max.max(cc);
        self.weighted_mi += mi * f64::from(loc.0);
        self.weight = self.weight.saturating_add(loc.0);
        self.callables += 1;
    }
}

fn metric(symbol: SymbolId, kind: MetricKind, value: f64) -> Result<Metric, ModelError> {
    Ok(Metric {
        symbol,
        kind,
        value: MetricValue::new(value)?,
    })
}

/// Measures every symbol in `counts`. `symbols` must hold the symbols of the counted files; their
/// parents give the rollup path. Counts of unknown symbols are ignored.
///
/// # Errors
/// [`ModelError::NotFinite`] if a value overflows, which `u32` counts rule out in practice.
pub fn measure(symbols: &[Symbol], counts: &[SymbolCounts]) -> Result<Vec<Metric>, ModelError> {
    let by_id: HashMap<SymbolId, &Symbol> = symbols.iter().map(|s| (s.id, s)).collect();
    let mut metrics = Vec::new();
    let mut rollups: HashMap<SymbolId, Rollup> = HashMap::new();
    for count in counts {
        let Some(symbol) = by_id.get(&count.symbol) else {
            continue;
        };
        if symbol.kind.is_rollup_target() {
            metrics.push(metric(
                count.symbol,
                MetricKind::Loc,
                f64::from(count.loc.0),
            )?);
        }
        // A callable without a body gets no metrics and takes no part in rollups.
        if let (true, Some(body)) = (symbol.kind.is_callable(), count.body) {
            metrics.extend(callable(symbol, count.loc, body, &by_id, &mut rollups)?);
        }
    }
    for count in counts.iter().filter(|count| count.counted) {
        if by_id
            .get(&count.symbol)
            .is_some_and(|s| s.kind.is_rollup_target())
        {
            let total = rollups.remove(&count.symbol).unwrap_or_default();
            metrics.extend(rolled_up(count.symbol, &total)?);
        }
    }
    Ok(metrics)
}

/// `loc`, `cc` and `mi` of a callable with a body; adds them to every type, namespace and module above it.
fn callable(
    symbol: &Symbol,
    loc: Loc,
    body: BodyCounts,
    by_id: &HashMap<SymbolId, &Symbol>,
    rollups: &mut HashMap<SymbolId, Rollup>,
) -> Result<[Metric; CALLABLE_METRICS], ModelError> {
    let cc = cyclomatic_complexity(body.decisions);
    let mi = maintainability_index(body.halstead, cc, loc)?;
    let mut parent = symbol.parent;
    while let Some(owner) = parent.and_then(|id| by_id.get(&id)) {
        if owner.kind.is_rollup_target() {
            rollups.entry(owner.id).or_default().add(cc, mi.get(), loc);
        }
        parent = owner.parent;
    }
    Ok([
        metric(symbol.id, MetricKind::Loc, f64::from(loc.0))?,
        metric(symbol.id, MetricKind::CyclomaticComplexity, f64::from(cc))?,
        Metric {
            symbol: symbol.id,
            kind: MetricKind::MaintainabilityIndex,
            value: mi,
        },
    ])
}

/// The metrics of a type, namespace or module besides its `loc`. Without callables only `cc` (sum
/// 0) exists; without code lines in the callables there is no `mi`.
fn rolled_up(symbol: SymbolId, total: &Rollup) -> Result<Vec<Metric>, ModelError> {
    let mut metrics = vec![metric(
        symbol,
        MetricKind::CyclomaticComplexity,
        f64::from(total.cc),
    )?];
    if total.callables > 0 {
        metrics.push(metric(
            symbol,
            MetricKind::CyclomaticComplexityMax,
            f64::from(total.cc_max),
        )?);
    }
    if total.weight > 0 {
        let mean = total.weighted_mi / f64::from(total.weight);
        metrics.push(metric(symbol, MetricKind::MaintainabilityIndex, mean)?);
    }
    Ok(metrics)
}
