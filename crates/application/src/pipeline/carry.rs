//! What an incremental scan keeps of the previous snapshot (blueprint section 7).
//!
//! Unchanged files are not analysed again, but their symbols are part of the global symbol table
//! and their names are resolved again against it, together with the new names. The result must be
//! the one a full scan would give (issue #49): the references that come out equal are kept, new
//! ones are written, kept ones that are no longer produced are dropped. References that level 2
//! verified are the exception, see [`Carried::reconcile`].
//!
//! Which names are resolved again depends on what changed ([`Carried::due_names`]): if the changed
//! files show the same [`Shape`] as before, only the names that pointed into them (and the imports
//! and base lists, which tell the resolver what a file sees), otherwise all names.

use std::collections::HashSet;

use codeskimr_domain::{
    Confidence, FileId, Reference, ReferenceId, ReferenceKind, RelativePath, SourceFile, Span,
    Symbol, SymbolId,
};

use super::shape::{Shape, is_base};
use crate::ports::{DiscoveredFile, PendingReference, PreviousIndex, PreviousState};

/// The kept part of the previous snapshot. Empty in a full scan.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Carried {
    /// Files that did not change.
    pub files: Vec<SourceFile>,
    /// Paths of the previous snapshot that no longer exist.
    pub removed: Vec<RelativePath>,
    /// Symbols of the unchanged files.
    pub symbols: Vec<Symbol>,
    /// References from unchanged files to symbols that still exist. [`Carried::reconcile`] removes
    /// those that a full scan would no longer produce.
    pub references: Vec<Reference>,
    /// The names as written in the unchanged files, to be resolved again with every scan.
    pub names: Vec<PendingReference>,
    /// Kept references that the resolution no longer produces; the store drops them.
    pub dropped: Vec<ReferenceId>,
    /// How many references that level 2 had verified pointed to a symbol that is gone and were
    /// discarded (ADR 0033, decision 13): the occurrence falls back to level 1.
    pub discarded_verified: u32,
    /// Unchanged files that hold references verified by level 2.
    verified_files: HashSet<FileId>,
    /// Places whose verified reference was discarded.
    lost: HashSet<Site>,
    /// Places of references whose target is gone.
    dead: HashSet<Site>,
    /// What the resolver saw of the files that changed or disappeared, before.
    previous_shape: Shape,
    /// The places whose names are resolved again, `None` if all are.
    scope: Option<HashSet<Site>>,
}

/// The place of an occurrence of a name; one place can hold several names of different kinds.
type Site = (SymbolId, FileId, Span);

fn site(reference: &Reference) -> Site {
    (reference.from, reference.file, reference.span)
}

/// The identity of one occurrence of a name.
type Occurrence = (SymbolId, FileId, Span, ReferenceKind);

/// What makes two references the same edge with the same strength.
type Edge = (Occurrence, SymbolId, Confidence);

fn occurrence(reference: &Reference) -> Occurrence {
    (
        reference.from,
        reference.file,
        reference.span,
        reference.kind,
    )
}

fn edge(reference: &Reference) -> Edge {
    (occurrence(reference), reference.to, reference.confidence)
}

/// What level 2 left in the unchanged files.
struct Level2 {
    files: HashSet<FileId>,
    lost: HashSet<Site>,
    discarded: u32,
}

impl Level2 {
    /// Reads it from the references that stay and those whose target is gone.
    fn of(stay: &[Reference], gone: &[Reference]) -> Self {
        let lost: HashSet<Site> = gone
            .iter()
            .filter(|reference| reference.verified)
            .map(site)
            .collect();
        Self {
            files: stay
                .iter()
                .chain(gone)
                .filter(|reference| reference.verified)
                .map(|reference| reference.file)
                .collect(),
            discarded: u32::try_from(gone.iter().filter(|r| r.verified).count())
                .unwrap_or(u32::MAX),
            lost,
        }
    }
}

impl Carried {
    /// Splits the previous snapshot into what is kept and what is gone, and marks every
    /// discovered file that the previous snapshot does not know as changed.
    #[must_use]
    pub fn from_previous(discovered: &mut [DiscoveredFile], previous: PreviousState) -> Self {
        let PreviousState {
            files,
            index,
            pending,
            names_stored: _,
        } = previous;
        let known: HashSet<&RelativePath> = files.iter().map(|file| &file.path).collect();
        for file in discovered
            .iter_mut()
            .filter(|file| !known.contains(&file.path))
        {
            file.changed = true;
        }
        let unchanged: HashSet<&RelativePath> = discovered
            .iter()
            .filter(|file| !file.changed)
            .map(|file| &file.path)
            .collect();
        let current: HashSet<&RelativePath> = discovered.iter().map(|file| &file.path).collect();
        let removed = files
            .iter()
            .filter(|file| !current.contains(&file.path))
            .map(|file| file.path.clone())
            .collect();
        let kept: Vec<SourceFile> = files
            .into_iter()
            .filter(|file| unchanged.contains(&file.path))
            .collect();
        Self {
            removed,
            ..Self::keep(kept, index, pending)
        }
    }

