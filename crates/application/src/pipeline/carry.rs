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
    Confidence, FileId, Reference, ReferenceId, ReferenceKind, RelativePath, SnapshotId,
    SourceFile, Span, Symbol, SymbolId, SymbolOrigin,
};

use super::shape::{Shape, is_structural};
use crate::error::CodedError;
use crate::ports::{
    DiscoveredFile, PendingReference, PreviousIndex, PreviousState, ProjectRepository, StoredNames,
};

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
    /// The base lists of the unchanged files, which every scan resolves again.
    pub bases: Vec<PendingReference>,
    /// The snapshot the other names are read from when they are needed.
    snapshot: Option<SnapshotId>,
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

/// The symbols that level 1 made itself.
fn level_one(symbols: Vec<Symbol>) -> Vec<Symbol> {
    symbols
        .into_iter()
        .filter(|symbol| symbol.origin == SymbolOrigin::Syntax)
        .collect()
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
    pub fn from_previous(
        discovered: &mut [DiscoveredFile],
        previous: PreviousState,
        bases: Vec<PendingReference>,
    ) -> Self {
        let PreviousState {
            files,
            index,
            snapshot,
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
            snapshot,
            ..Self::keep(kept, index, bases)
        }
    }

    /// Keeps the symbols, references and names of `files`; the other fields stay empty, except
    /// what the resolver saw of the files that are not kept.
    fn keep(files: Vec<SourceFile>, index: PreviousIndex, bases: Vec<PendingReference>) -> Self {
        let kept: HashSet<FileId> = files.iter().map(|file| file.id).collect();
        let PreviousIndex {
            symbols,
            references,
        } = index;
        let (symbols, vanished): (Vec<Symbol>, Vec<Symbol>) = symbols
            .into_iter()
            .partition(|symbol| kept.contains(&symbol.file));
        let (names, vanished_names): (Vec<PendingReference>, Vec<PendingReference>) = bases
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
            bases: names,
            discarded_verified: level_two.discarded,
            verified_files: level_two.files,
            lost: level_two.lost,
            dead: gone.iter().map(site).collect(),
            // The symbols level 2 created are not part of what level 1 sees (ADR 0055).
            previous_shape: Shape::of(&level_one(vanished), &vanished_names),
            ..Self::default()
        }
    }

    /// The names to resolve again besides the new ones `fresh`, written in the files `symbols`
    /// belong to. If the changed files show the same shape as before, nothing an unchanged name
    /// resolves against has changed but the ids of the symbols: the names written inside the
    /// symbols that pointed at such a symbol are resolved again, and the imports and base lists,
    /// which the resolver needs to know what a file sees and which type inherits from which.
    /// Otherwise all names are. The names come from the store only now, and only as far as they are
    /// needed.
    ///
    /// # Errors
    /// A coded failure of the store.
    pub fn due_names(
        &mut self,
        (symbols, fresh): (&[Symbol], &[PendingReference]),
        store: &dyn ProjectRepository,
    ) -> Result<Vec<PendingReference>, CodedError> {
        let Some(snapshot) = self.snapshot else {
            return Ok(Vec::new());
        };
        if Shape::of(symbols, fresh) != self.previous_shape {
            self.scope = None;
            return self.all_names(snapshot, store);
        }
        let due = self.narrow_names(snapshot, store)?;
        self.scope = Some(
            due.iter()
                .map(|name| (name.from, name.file, name.span))
                .collect(),
        );
        Ok(due)
    }

    /// The base lists, the names written inside the symbols that pointed at a gone symbol, and the
    /// imports of the files of those names and of the base lists, which tell what these see.
    fn narrow_names(
        &self,
        snapshot: SnapshotId,
        store: &dyn ProjectRepository,
    ) -> Result<Vec<PendingReference>, CodedError> {
        let callers: HashSet<SymbolId> = self.dead.iter().map(|(from, _, _)| *from).collect();
        let callers: Vec<SymbolId> = callers.into_iter().collect();
        let dead_files: HashSet<FileId> = self.dead.iter().map(|(_, file, _)| *file).collect();
        let dead_files: Vec<FileId> = dead_files.into_iter().collect();
        let mut due = self.bases.clone();
        due.extend(
            store
                .stored_names(snapshot, StoredNames::Callers(&dead_files, &callers))?
                .into_iter()
                .filter(|name| !is_structural(name.kind))
                .filter(|name| self.dead.contains(&(name.from, name.file, name.span))),
        );
        let mut files: HashSet<FileId> = due.iter().map(|name| name.file).collect();
        files.extend(dead_files);
        let files: Vec<FileId> = files.into_iter().collect();
        due.extend(store.stored_names(snapshot, StoredNames::ImportsOf(&files))?);
        Ok(due)
    }

    /// Every name of the unchanged files.
    fn all_names(
        &self,
        snapshot: SnapshotId,
        store: &dyn ProjectRepository,
    ) -> Result<Vec<PendingReference>, CodedError> {
        let kept: HashSet<FileId> = self.files.iter().map(|file| file.id).collect();
        Ok(store
            .stored_names(snapshot, StoredNames::All)?
            .into_iter()
            .filter(|name| kept.contains(&name.file))
            .collect())
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
        let held: HashSet<Edge> = self
            .references
            .iter()
            .filter(|reference| self.in_scope(reference))
            .map(edge)
            .collect();
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
