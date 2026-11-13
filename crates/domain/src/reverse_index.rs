//! The reverse index of references: who uses a symbol, what it uses, what a change reaches.
//!
//! The index holds the references of one snapshot and answers by walking them breadth first from
//! one symbol (blueprint 9.1). It needs no database and runs in time proportional to the part of
//! the graph it visits, which keeps depth 2 far below the 20 ms budget of blueprint 16.1.

use std::collections::{HashMap, HashSet};

use crate::reference::{Reference, ReferenceKind};
use crate::symbol::SymbolId;

/// Which way a walk follows the references.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// To the symbols that use the start symbol: callers, readers, writers, implementers.
    Incoming,
    /// To the symbols the start symbol uses.
    Outgoing,
}

/// One question to the index.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UsageQuery {
    /// Where the walk starts.
    pub symbol: SymbolId,
    /// Which way it goes.
    pub direction: Direction,
    /// How many steps it goes; one lists the direct references only.
    pub depth: u32,
    /// Whether references of the kind `test` count.
    pub include_tests: bool,
}

/// A reference found by a walk and the step it was found in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Usage {
    /// Step of the walk, starting at one.
    pub depth: u32,
    /// The reference.
    pub reference: Reference,
}

/// A symbol that a walk reached for the first time, and the step it was reached in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reached {
    /// The symbol.
    pub symbol: SymbolId,
    /// Step of the walk, starting at one.
    pub depth: u32,
    /// Whether every reference on the way to the symbol is verified by level 2. One unchecked
    /// step makes the whole path unchecked (ADR 0033).
    pub verified: bool,
}

/// The references of one snapshot, indexed by both ends.
#[derive(Debug, Clone, Default)]
pub struct ReverseIndex {
    references: Vec<Reference>,
    /// Whether any reference is verified. If none is, no path is, and a walk skips the check.
    any_verified: bool,
    incoming: HashMap<SymbolId, Vec<usize>>,
    outgoing: HashMap<SymbolId, Vec<usize>>,
}

impl ReverseIndex {
    /// Builds the index. Within one symbol the references are ordered by file, position and id,
    /// so every answer comes out in the same order.
    #[must_use]
    pub fn new(references: impl IntoIterator<Item = Reference>) -> Self {
        let mut all: Vec<Reference> = references.into_iter().collect();
        all.sort_by_key(|reference| (reference.file, reference.span.start(), reference.id));
        let mut incoming: HashMap<SymbolId, Vec<usize>> = HashMap::new();
        let mut outgoing: HashMap<SymbolId, Vec<usize>> = HashMap::new();
        for (position, reference) in all.iter().enumerate() {
            incoming.entry(reference.to).or_default().push(position);
            outgoing.entry(reference.from).or_default().push(position);
        }
        Self {
            any_verified: all.iter().any(|reference| reference.verified),
            references: all,
            incoming,
            outgoing,
        }
    }

    /// Number of references in the index.
    #[must_use]
    pub fn len(&self) -> usize {
        self.references.len()
    }

