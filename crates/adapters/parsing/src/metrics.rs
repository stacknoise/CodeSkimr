//! The counts behind the metrics of one file (ADR 0015): lines of code per symbol, and per callable
//! the decision points and the Halstead counts that `metrics.scm` captures.

use std::collections::{HashMap, HashSet};
use std::ops::{Range, RangeInclusive};

use codeskimr_domain::{
    BodyCounts, HalsteadCounts, LineKind, Loc, Span, Symbol, SymbolCounts, SymbolId,
};
use tree_sitter::{Node, QueryCursor, StreamingIterator};

use crate::captures::{MetricRole as Role, MetricsQuery};
use crate::location::text_of;
use crate::symbols::SymbolTable;

/// Decision points, token texts and cognitive parts seen in one callable.
#[derive(Default)]
struct Tally<'t> {
    decisions: u32,
    operators: HashMap<String, u32>,
    operands: HashMap<String, u32>,
    /// Structures that count with their nesting.
    structures: Vec<Node<'t>>,
    /// Parts that count one each, whatever the nesting.
    flat: u32,
    /// The boolean operators: the operation, the position and the text of each.
    operators_seen: Vec<(Node<'t>, usize, String)>,
    parameters: u32,
}

/// What the pack of the file counts besides decisions and Halstead (ADR 0048).
#[derive(Clone, Copy)]
struct Support {
    cognitive: bool,
    parameters: bool,
}

impl Tally<'_> {
    fn counts(&self, support: Support, cognitive: u32) -> BodyCounts {
        BodyCounts {
            decisions: self.decisions,
            halstead: HalsteadCounts {
                distinct_operators: size(self.operators.len()),
                distinct_operands: size(self.operands.len()),
                total_operators: self.operators.values().sum(),
                total_operands: self.operands.values().sum(),
            },
            cognitive: support.cognitive.then_some(cognitive),
            parameters: support.parameters.then_some(self.parameters),
        }
    }
}

/// What one structure adds before its nesting is counted.
const NODE_COST: u32 = 1;

fn size(count: usize) -> u32 {
    u32::try_from(count).unwrap_or(u32::MAX)
}

/// What one run of the query found in a file.
#[derive(Default)]
struct Found<'t> {
    tallies: HashMap<SymbolId, Tally<'t>>,
    /// Nodes that nest what is inside them, by node id.
    nesting: HashSet<usize>,
    /// Nodes that continue the chain around them and neither count nor nest, by node id.
    continuations: HashSet<usize>,
}

/// Runs the query once and assigns every capture to the innermost callable that contains it. A
/// lambda is no symbol, so its captures count for the enclosing callable; a named local function
/// is its own symbol. Captures outside any callable (field initialisers) count for nobody.
fn tally<'t>(query: &MetricsQuery, root: Node<'t>, source: &str, table: &SymbolTable) -> Found<'t> {
    let mut found = Found::default();
    let mut seen: HashSet<(Role, usize)> = HashSet::new();
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query.query, root, source.as_bytes());
    while let Some(matched) = matches.next() {
        for capture in matched.captures() {
            let role = usize::try_from(capture.index)
                .ok()
                .and_then(|index| query.roles.get(index));
            let (Some(&role), Some(owner)) = (role, table.innermost(capture.node.start_byte()))
            else {
                continue;
            };
            // Overlapping patterns may capture a node twice; it counts once.
            if !seen.insert((role, capture.node.id())) {
                continue;
            }
            match role {
                Role::Nesting => drop(found.nesting.insert(capture.node.id())),
                Role::Continuation => drop(found.continuations.insert(capture.node.id())),
                _ => {}
            }
            if owner.kind.is_callable() {
                record(&mut found, owner.id, role, capture.node, source);
            }
        }
    }
    found
}

fn record<'t>(found: &mut Found<'t>, owner: SymbolId, role: Role, node: Node<'t>, source: &str) {
    let entry = found.tallies.entry(owner).or_default();
    match role {
        Role::Decision => entry.decisions += 1,
        Role::Operator => {
            *entry
                .operators
                .entry(text_of(node, source).to_owned())
                .or_default() += 1;
        }
        Role::Operand => {
            *entry
                .operands
                .entry(text_of(node, source).to_owned())
                .or_default() += 1;
        }
        Role::Cognitive => entry.structures.push(node),
        Role::CognitiveFlat => entry.flat += 1,
        Role::CognitiveSequence => {
            if let Some(operation) = node.parent() {
                let text = text_of(node, source).to_owned();
                entry
                    .operators_seen
                    .push((operation, node.start_byte(), text));
            }
        }
        Role::Parameter => entry.parameters += 1,
        Role::Comment | Role::Nesting | Role::Continuation => {}
    }
}

