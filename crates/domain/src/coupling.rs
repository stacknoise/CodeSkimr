//! Coupling between namespaces and modules: efferent coupling (Ce), afferent coupling (Ca) and
//! instability (ADR 0015, ADR 0037).
//!
//! A symbol belongs to the nearest enclosing namespace or module, found by its qualified name like
//! in the namespace graph; a namespace or module belongs to itself. `ce` counts the other units
//! that a unit refers to, `ca` the other units that refer to it, each unit once however many
//! references there are. Instability is `ce / (ca + ce)` and does not exist for a unit that has no
//! dependency in either direction.

use std::collections::{HashMap, HashSet};

use crate::consts::{MODULE_NAME_SEPARATOR, NAME_SEPARATOR};
use crate::error::ModelError;
use crate::id_hasher::IdHash;
use crate::metric::{Metric, MetricKind, MetricValue};
use crate::reference::{Reference, ReferenceKind};
use crate::symbol::{Symbol, SymbolId, SymbolKind};

/// The symbols that declare each unit, by name; a namespace that several files declare has several.
type Declared<'a> = HashMap<&'a str, Vec<SymbolId>>;
/// The position of the unit of every symbol that sits in one.
type Owners = HashMap<SymbolId, usize, IdHash>;

fn is_unit(kind: SymbolKind) -> bool {
    matches!(kind, SymbolKind::Namespace | SymbolKind::Module)
}

fn declared_units(symbols: &[Symbol]) -> Declared<'_> {
    let mut units = Declared::new();
    for symbol in symbols.iter().filter(|symbol| is_unit(symbol.kind)) {
        units
            .entry(symbol.qualified_name.as_str())
            .or_default()
            .push(symbol.id);
    }
    units
}

/// The name without its last segment, `None` for a name of one segment.
fn parent_name(name: &str) -> Option<&str> {
    let dotted = name.rfind(NAME_SEPARATOR);
    let pathed = name.rfind(MODULE_NAME_SEPARATOR);
    dotted.max(pathed).and_then(|cut| name.get(..cut))
}

/// The unit that is `name` or the nearest one among its parents. The answer for every name on the
/// way is remembered, so the members of one type share the lookup.
fn unit_at<'a>(
    name: &'a str,
    position: &HashMap<&str, usize>,
    known: &mut HashMap<&'a str, Option<usize>>,
) -> Option<usize> {
    if let Some(found) = known.get(name) {
        return *found;
    }
    let found = position
        .get(name)
        .copied()
        .or_else(|| parent_name(name).and_then(|parent| unit_at(parent, position, known)));
    known.insert(name, found);
    found
}

/// The unit of every symbol: a unit is its own, any other symbol sits in the nearest enclosing one.
fn owners(symbols: &[Symbol], position: &HashMap<&str, usize>) -> Owners {
    let mut owner = Owners::default();
    let mut known = HashMap::new();
    for symbol in symbols {
        let name = symbol.qualified_name.as_str();
        let start = if is_unit(symbol.kind) {
            Some(name)
        } else {
            parent_name(name)
        };
        if let Some(found) = start.and_then(|start| unit_at(start, position, &mut known)) {
            owner.insert(symbol.id, found);
        }
    }
    owner
}

/// How many other units each unit refers to and how many refer to it, by position. References from
/// test code do not count, so that a test namespace does not make the code it checks look depended
/// upon.
fn dependencies(
    owner: &Owners,
    references: &[Reference],
    units: usize,
) -> (Vec<usize>, Vec<usize>) {
    let mut pairs: HashSet<(usize, usize), IdHash> = HashSet::default();
    let mut last = None;
    for reference in references {
        if reference.kind == ReferenceKind::Test {
            continue;
        }
        if let (Some(&from), Some(&to)) = (owner.get(&reference.from), owner.get(&reference.to))
            && from != to
            && last != Some((from, to))
        {
            pairs.insert((from, to));
            last = Some((from, to));
        }
    }
    let mut efferent = vec![0; units];
    let mut afferent = vec![0; units];
    for (from, to) in pairs {
        bump(&mut efferent, from);
        bump(&mut afferent, to);
    }
    (efferent, afferent)
}

fn bump(counts: &mut [usize], at: usize) {
    if let Some(count) = counts.get_mut(at) {
        *count += 1;
    }
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
    let units = declared_units(symbols);
    let position: HashMap<&str, usize> = units
        .keys()
        .enumerate()
        .map(|(index, name)| (*name, index))
        .collect();
    let owner = owners(symbols, &position);
    let (efferent, afferent) = dependencies(&owner, references, units.len());
    let mut metrics = Vec::new();
    for (unit, declared) in &units {
        let at = position.get(unit).copied();
        let ce = at.and_then(|at| efferent.get(at)).copied().unwrap_or(0);
        let ca = at.and_then(|at| afferent.get(at)).copied().unwrap_or(0);
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
