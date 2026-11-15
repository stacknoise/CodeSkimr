//! The calls of a constructor that javac counts and scip-java does not write (issue #259,
//! `docs/level2-import.md` point 14): the call of the constructor of the superclass that a
//! constructor without `this(...)` or `super(...)` implies, and the call that `new Type(...) {..}`
//! makes for its anonymous subclass. Both are rules of the language, and both are applied only
//! where the target is certain: the superclass is the one the source names in its `extends`, and
//! the constructor is the one without parameters (or else the one with only a varargs parameter),
//! or, for an anonymous subclass, the only one the type has. Anything else stays unmarked.

use std::collections::HashMap;

use codeskimr_domain::{Confidence, FileId, Reference, ReferenceKind, Span, SymbolId, SymbolKind};

use super::{Importer, LevelOne};
use crate::consts::NAME_QUOTE;
use crate::model::{AnonymousCreation, Arity, ConstructorSite, ScipDocument, ScipIndex, SymbolRef};

/// A constructor of a type, as the text of its file shows it.
#[derive(Debug, Clone, Copy)]
pub(super) struct ConstructorEntry {
    symbol: SymbolRef,
    arity: Arity,
    declared: bool,
}

/// What the text of the documents tells about constructors.
#[derive(Debug, Default)]
pub(super) struct Constructors {
    /// The constructors of every type.
    pub(super) by_owner: HashMap<SymbolRef, Vec<ConstructorEntry>>,
    /// The type of each default constructor. `new Type()` of a class without a constructor is a
    /// use of the default constructor, which has no symbol in the project: it is a use of the
    /// type.
    pub(super) defaults: HashMap<SymbolRef, SymbolRef>,
}

/// The text of the type that a constructor symbol belongs to: the symbol up to its last
/// descriptor, which is the quoted name of the constructor.
fn owner_text(constructor: &str) -> Option<&str> {
    let close = constructor.rfind(NAME_QUOTE)?;
    let open = constructor.get(..close)?.rfind(NAME_QUOTE)?;
    constructor.get(..open)
}

/// The constructors the documents define, by the symbol of their type, and the type of every
/// constructor that the source does not declare (the default constructor of a class).
pub(super) fn constructors_by_owner(index: &ScipIndex) -> Constructors {
    let mut found = Constructors::default();
    if index
        .documents
        .iter()
        .all(|document| document.facts.constructors.is_empty())
    {
        return found;
    }
    let by_text: HashMap<&str, SymbolRef> = index
        .symbols
        .iter()
        .map(String::as_str)
        .zip((0..).map(SymbolRef))
        .collect();
    for site in index
        .documents
        .iter()
        .flat_map(|document| &document.facts.constructors)
    {
        let owner = index
            .symbol(site.symbol)
            .and_then(owner_text)
            .and_then(|text| by_text.get(text));
        let Some(owner) = owner else { continue };
        if !site.declared {
            found.defaults.insert(site.symbol, *owner);
        }
        let entries = found.by_owner.entry(*owner).or_default();
        if entries.iter().all(|entry| entry.symbol != site.symbol) {
            entries.push(ConstructorEntry {
                symbol: site.symbol,
                arity: site.arity,
                declared: site.declared,
            });
        }
    }
    found
}

/// The constructor that `super()` calls: the declared one without parameters, else the only
/// declared one with a single varargs parameter, else none.
fn constructor_without_arguments(entries: &[ConstructorEntry]) -> Option<SymbolRef> {
    let declared = || entries.iter().filter(|entry| entry.declared);
    let mut plain = declared().filter(|entry| entry.arity == Arity::Zero);
    if let Some(first) = plain.next() {
        return plain.next().is_none().then_some(first.symbol);
    }
    let mut varargs = declared().filter(|entry| entry.arity == Arity::One { varargs: true });
    let first = varargs.next()?;
    varargs.next().is_none().then_some(first.symbol)
}

