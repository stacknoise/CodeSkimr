//! Coupling between namespaces and modules: efferent coupling (Ce), afferent coupling (Ca) and
//! instability (ADR 0015, ADR 0037).
//!
//! A symbol belongs to the nearest enclosing namespace or module, found by its qualified name like
//! in the namespace graph; a namespace or module belongs to itself. `ce` counts the other units
//! that a unit refers to, `ca` the other units that refer to it, each unit once however many
//! references there are. Instability is `ce / (ca + ce)` and does not exist for a unit that has no
//! dependency in either direction.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::error::ModelError;
use crate::metric::{Metric, MetricKind, MetricValue};
use crate::namespace_graph::enclosing;
use crate::reference::{Reference, ReferenceKind};
use crate::symbol::{Symbol, SymbolId, SymbolKind};

/// The symbols that declare each unit; a namespace that several files declare has several.
type Units<'a> = HashMap<&'a str, Vec<SymbolId>>;
/// The unit of every symbol that sits in one.
type Owners<'a> = HashMap<SymbolId, &'a str>;
/// The other units on one side of the dependencies of every unit.
type Links<'a> = BTreeMap<&'a str, BTreeSet<&'a str>>;

fn units_of(symbols: &[Symbol]) -> Units<'_> {
    let mut units = Units::new();
    for symbol in symbols {
        if matches!(symbol.kind, SymbolKind::Namespace | SymbolKind::Module) {
            units
                .entry(symbol.qualified_name.as_str())
                .or_default()
                .push(symbol.id);
        }
    }
    units
}

fn owners<'a>(symbols: &'a [Symbol], units: &Units<'a>) -> Owners<'a> {
    let mut owner = Owners::new();
    for symbol in symbols {
        let name = symbol.qualified_name.as_str();
        let own = matches!(symbol.kind, SymbolKind::Namespace | SymbolKind::Module)
            .then(|| units.get_key_value(name).map(|(key, _)| *key))
            .flatten();
        if let Some(unit) = own.or_else(|| enclosing(units, name)) {
            owner.insert(symbol.id, unit);
        }
    }
    owner
}

/// The units each unit refers to and the units that refer to it. References from test code do not
/// count, so that a test namespace does not make the code it checks look depended upon.
fn dependencies<'a>(owner: &Owners<'a>, references: &[Reference]) -> (Links<'a>, Links<'a>) {
    let mut efferent = Links::new();
    let mut afferent = Links::new();
    for reference in references {
        if reference.kind == ReferenceKind::Test {
            continue;
        }
        if let (Some(&from), Some(&to)) = (owner.get(&reference.from), owner.get(&reference.to))
            && from != to
        {
            efferent.entry(from).or_default().insert(to);
            afferent.entry(to).or_default().insert(from);
        }
    }
    (efferent, afferent)
}

fn count(links: &Links<'_>, unit: &str) -> usize {
    links.get(unit).map_or(0, BTreeSet::len)
}

fn number(count: usize) -> f64 {
    f64::from(u32::try_from(count).unwrap_or(u32::MAX))
}

fn metric(symbol: SymbolId, kind: MetricKind, value: f64) -> Result<Metric, ModelError> {
    Ok(Metric {
        symbol,
        kind,
        value: MetricValue::new(value)?,
    })
}

/// `ce`, `ca` and, where there is a dependency, `instability` of every namespace and module of
/// `symbols`, from the references between them. Pass the symbols and references of the whole
/// snapshot: the values depend on files other than the unit's own.
///
/// # Errors
/// [`ModelError::NotFinite`] if a value overflows, which counts of `u32` rule out in practice.
pub fn coupling(symbols: &[Symbol], references: &[Reference]) -> Result<Vec<Metric>, ModelError> {
    let units = units_of(symbols);
    let owner = owners(symbols, &units);
    let (efferent, afferent) = dependencies(&owner, references);
    let mut metrics = Vec::new();
    for (unit, declared) in &units {
        let ce = count(&efferent, unit);
        let ca = count(&afferent, unit);
        for &symbol in declared {
            metrics.push(metric(symbol, MetricKind::EfferentCoupling, number(ce))?);
            metrics.push(metric(symbol, MetricKind::AfferentCoupling, number(ca))?);
            if ce + ca > 0 {
                let ratio = number(ce) / number(ce + ca);
                metrics.push(metric(symbol, MetricKind::Instability, ratio)?);
            }
        }
    }
    metrics.sort_by_key(|found| found.symbol);
    Ok(metrics)
}