    /// Whether the index holds no reference.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.references.is_empty()
    }

    /// Number of references that point at `symbol`, the "Usages" column.
    #[must_use]
    pub fn usage_count(&self, symbol: SymbolId) -> usize {
        self.incoming.get(&symbol).map_or(0, Vec::len)
    }

    /// The references a walk finds, step by step: first those of the start symbol, then those of
    /// the symbols reached by the first step, and so on. A reference shows up once. Symbols that
    /// were reached before are not walked again, so cycles end.
    #[must_use]
    pub fn usages(&self, query: &UsageQuery) -> Vec<Usage> {
        let mut found = Vec::new();
        self.walk(query, |usage, _| found.push(usage));
        found
    }

    /// The symbols a walk reaches, nearest first, each once, without the start symbol. With
    /// `Direction::Incoming` this is everything a change of the start symbol can affect. A symbol
    /// counts as verified if one path of verified references reaches it in the nearest step.
    #[must_use]
    pub fn reached(&self, query: &UsageQuery) -> Vec<Reached> {
        let mut found = Vec::new();
        if !self.any_verified {
            self.walk(query, |_, reached| found.extend(reached));
            return found;
        }
        let mut usages = Vec::new();
        self.walk(query, |usage, reached| {
            usages.push(usage);
            found.extend(reached);
        });
        mark_verified(query, &usages, &mut found);
        found
    }

    fn neighbours(
        &self,
        symbol: SymbolId,
        direction: Direction,
    ) -> impl Iterator<Item = &Reference> {
        let side = match direction {
            Direction::Incoming => &self.incoming,
            Direction::Outgoing => &self.outgoing,
        };
        side.get(&symbol)
            .map_or(&[][..], Vec::as_slice)
            .iter()
            .filter_map(|&position| self.references.get(position))
    }

    /// One step of a walk: the references of every symbol in `frontier`. Returns the symbols that
    /// were reached for the first time.
    fn step(
        &self,
        step: Step<'_>,
        frontier: &[SymbolId],
        seen: &mut HashSet<SymbolId>,
        visit: &mut impl FnMut(Usage, Option<Reached>),
    ) -> Vec<SymbolId> {
        let Step { query, depth } = step;
        let mut next = Vec::new();
        let candidates = frontier
            .iter()
            .flat_map(|&symbol| self.neighbours(symbol, query.direction))
            .filter(|reference| query.include_tests || reference.kind != ReferenceKind::Test);
        for &reference in candidates {
            let far = far_end(&reference, query.direction);
            let reached = seen.insert(far).then_some(Reached {
                symbol: far,
                depth,
                verified: false,
            });
            if reached.is_some() {
                next.push(far);
            }
            visit(Usage { depth, reference }, reached);
        }
        next
    }

    fn walk(&self, query: &UsageQuery, mut visit: impl FnMut(Usage, Option<Reached>)) {
        let mut seen = HashSet::from([query.symbol]);
        let mut frontier = vec![query.symbol];
        for depth in 1..=query.depth {
            frontier = self.step(Step { query, depth }, &frontier, &mut seen, &mut visit);
            if frontier.is_empty() {
                return;
            }
        }
    }
}

/// The question and the number of the step a walk is in.
#[derive(Clone, Copy)]
struct Step<'a> {
    query: &'a UsageQuery,
    depth: u32,
}

/// The end of a reference that lies away from the start of the step.
const fn far_end(reference: &Reference, direction: Direction) -> SymbolId {
    match direction {
        Direction::Incoming => reference.from,
        Direction::Outgoing => reference.to,
    }
}

/// Sets `verified` of every reached symbol: true if a path of verified references leads to it
/// from the start symbol in the step that reached it first. `usages` come in the order of the
/// steps, so the symbols of the step before are final when a step is read.
fn mark_verified(query: &UsageQuery, usages: &[Usage], found: &mut [Reached]) {
    let mut verified: HashMap<SymbolId, bool> = HashMap::from([(query.symbol, true)]);
    let depth_of: HashMap<SymbolId, u32> = found.iter().map(|r| (r.symbol, r.depth)).collect();
    for usage in usages {
        let reference = &usage.reference;
        let far = far_end(reference, query.direction);
        if depth_of.get(&far) != Some(&usage.depth) {
            continue;
        }
        let near = near_end(reference, query.direction);
        let path = verified.get(&near).copied().unwrap_or(false) && reference.verified;
        *verified.entry(far).or_insert(false) |= path;
    }
    for reached in found {
        reached.verified = verified.get(&reached.symbol).copied().unwrap_or(false);
    }
}

/// The end of a reference that lies at the start of the step.
const fn near_end(reference: &Reference, direction: Direction) -> SymbolId {
    match direction {
        Direction::Incoming => reference.to,
        Direction::Outgoing => reference.from,
    }
}