impl Importer<'_> {
    /// The superclass of every class: the symbol that stands in its `extends`, where level 1
    /// reads an inheritance at the same place and the symbol is a class of the project. A class
    /// with two answers has none.
    pub(super) fn collect_superclasses(
        &mut self,
        covered: &[(&ScipDocument, FileId)],
        level_one: &LevelOne<'_>,
    ) {
        if self.constructors.by_owner.is_empty() {
            return;
        }
        for (document, file) in covered {
            for occurrence in document.occurrences.iter().filter(|o| !o.is_definition()) {
                let is_class = self.frozen.get(&occurrence.symbol).is_some_and(|targets| {
                    targets
                        .iter()
                        .any(|id| self.kinds_of.get(id) == Some(&SymbolKind::Class))
                });
                let inherits = || {
                    level_one
                        .starting_at(*file, occurrence.range)
                        .iter()
                        .any(|reference| reference.kind == ReferenceKind::Inherit)
                };
                if !is_class || !inherits() {
                    continue;
                }
                let from = self.table.innermost(*file, occurrence.range.start());
                if let Some(class) = from.filter(|symbol| symbol.kind == SymbolKind::Class) {
                    let supers = self.superclass.entry(class.id).or_default();
                    if !supers.contains(&occurrence.symbol) {
                        supers.push(occurrence.symbol);
                    }
                }
            }
        }
    }

    /// The references that the text of one document implies.
    pub(super) fn implicit_calls(
        &mut self,
        file: FileId,
        document: &ScipDocument,
    ) -> Vec<Reference> {
        let mut found = Vec::new();
        for site in &document.facts.constructors {
            found.extend(self.super_call(file, site));
        }
        for creation in &document.facts.anonymous {
            found.extend(self.anonymous_call(file, creation));
        }
        for name in &document.facts.enum_bodies {
            found.extend(self.enum_body_use(file, *name));
        }
        found
    }

    /// The call of `super()` that a constructor without an explicit call makes.
    fn super_call(&mut self, file: FileId, site: &ConstructorSite) -> Vec<Reference> {
        if site.explicit_call {
            return Vec::new();
        }
        let Some(from) = self.table.innermost(file, site.call_at.start()) else {
            return Vec::new();
        };
        let class = if site.declared {
            let own = self.frozen.get(&site.symbol);
            if !own.is_some_and(|targets| targets.contains(&from.id)) {
                return Vec::new();
            }
            self.owners.get(&from.id).copied()
        } else {
            (from.kind == SymbolKind::Class).then_some(from.id)
        };
        let target = class
            .and_then(|class| self.superclass.get(&class))
            .and_then(|supers| match supers.as_slice() {
                [only] => self.constructors.by_owner.get(only),
                _ => None,
            })
            .and_then(|entries| constructor_without_arguments(entries));
        self.calls(file, from.id, site.call_at, target)
    }

    /// The call of the only constructor of the type that `new Type(...) { ... }` makes.
    fn anonymous_call(&mut self, file: FileId, creation: &AnonymousCreation) -> Vec<Reference> {
        let Some(from) = self.table.innermost(file, creation.body.start()) else {
            return Vec::new();
        };
        let entries = self.constructors.by_owner.get(&creation.symbol);
        let mut declared = entries.into_iter().flatten().filter(|entry| entry.declared);
        let target = declared
            .next()
            .filter(|_| declared.next().is_none())
            .map(|entry| entry.symbol);
        self.calls(file, from.id, creation.body, target)
    }

    /// The use of an enum that an enum constant with a body stands for: the constant creates a
    /// subclass of the enum. From the constant to the enum, as a use of the type.
    fn enum_body_use(&mut self, file: FileId, name: Span) -> Option<Reference> {
        let constant = self.table.innermost(file, name.start())?;
        let parent = constant.parent?;
        if constant.kind != SymbolKind::EnumMember
            || self.kinds_of.get(&parent) != Some(&SymbolKind::Enum)
        {
            return None;
        }
        Some(Reference {
            id: self.fresh_id(),
            from: constant.id,
            to: parent,
            file,
            span: name,
            kind: ReferenceKind::TypeUse,
            confidence: Confidence::Certain,
            verified: true,
        })
    }

    /// One call reference per project symbol of the constructor `target`.
    fn calls(
        &mut self,
        file: FileId,
        from: SymbolId,
        span: Span,
        target: Option<SymbolRef>,
    ) -> Vec<Reference> {
        let Some(targets) = target.and_then(|symbol| self.frozen.get(&symbol).cloned()) else {
            return Vec::new();
        };
        targets
            .iter()
            .map(|to| Reference {
                id: self.fresh_id(),
                from,
                to: *to,
                file,
                span,
                kind: ReferenceKind::Call,
                confidence: Confidence::Certain,
                verified: true,
            })
            .collect()
    }
}
