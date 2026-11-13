//! The counts behind the metrics of one file (ADR 0015): lines of code per symbol, and per callable
//! the decision points and the Halstead counts that `metrics.scm` captures.

use std::collections::{HashMap, HashSet};
use std::ops::RangeInclusive;

use codeskimr_domain::{
    BodyCounts, HalsteadCounts, LineKind, Loc, Span, Symbol, SymbolCounts, SymbolId,
};
use tree_sitter::{Node, QueryCursor, StreamingIterator};

use crate::captures::{MetricRole as Role, MetricsQuery};
use crate::location::text_of;
use crate::symbols::SymbolTable;

/// Decision points and token texts seen in one callable.
#[derive(Default)]
struct Tally {
    decisions: u32,
    operators: HashMap<String, u32>,
    operands: HashMap<String, u32>,
}

impl Tally {
    fn counts(&self) -> BodyCounts {
        BodyCounts {
            decisions: self.decisions,
            halstead: HalsteadCounts {
                distinct_operators: size(self.operators.len()),
                distinct_operands: size(self.operands.len()),
                total_operators: self.operators.values().sum(),
                total_operands: self.operands.values().sum(),
            },
        }
    }
}

fn size(count: usize) -> u32 {
    u32::try_from(count).unwrap_or(u32::MAX)
}

/// Runs the query once and assigns every capture to the innermost callable that contains it. A
/// lambda is no symbol, so its captures count for the enclosing callable; a named local function
/// is its own symbol. Captures outside any callable (field initialisers) count for nobody.
fn tally(
    query: &MetricsQuery,
    root: Node<'_>,
    source: &str,
    table: &SymbolTable,
) -> HashMap<SymbolId, Tally> {
    let mut tallies: HashMap<SymbolId, Tally> = HashMap::new();
    let mut seen: HashSet<(Role, usize)> = HashSet::new();
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query.query, root, source.as_bytes());
    while let Some(found) = matches.next() {
        for capture in found.captures() {
            let role = usize::try_from(capture.index)
                .ok()
                .and_then(|index| query.roles.get(index));
            let (Some(&role), Some(owner)) = (role, table.innermost(capture.node.start_byte()))
            else {
                continue;
            };
            // Overlapping patterns may capture a node twice; it counts once.
            if !owner.kind.is_callable() || !seen.insert((role, capture.node.id())) {
                continue;
            }
            let entry = tallies.entry(owner.id).or_default();
            let text = text_of(capture.node, source).to_owned();
            match role {
                Role::Decision => entry.decisions += 1,
                Role::Operator => *entry.operators.entry(text).or_default() += 1,
                Role::Operand => *entry.operands.entry(text).or_default() += 1,
                Role::Comment => {}
            }
        }
    }
    tallies
}

/// The rows a span touches; a span that ends at column zero does not touch its last row.
fn rows(span: &Span) -> RangeInclusive<usize> {
    let (start, end) = (span.start(), span.end());
    let last = if end.column == 0 && end.line > start.line {
        end.line - 1
    } else {
        end.line
    };
    to_row(start.line)..=to_row(last)
}

fn to_row(line: u32) -> usize {
    usize::try_from(line).unwrap_or(usize::MAX)
}

/// Code lines of `rows`, without those in `except`.
fn code_lines(
    lines: &[LineKind],
    rows: RangeInclusive<usize>,
    except: &[RangeInclusive<usize>],
) -> Loc {
    let count = rows
        .filter(|row| !except.iter().any(|skip| skip.contains(row)))
        .filter(|row| lines.get(*row) == Some(&LineKind::Code))
        .count();
    Loc(size(count))
}

/// Counts for every callable, type, namespace and module of the file. `lines` are the line kinds
/// of the whole file. The loc of a callable leaves out its named local functions and local types.
/// A pack without `metrics.scm` delivers lines only.
#[must_use]
pub fn counts_of(
    query: Option<&MetricsQuery>,
    root: Node<'_>,
    source: &str,
    table: &SymbolTable,
    lines: &[LineKind],
) -> Vec<SymbolCounts> {
    let tallies = query.map(|query| tally(query, root, source, table));
    let local = locals(&table.symbols);
    table
        .symbols
        .iter()
        .filter(|symbol| symbol.kind.is_callable() || symbol.kind.is_rollup_target())
        .map(|symbol| {
            let except = if symbol.kind.is_callable() {
                local.get(&symbol.id).map_or(&[][..], Vec::as_slice)
            } else {
                &[]
            };
            SymbolCounts {
                symbol: symbol.id,
                loc: code_lines(lines, rows(&symbol.span), except),
                body: body_of(symbol, table, tallies.as_ref()),
                counted: tallies.is_some(),
            }
        })
        .collect()
}

fn body_of(
    symbol: &Symbol,
    table: &SymbolTable,
    tallies: Option<&HashMap<SymbolId, Tally>>,
) -> Option<BodyCounts> {
    if !symbol.kind.is_callable() || !table.has_body(symbol.id) {
        return None;
    }
    tallies.map(|all| {
        all.get(&symbol.id)
            .map_or_else(BodyCounts::default, Tally::counts)
    })
}

/// The rows of the callables and types declared directly inside each callable.
fn locals(symbols: &[Symbol]) -> HashMap<SymbolId, Vec<RangeInclusive<usize>>> {
    let callables: HashSet<SymbolId> = symbols
        .iter()
        .filter(|symbol| symbol.kind.is_callable())
        .map(|symbol| symbol.id)
        .collect();
    let mut result: HashMap<SymbolId, Vec<RangeInclusive<usize>>> = HashMap::new();
    for symbol in symbols {
        let owner = symbol.parent.filter(|parent| callables.contains(parent));
        if let (Some(owner), true) = (owner, symbol.kind.is_callable() || symbol.kind.is_type()) {
            result.entry(owner).or_default().push(rows(&symbol.span));
        }
    }
    result
}
