//! The comparison of the symbols of two snapshots of one project (ADR 0097).
//!
//! Pure logic on facts that a store hands over: no database, no snapshot ids. Two symbols are the
//! same symbol when kind, qualified name and parameter types are equal (the key of ADR 0068).
//! The result tells which symbols were *added*, *removed*, *changed* (signature, visibility or
//! content) or *moved* (the same key in another file). A renamed symbol has another key, so it is
//! removed and added; the comparison never guesses.

use std::collections::{HashMap, HashSet};

use crate::file::{ContentHash, RelativePath};
use crate::qualified_name::QualifiedName;
use crate::signature::signature_parameters;
use crate::symbol::{SymbolId, SymbolKind, Visibility};

/// What the comparison knows of one symbol of one snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolFact {
    /// Identity within its snapshot.
    pub id: SymbolId,
    /// Kind.
    pub kind: SymbolKind,
    /// Fully qualified name.
    pub qualified_name: QualifiedName,
    /// Signature as written, if any; the parameter types are taken from it.
    pub signature: Option<String>,
    /// Visibility, `None` where the language has no such concept or it is unknown.
    pub visibility: Option<Visibility>,
    /// The file that declares it.
    pub path: RelativePath,
    /// Hash over its tokens; `None` for a symbol that has none (level 2 symbols, for instance).
    pub content_hash: Option<ContentHash>,
}

/// Whether symbols of this kind take part in a comparison: types, callables, properties, fields,
/// constants, events and enum members (ADR 0097, decision 2).
#[must_use]
pub const fn compares_kind(kind: SymbolKind) -> bool {
    kind.is_type()
        || kind.is_callable()
        || matches!(
            kind,
            SymbolKind::Property
                | SymbolKind::Field
                | SymbolKind::Constant
                | SymbolKind::Event
                | SymbolKind::EnumMember
        )
}

/// The key that names a symbol across snapshots: kind, qualified name and parameter types.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SymbolIdentity {
    /// Kind.
    pub kind: SymbolKind,
    /// Fully qualified name.
    pub name: QualifiedName,
    /// The parameter types as written, in order; tells overloads apart.
    pub parameters: Vec<String>,
}

impl SymbolFact {
    /// The key of the symbol.
    #[must_use]
    pub fn identity(&self) -> SymbolIdentity {
        SymbolIdentity {
            kind: self.kind,
            name: self.qualified_name.clone(),
            parameters: self
                .signature
                .as_deref()
                .map(signature_parameters)
                .unwrap_or_default(),
        }
    }

    /// Whether the symbol takes part in a comparison: a type, a callable, a property, a field, a
    /// constant, an event or an enum member. Namespaces, modules, parameters and type aliases do
    /// not (ADR 0097, decision 2).
    #[must_use]
    pub const fn takes_part(&self) -> bool {
        compares_kind(self.kind)
    }

    /// The place of the symbol in the output: file, name, then id.
    fn place(&self) -> (&str, &str, SymbolId) {
        (self.path.as_str(), self.qualified_name.as_str(), self.id)
    }
}

/// Why a symbol counts as changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeReason {
    /// The signature text differs.
    Signature,
    /// The visibility differs; `narrowed` is set when the symbol is less visible than before.
    Visibility {
        /// Whether the new visibility is narrower than the old one.
        narrowed: bool,
    },
    /// Only the content hash differs: the signature and the visibility are the same.
    Content,
}

impl ChangeReason {
    /// Whether usages of the symbol can break by this reason: a signature or a visibility
    /// change does, a change of the body does not.
    #[must_use]
    pub const fn can_break_usages(self) -> bool {
        !matches!(self, Self::Content)
    }
}

/// One symbol that exists in both snapshots, with what differs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Matched {
    /// The symbol in the base snapshot.
    pub base: SymbolFact,
    /// The symbol in the target snapshot.
    pub target: SymbolFact,
    /// What differs; empty for a symbol that only moved.
    pub reasons: Vec<ChangeReason>,
}

/// Why the usages of a base symbol are looked at (ADR 0097, decision 4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImpactCause {
    /// The symbol is gone.
    Removed,
    /// The signature or the visibility changed.
    Changed(Vec<ChangeReason>),
}

/// A symbol of the base snapshot whose usages a change can break.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImpactSubject {
    /// The symbol as the base snapshot has it.
    pub symbol: SymbolFact,
    /// Why its usages are looked at.
    pub cause: ImpactCause,
}

/// The difference between two snapshots, every list in the order of file, name and id.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Comparison {
    /// Symbols only the target has.
    pub added: Vec<SymbolFact>,
    /// Symbols only the base has.
    pub removed: Vec<SymbolFact>,
    /// Symbols of both that differ in place, with the reasons.
    pub changed: Vec<Matched>,
    /// Symbols with the same key in another file.
    pub moved: Vec<Matched>,
}

impl Comparison {
    /// The symbols of the base snapshot that the comparison changed, moved or removed: a usage
    /// that sits in one of them counts as adapted.
    #[must_use]
    pub fn touched_in_base(&self) -> HashSet<SymbolId> {
        let removed = self.removed.iter().map(|fact| fact.id);
        let changed = self.changed.iter().map(|matched| matched.base.id);
        let moved = self.moved.iter().map(|moved| moved.base.id);
        removed.chain(changed).chain(moved).collect()
    }

    /// The symbols whose usages a change can break: the removed ones and those
    /// that changed in signature or visibility (also when they moved). Content-only changes and
    /// added symbols have none.
    #[must_use]
    pub fn impact_subjects(&self) -> Vec<ImpactSubject> {
        let removed = self.removed.iter().map(|fact| ImpactSubject {
            symbol: fact.clone(),
            cause: ImpactCause::Removed,
        });
        let changed = self.changed.iter().filter_map(breaking);
        let moved = self.moved.iter().filter_map(breaking);
        removed.chain(changed).chain(moved).collect()
    }

