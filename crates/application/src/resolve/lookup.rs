//! The resolution steps of blueprint 9.2 for one pending name.

use std::collections::HashSet;

use codeskimr_domain::{Confidence, FileId, ReferenceKind, Symbol, SymbolId, SymbolKind};

use super::consts::NAME_JOINER;
use super::context::Context;
use super::kinds::{OVERRIDABLE, TYPES, candidates, is_overridable, is_type, reported};
use crate::ports::{PendingReference, Receiver};

/// One symbol that a pending name resolves to.
#[derive(Debug, Clone, Copy)]
pub struct Hit {
    /// The target.
    pub to: SymbolId,
    /// The kind the reference is reported as.
    pub kind: ReferenceKind,
    /// How sure the resolution is.
    pub confidence: Confidence,
}

/// Candidates of one step and whether they come from the last resort.
struct Found<'a> {
    symbols: Vec<&'a Symbol>,
    /// Step 5: the name matched without any scope or type telling it apart.
    global: bool,
}

impl<'a> Found<'a> {
    const fn near(symbols: Vec<&'a Symbol>) -> Self {
        Self {
            symbols,
            global: false,
        }
    }
}

impl<'a> Context<'a> {
    /// Resolves one occurrence. Names that match nothing give no hit.
    pub fn lookup(&self, occurrence: &PendingReference) -> Vec<Hit> {
        if occurrence.kind == ReferenceKind::Import || occurrence.previous.is_some() {
            return self.exact(occurrence);
        }
        let receiver = self.receiver_types(occurrence);
        for kinds in candidates(occurrence.kind) {
            let named = self.named_of(occurrence.target.as_str(), kinds);
            let found = match &receiver {
                Some(types) => self.in_types(types, &named),
                None => self.in_scope(occurrence.from, occurrence.file, &named),
            };
            if !found.symbols.is_empty() {
                return self.hits(occurrence, found);
            }
        }
        Vec::new()
    }

    /// An import, or a name that already is a qualified name because it is resolved again.
    fn exact(&self, occurrence: &PendingReference) -> Vec<Hit> {
        let confidence = occurrence.previous.unwrap_or(Confidence::Certain);
        let imported = occurrence.kind == ReferenceKind::Import;
        self.table
            .qualified(occurrence.target.as_str())
            .iter()
            .filter(|symbol| !imported || fits(occurrence.kind, symbol.kind))
            .map(|symbol| Hit {
                to: symbol.id,
                kind: reported(occurrence.kind, symbol.kind),
                confidence,
            })
            .collect()
    }