    /// Keeps the symbols, references and names of `files`; the other fields stay empty, except
    /// what the resolver saw of the files that are not kept.
    fn keep(files: Vec<SourceFile>, index: PreviousIndex, pending: Vec<PendingReference>) -> Self {
        let kept: HashSet<FileId> = files.iter().map(|file| file.id).collect();
        let PreviousIndex {
            symbols,
            references,
        } = index;
        let (symbols, vanished): (Vec<Symbol>, Vec<Symbol>) = symbols
            .into_iter()
            .partition(|symbol| kept.contains(&symbol.file));
        let (names, vanished_names): (Vec<PendingReference>, Vec<PendingReference>) = pending
            .into_iter()
            .partition(|name| kept.contains(&name.file));
        let alive: HashSet<SymbolId> = symbols.iter().map(|symbol| symbol.id).collect();
        let (stay, gone): (Vec<Reference>, Vec<Reference>) = references
            .into_iter()
            .filter(|reference| kept.contains(&reference.file))
            .partition(|reference| alive.contains(&reference.to));
        let level_two = Level2::of(&stay, &gone);
        Self {
            files,
            symbols,
            references: stay,
            names,
            discarded_verified: level_two.discarded,
            verified_files: level_two.files,
            lost: level_two.lost,
            dead: gone.iter().map(site).collect(),
            previous_shape: Shape::of(&vanished, &vanished_names),
            ..Self::default()
        }
    }

    /// The names to resolve again besides the new ones `fresh`, written in the files `symbols`
    /// belong to. If the changed files show the same shape as before, nothing an unchanged name
    /// resolves against has changed but the ids of the symbols: the names that pointed at such a
    /// symbol are resolved again, and the imports and base lists, which the resolver needs to know
    /// what a file sees and which type inherits from which. Otherwise all names are.
    pub fn due_names(
        &mut self,
        symbols: &[Symbol],
        fresh: &[PendingReference],
    ) -> Vec<PendingReference> {
        if Shape::of(symbols, fresh) != self.previous_shape {
            self.scope = None;
            return self.names.clone();
        }
        let due: Vec<PendingReference> = self
            .names
            .iter()
            .filter(|name| {
                is_base(name.kind)
                    || name.kind == ReferenceKind::Import
                    || self.dead.contains(&(name.from, name.file, name.span))
            })
            .cloned()
            .collect();
        self.scope = Some(
            due.iter()
                .map(|name| (name.from, name.file, name.span))
                .collect(),
        );
        due
    }

    /// Takes the references `resolved` for all names of this scan (the new ones and `names`) and
    /// returns those that must be written: every reference of a changed file, and those of an
    /// unchanged file that the kept ones do not already contain with the same target and
    /// confidence. Kept references that are not produced again are moved to `dropped`.
    ///
    /// A file with references verified by level 2 keeps them (level 2 replaced its level 1
    /// references, ADR 0033), and level 1 adds nothing to it except the occurrences whose verified
    /// reference was discarded because its target is gone.
    pub fn reconcile(&mut self, resolved: Vec<Reference>) -> Vec<Reference> {
        let kept_files: HashSet<FileId> = self.files.iter().map(|file| file.id).collect();
        let produced: HashSet<Edge> = resolved
            .iter()
            .filter(|reference| kept_files.contains(&reference.file))
            .map(edge)
            .collect();
        let held: HashSet<Edge> = self.references.iter().map(edge).collect();
        let (stale, stay): (Vec<Reference>, Vec<Reference>) = std::mem::take(&mut self.references)
            .into_iter()
            .partition(|reference| {
                !self.verified_files.contains(&reference.file)
                    && self.in_scope(reference)
                    && !produced.contains(&edge(reference))
            });
        self.dropped
            .extend(stale.iter().map(|reference| reference.id));
        self.references = stay;
        resolved
            .into_iter()
            .filter(|reference| {
                !kept_files.contains(&reference.file)
                    || self.is_wanted(reference) && !held.contains(&edge(reference))
            })
            .collect()
    }

    /// Whether level 1 may write the reference of an unchanged file.
    fn is_wanted(&self, reference: &Reference) -> bool {
        !self.verified_files.contains(&reference.file) || self.lost.contains(&site(reference))
    }

    /// Whether the name behind the reference was resolved again in this scan.
    fn in_scope(&self, reference: &Reference) -> bool {
        self.scope
            .as_ref()
            .is_none_or(|scope| scope.contains(&site(reference)))
    }
}