/// The boolean operations of one callable counted as runs: the operators that follow each other in
/// one expression count one, and one more at every change of the operator. An operation that is
/// the operand of another (`a && b || c`) belongs to its expression; a parenthesis or a call
/// starts a new one.
fn runs(seen: &[(Node<'_>, usize, String)]) -> u32 {
    let operations: HashSet<usize> = seen.iter().map(|(operation, ..)| operation.id()).collect();
    let mut by_expression: HashMap<usize, Vec<(usize, &str)>> = HashMap::new();
    for (operation, position, text) in seen {
        let mut root = *operation;
        while let Some(parent) = root
            .parent()
            .filter(|parent| operations.contains(&parent.id()))
        {
            root = parent;
        }
        by_expression
            .entry(root.id())
            .or_default()
            .push((*position, text.as_str()));
    }
    by_expression
        .into_values()
        .map(|mut operators| {
            operators.sort_unstable();
            let changes = operators
                .iter()
                .zip(operators.iter().skip(1))
                .filter(|(before, after)| before.1 != after.1)
                .count();
            size(changes + 1)
        })
        .sum()
}

/// How many nesting nodes lie around `node` inside the byte range of its callable.
fn nesting_depth(
    node: Node<'_>,
    owner: &Range<usize>,
    nesting: &HashSet<usize>,
    continuations: &HashSet<usize>,
) -> u32 {
    let mut depth = 0;
    let mut current = node.parent();
    while let Some(parent) = current {
        if parent.start_byte() < owner.start || parent.end_byte() > owner.end {
            break;
        }
        let nests = nesting.contains(&parent.id()) && !continuations.contains(&parent.id());
        depth += u32::from(nests);
        current = parent.parent();
    }
    depth
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
    let measured = query.map(|query| Measured::new(query, root, source, table));
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
                body: body_of(symbol, table, measured.as_ref()),
                counted: measured.is_some(),
            }
        })
        .collect()
}

/// The query's findings in one file, with what is needed to total them per callable.
struct Measured<'t> {
    found: Found<'t>,
    support: Support,
    ranges: HashMap<SymbolId, Range<usize>>,
}

impl<'t> Measured<'t> {
    fn new(query: &MetricsQuery, root: Node<'t>, source: &str, table: &SymbolTable) -> Self {
        Self {
            found: tally(query, root, source, table),
            support: Support {
                cognitive: query.counts_cognitive(),
                parameters: query.counts_parameters(),
            },
            ranges: table
                .callables()
                .map(|(symbol, range)| (symbol.id, range.clone()))
                .collect(),
        }
    }

    /// Cognitive complexity of a callable: one for every structure and for every level it is
    /// nested at, plus the parts that count one each.
    fn cognitive(&self, id: SymbolId, tally: &Tally<'_>) -> u32 {
        let Some(range) = self.ranges.get(&id) else {
            return tally.flat + runs(&tally.operators_seen);
        };
        let flat = tally.flat + runs(&tally.operators_seen);
        let counted = tally
            .structures
            .iter()
            .filter(|node| !self.found.continuations.contains(&node.id()));
        counted.fold(flat, |total, node| {
            total + NODE_COST + self.depth(*node, range)
        })
    }

    /// The nesting nodes around `node`, leaving out those that only continue a chain.
    fn depth(&self, node: Node<'_>, range: &Range<usize>) -> u32 {
        nesting_depth(node, range, &self.found.nesting, &self.found.continuations)
    }

    fn body(&self, id: SymbolId) -> BodyCounts {
        let empty = Tally::default();
        let tally = self.found.tallies.get(&id).unwrap_or(&empty);
        tally.counts(self.support, self.cognitive(id, tally))
    }
}

fn body_of(
    symbol: &Symbol,
    table: &SymbolTable,
    measured: Option<&Measured<'_>>,
) -> Option<BodyCounts> {
    if !symbol.kind.is_callable() || !table.has_body(symbol.id) {
        return None;
    }
    measured.map(|measured| measured.body(symbol.id))
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