    /// Whether the two snapshots have the same symbols in the same state.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.added.is_empty()
            && self.removed.is_empty()
            && self.changed.is_empty()
            && self.moved.is_empty()
    }

    fn sort(&mut self) {
        self.added.sort_by(|a, b| a.place().cmp(&b.place()));
        self.removed.sort_by(|a, b| a.place().cmp(&b.place()));
        self.changed
            .sort_by(|a, b| a.target.place().cmp(&b.target.place()));
        self.moved
            .sort_by(|a, b| a.target.place().cmp(&b.target.place()));
    }
}

/// The subject of a pair whose signature or visibility changed, if it did.
fn breaking(matched: &Matched) -> Option<ImpactSubject> {
    let reasons: Vec<ChangeReason> = matched
        .reasons
        .iter()
        .copied()
        .filter(|reason| reason.can_break_usages())
        .collect();
    (!reasons.is_empty()).then(|| ImpactSubject {
        symbol: matched.base.clone(),
        cause: ImpactCause::Changed(reasons),
    })
}

/// Whether `to` shows the symbol to fewer than `from` did; the order of [`Visibility::ALL`] runs
/// from the most to the least visible one. Unknown visibilities are not compared.
fn narrows(from: Option<Visibility>, to: Option<Visibility>) -> bool {
    let rank = |visibility: Visibility| Visibility::ALL.iter().position(|v| *v == visibility);
    match (from.and_then(rank), to.and_then(rank)) {
        (Some(before), Some(after)) => after > before,
        _ => false,
    }
}

/// What differs between the two sides of a pair.
fn reasons_of(base: &SymbolFact, target: &SymbolFact) -> Vec<ChangeReason> {
    let mut reasons = Vec::new();
    if base.signature != target.signature {
        reasons.push(ChangeReason::Signature);
    }
    if base.visibility != target.visibility {
        reasons.push(ChangeReason::Visibility {
            narrowed: narrows(base.visibility, target.visibility),
        });
    }
    let both_hashed = base.content_hash.is_some() && target.content_hash.is_some();
    if reasons.is_empty() && both_hashed && base.content_hash != target.content_hash {
        reasons.push(ChangeReason::Content);
    }
    reasons
}

type Group<'a> = Vec<&'a SymbolFact>;

/// The participating symbols by key, each group in the order of the ids.
fn group_by_identity(facts: &[SymbolFact]) -> HashMap<SymbolIdentity, Group<'_>> {
    let mut groups: HashMap<SymbolIdentity, Group<'_>> = HashMap::new();
    for fact in facts.iter().filter(|fact| fact.takes_part()) {
        groups.entry(fact.identity()).or_default().push(fact);
    }
    for group in groups.values_mut() {
        group.sort_by_key(|fact| fact.id);
    }
    groups
}

/// The symbols of the groups that no key pairs, and the pairs.
#[derive(Default)]
struct Paired<'a> {
    pairs: Vec<(&'a SymbolFact, &'a SymbolFact)>,
    removed: Group<'a>,
    added: Group<'a>,
}

/// Pairs the symbols of one key: first those in the same file, in the order of the ids, then the
/// two that are left if there is exactly one on each side (it moved). More than that is not
/// guessed: the rest stays removed and added.
fn pair_group<'a>(bases: Group<'a>, mut targets: Group<'a>) -> Paired<'a> {
    let mut paired = Paired::default();
    let mut open = Vec::new();
    for base in bases {
        match targets.iter().position(|target| target.path == base.path) {
            Some(at) => paired.pairs.push((base, targets.remove(at))),
            None => open.push(base),
        }
    }
    match (open.pop(), targets.pop()) {
        (Some(base), Some(target)) if open.is_empty() && targets.is_empty() => {
            paired.pairs.push((base, target));
        }
        (base, target) => {
            paired.removed.extend(open.into_iter().chain(base));
            paired.added.extend(targets.into_iter().chain(target));
        }
    }
    paired
}

/// Compares the symbols of a base snapshot with those of a target snapshot.
///
/// Only the symbols that [`SymbolFact::takes_part`] are looked at. The result lists every
/// symbol once: a moved symbol is not also removed and added.
#[must_use]
pub fn compare_symbols(base: &[SymbolFact], target: &[SymbolFact]) -> Comparison {
    let mut targets = group_by_identity(target);
    let mut paired = Paired::default();
    for (key, bases) in group_by_identity(base) {
        let found = pair_group(bases, targets.remove(&key).unwrap_or_default());
        paired.pairs.extend(found.pairs);
        paired.removed.extend(found.removed);
        paired.added.extend(found.added);
    }
    paired.added.extend(targets.into_values().flatten());
    let mut comparison = Comparison {
        added: paired.added.into_iter().cloned().collect(),
        removed: paired.removed.into_iter().cloned().collect(),
        ..Comparison::default()
    };
    for (base, target) in paired.pairs {
        sort_pair(&mut comparison, base, target);
    }
    comparison.sort();
    comparison
}

/// Files a pair under moved (another file) or changed (reasons), or drops it if the
/// two sides are the same.
fn sort_pair(comparison: &mut Comparison, base: &SymbolFact, target: &SymbolFact) {
    let matched = Matched {
        base: base.clone(),
        target: target.clone(),
        reasons: reasons_of(base, target),
    };
    if base.path != target.path {
        comparison.moved.push(matched);
    } else if !matched.reasons.is_empty() {
        comparison.changed.push(matched);
    }
}
