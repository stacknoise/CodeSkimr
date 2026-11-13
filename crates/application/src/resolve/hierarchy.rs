//! Which type derives from which, as far as the resolved base lists tell.

use std::collections::{HashMap, HashSet};

use codeskimr_domain::SymbolId;

/// Direct base types and direct subtypes of every type that has an inheritance reference.
#[derive(Debug, Default)]
pub struct Hierarchy {
    bases: HashMap<SymbolId, Vec<SymbolId>>,
    derived: HashMap<SymbolId, Vec<SymbolId>>,
}

impl Hierarchy {
    /// Records that `sub` has `base` in its base list.
    pub fn add(&mut self, sub: SymbolId, base: SymbolId) {
        let bases = self.bases.entry(sub).or_default();
        if !bases.contains(&base) {
            bases.push(base);
            self.derived.entry(base).or_default().push(sub);
        }
    }

    /// The types in the base list of `id` itself.
    pub fn direct_bases_of(&self, id: SymbolId) -> Vec<SymbolId> {
        self.bases.get(&id).cloned().unwrap_or_default()
    }

    /// All base types of `id`, nearest first, without `id` itself.
    pub fn bases_of(&self, id: SymbolId) -> Vec<SymbolId> {
        reachable(id, &self.bases)
    }

    /// All types that derive from `id`, directly or not.
    pub fn descendants_of(&self, id: SymbolId) -> Vec<SymbolId> {
        reachable(id, &self.derived)
    }

    /// `id` followed by all its base types.
    pub fn closure(&self, id: SymbolId) -> Vec<SymbolId> {
        let mut all = vec![id];
        all.extend(self.bases_of(id));
        all
    }
}

/// Breadth-first walk along `edges`, each node once, the start excluded.
fn reachable(start: SymbolId, edges: &HashMap<SymbolId, Vec<SymbolId>>) -> Vec<SymbolId> {
    let mut seen = HashSet::from([start]);
    let mut order = Vec::new();
    let mut next = vec![start];
    while !next.is_empty() {
        let mut following = Vec::new();
        for node in next {
            for &target in edges.get(&node).map_or(&[][..], Vec::as_slice) {
                if seen.insert(target) {
                    order.push(target);
                    following.push(target);
                }
            }
        }
        next = following;
    }
    order
}
