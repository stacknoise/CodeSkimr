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
    /// step makes the whole path unchecked (ADR 0032).
    pub verified: bool,
}

/// The references of one snapshot, indexed by both ends.
#[derive(Debug, Clone, Default)]
pub struct ReverseIndex {
    references: Vec<Reference>,
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
        self.walk(query).0
    }

    /// The symbols a walk reaches, nearest first, each once, without the start symbol. With
    /// `Direction::Incoming` this is everything a change of the start symbol can affect. A symbol
    /// counts as verified if one path of verified references reaches it in the nearest step.
    #[must_use]
    pub fn reached(&self, query: &UsageQuery) -> Vec<Reached> {
        self.walk(query).1
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

    /// One step of a walk: the references of every symbol in `frontier`, which carries whether
    /// the path to it is verified. Returns the symbols that were reached for the first time.
    fn step(
        &self,
        step: Step<'_>,
        frontier: &[(SymbolId, bool)],
        seen: &mut HashSet<SymbolId>,
        usages: &mut Vec<Usage>,
    ) -> Vec<Reached> {
        let Step { query, depth } = step;
        let mut reached: Vec<Reached> = Vec::new();
        let mut at: HashMap<SymbolId, usize> = HashMap::new();
        for &(symbol, path_verified) in frontier {
            let candidates = self
                .neighbours(symbol, query.direction)
                .filter(|reference| query.include_tests || reference.kind != ReferenceKind::Test);
            for &reference in candidates {
                let far = far_end(&reference, query.direction);
                let verified = path_verified && reference.verified;
                if seen.contains(&far)
                    && let Some(&position) = at.get(&far)
                    && let Some(entry) = reached.get_mut(position)
                {
                    entry.verified |= verified;
                } else if seen.insert(far) {
                    at.insert(far, reached.len());
                    reached.push(Reached {
                        symbol: far,
                        depth,
                        verified,
                    });
                }
                usages.push(Usage { depth, reference });
            }
        }
        reached
    }

    fn walk(&self, query: &UsageQuery) -> (Vec<Usage>, Vec<Reached>) {
        let mut seen = HashSet::from([query.symbol]);
        let mut frontier = vec![(query.symbol, true)];
        let (mut usages, mut found) = (Vec::new(), Vec::new());
        for depth in 1..=query.depth {
            let reached = self.step(Step { query, depth }, &frontier, &mut seen, &mut usages);
            frontier = reached.iter().map(|r| (r.symbol, r.verified)).collect();
            found.extend(reached);
            if frontier.is_empty() {
                break;
            }
        }
        (usages, found)
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