    fn named_of(&self, name: &str, kinds: &[SymbolKind]) -> Vec<&'a Symbol> {
        self.table
            .named(name)
            .iter()
            .copied()
            .filter(|symbol| kinds.contains(&symbol.kind))
            .collect()
    }

    /// The types in which a member must be found (step 3), or `None` if the syntax says nothing
    /// usable about the receiver. A receiver of a type outside the project gives an empty list,
    /// so its members are not mistaken for project symbols of the same name.
    fn receiver_types(&self, occurrence: &PendingReference) -> Option<Vec<SymbolId>> {
        match (&occurrence.receiver, occurrence.kind) {
            (Some(Receiver::Own), _) => self.enclosing_type(occurrence.from).map(|own| vec![own]),
            (Some(Receiver::Base), _) | (None, ReferenceKind::Override) => self
                .enclosing_type(occurrence.from)
                .map(|own| self.hierarchy.direct_bases_of(own)),
            (Some(Receiver::Typed(name)), _) => Some(self.type_ids(occurrence, name.as_str())),
            (Some(Receiver::Named(name)), _) => {
                Some(self.type_ids(occurrence, name.as_str())).filter(|ids| !ids.is_empty())
            }
            (None, _) => None,
        }
    }

    fn enclosing_type(&self, from: SymbolId) -> Option<SymbolId> {
        self.table
            .ancestors(from)
            .find(|symbol| is_type(symbol.kind))
            .map(|symbol| symbol.id)
    }

    fn type_ids(&self, occurrence: &PendingReference, name: &str) -> Vec<SymbolId> {
        let named = self.named_of(name, TYPES);
        let found = self.in_scope(occurrence.from, occurrence.file, &named);
        found.symbols.iter().map(|symbol| symbol.id).collect()
    }

    /// Members of `types` and their base types.
    fn in_types(&self, types: &[SymbolId], named: &[&'a Symbol]) -> Found<'a> {
        let mut found: Vec<&'a Symbol> = types
            .iter()
            .flat_map(|&root| self.nearest(root, named))
            .collect();
        found.sort_by_key(|symbol| symbol.id);
        found.dedup_by_key(|symbol| symbol.id);
        Found::near(found)
    }

    /// The members of `root` or, if it has none of that name, of its nearest base type that has.
    fn nearest(&self, root: SymbolId, named: &[&'a Symbol]) -> Vec<&'a Symbol> {
        for id in self.hierarchy.closure(root) {
            let found: Vec<&'a Symbol> = named
                .iter()
                .copied()
                .filter(|symbol| symbol.parent == Some(id))
                .collect();
            if !found.is_empty() {
                return found;
            }
        }
        Vec::new()
    }

    /// Steps 1, 2, 4 and 5 for a name without usable receiver: enclosing scopes from the inside
    /// out, then the imports of the file, then the name alone.
    fn in_scope(&self, from: SymbolId, file: FileId, named: &[&'a Symbol]) -> Found<'a> {
        for container in self.table.ancestors(from) {
            let found = self.visible_in(container, named);
            if !found.is_empty() {
                return Found::near(found);
            }
        }
        let imported: Vec<&'a Symbol> = named
            .iter()
            .copied()
            .filter(|symbol| self.is_imported(file, symbol))
            .collect();
        if imported.is_empty() {
            Found {
                symbols: named.to_vec(),
                global: true,
            }
        } else {
            Found::near(imported)
        }
    }

    fn visible_in(&self, container: &Symbol, named: &[&'a Symbol]) -> Vec<&'a Symbol> {
        if is_type(container.kind) {
            return self.nearest(container.id, named);
        }
        named
            .iter()
            .copied()
            .filter(|symbol| self.is_declared_in(container, symbol))
            .collect()
    }

    /// Whether `symbol` is a direct child of `container`, or of a namespace that encloses it.
    fn is_declared_in(&self, container: &Symbol, symbol: &Symbol) -> bool {
        let Some(parent) = symbol.parent.and_then(|id| self.table.get(id)) else {
            return false;
        };
        parent.id == container.id
            || (container.kind == SymbolKind::Namespace
                && parent.kind == SymbolKind::Namespace
                && encloses(
                    parent.qualified_name.as_str(),
                    container.qualified_name.as_str(),
                ))
    }

    fn is_imported(&self, file: FileId, symbol: &Symbol) -> bool {
        let Some(parent) = symbol.parent.and_then(|id| self.table.get(id)) else {
            return false;
        };
        parent.kind == SymbolKind::Namespace
            && self
                .imports
                .get(&file)
                .is_some_and(|names| names.contains(parent.qualified_name.as_str()))
    }

    /// Turns candidates into hits. Several candidates, the last resort and a dispatch that can
    /// reach more targets than the written one make every hit probable (blueprint 9.2).
    fn hits(&self, occurrence: &PendingReference, found: Found<'a>) -> Vec<Hit> {
        let mut targets = found.symbols;
        let others = if occurrence.kind == ReferenceKind::Override {
            Vec::new()
        } else {
            self.dispatch_targets(&targets)
        };
        let probable = found.global || targets.len() > 1 || !others.is_empty();
        targets.extend(others);
        targets.sort_by_key(|symbol| symbol.id);
        targets.dedup_by_key(|symbol| symbol.id);
        let confidence = if probable {
            Confidence::Probable
        } else {
            Confidence::Certain
        };
        targets
            .into_iter()
            .map(|symbol| Hit {
                to: symbol.id,
                kind: reported(occurrence.kind, symbol.kind),
                confidence,
            })
            .collect()
    }

    /// The members of subtypes that replace or implement a member of `targets`: a call through
    /// a base type or an interface may end up in any of them.
    fn dispatch_targets(&self, targets: &[&'a Symbol]) -> Vec<&'a Symbol> {
        targets
            .iter()
            .filter(|member| is_overridable(member.kind))
            .flat_map(|member| self.replacements_of(member))
            .collect()
    }

    fn replacements_of(&self, member: &Symbol) -> Vec<&'a Symbol> {
        let Some(owner) = member.parent else {
            return Vec::new();
        };
        let below: HashSet<SymbolId> = self.hierarchy.descendants_of(owner).into_iter().collect();
        self.named_of(member.name.as_str(), OVERRIDABLE)
            .into_iter()
            .filter(|symbol| symbol.kind == member.kind)
            .filter(|symbol| symbol.parent.is_some_and(|parent| below.contains(&parent)))
            .collect()
    }
}

/// Whether the group of kinds of this reference kind contains `kind`.
fn fits(reference: ReferenceKind, kind: SymbolKind) -> bool {
    candidates(reference)
        .iter()
        .any(|group| group.contains(&kind))
}

/// Whether the namespace `outer` is `inner` or a part of its name in front of a separator.
fn encloses(outer: &str, inner: &str) -> bool {
    inner.strip_prefix(outer).is_some_and(|rest| {
        rest.chars()
            .next()
            .is_none_or(|next| !(next.is_alphanumeric() || next == NAME_JOINER))
    })
}
