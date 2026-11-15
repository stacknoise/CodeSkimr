//! What an incremental scan keeps of the previous snapshot (blueprint section 7).
//!
//! Unchanged files are not analysed again, but their symbols are part of the global symbol table
//! and their names are resolved again against it, together with the new names. The result must be
//! the one a full scan would give (issue #49): the references that come out equal are kept, new
//! ones are written, kept ones that are no longer produced are dropped. References that level 2
//! verified are the exception, see [`Carried::reconcile`].
//!
//! Which names are resolved again depends on what changed ([`Carried::due_names`]): the names that
//! pointed into the changed files, the names the resolver looks up by a symbol whose [`Shape`]
//! differs, the files that import a changed qualified name, and the imports and base lists, which
//! tell the resolver what a file sees; all names only if a base list changed (issue #408).

use std::collections::HashSet;

use codeskimr_domain::{
    Confidence, FileId, Reference, ReferenceId, ReferenceKind, RelativePath, SnapshotId,
    SourceFile, Span, Symbol, SymbolId, SymbolOrigin,
};

use super::shape::{Delta, Shape, is_structural};
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
    /// belong to. What an unchanged name resolves against differs from before only where a symbol
    /// was added, removed, renamed, moved or changed in kind in the changed files, and where an
    /// id changed. So these names are resolved again (issue #408):
    ///
    /// - the base lists, which the resolver needs to know which type inherits from which,
    /// - the names written inside the symbols that pointed at a gone symbol,
    /// - the names that the resolver looks up by the simple name of a changed symbol, as target or
    ///   as named receiver,
    /// - all names of a file with an import of a changed qualified name, because the import decides
    ///   what the file sees,
    /// - the imports of the files of all these names.
    ///
    /// If a base list changed, a type can inherit something else and all names are resolved again.
    /// The names come from the store only now, and only as far as they are needed.
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
        let Some(delta) = Shape::of(symbols, fresh).delta(&self.previous_shape) else {
            self.scope = None;
            return self.all_names(snapshot, store);
        };
        let due = self.narrow_names(snapshot, &delta, store)?;
        self.scope = Some(
            due.iter()
                .map(|name| (name.from, name.file, name.span))
                .collect(),
        );
        Ok(due)
    }

    /// The names that [`Carried::due_names`] lists for a scan whose changed files differ by `delta`.
    fn narrow_names(
        &self,
        snapshot: SnapshotId,
        delta: &Delta,
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
        due.extend(self.looked_up_by(snapshot, delta, store)?);
        let mut files: HashSet<FileId> = due.iter().map(|name| name.file).collect();
        files.extend(dead_files);
        files.extend(self.names_of_importers(snapshot, delta, store, &mut due)?);
        let files: Vec<FileId> = files.into_iter().collect();
        due.extend(store.stored_names(snapshot, StoredNames::ImportsOf(&files))?);
        Ok(Self::distinct(due))
    }

    /// The names of unchanged files that the resolver looks up by a changed simple name.
    fn looked_up_by(
        &self,
        snapshot: SnapshotId,
        delta: &Delta,
        store: &dyn ProjectRepository,
    ) -> Result<Vec<PendingReference>, CodedError> {
        if delta.names.is_empty() {
            return Ok(Vec::new());
        }
        let names: Vec<String> = delta.names.iter().cloned().collect();
        Ok(self.kept(store.stored_names(snapshot, StoredNames::Targeting(&names))?))
    }

    /// Adds every name of the files with an import of a changed qualified name to `due` and
    /// returns those files.
    fn names_of_importers(
        &self,
        snapshot: SnapshotId,
        delta: &Delta,
        store: &dyn ProjectRepository,
        due: &mut Vec<PendingReference>,
    ) -> Result<Vec<FileId>, CodedError> {
        if delta.qualified.is_empty() {
            return Ok(Vec::new());
        }
        let qualified: Vec<String> = delta.qualified.iter().cloned().collect();
        let importers: HashSet<FileId> = self
            .kept(store.stored_names(snapshot, StoredNames::ImportsTargeting(&qualified))?)
            .into_iter()
            .map(|name| name.file)
            .collect();
        let importers: Vec<FileId> = importers.into_iter().collect();
        due.extend(self.kept(store.stored_names(snapshot, StoredNames::OfFiles(&importers))?));
        Ok(importers)
    }

    /// The names that belong to an unchanged file. The store reads names of the whole previous
    /// snapshot, which holds the files that changed too.
    fn kept(&self, names: Vec<PendingReference>) -> Vec<PendingReference> {
        let kept: HashSet<FileId> = self.files.iter().map(|file| file.id).collect();
        names
            .into_iter()
            .filter(|name| kept.contains(&name.file))
            .collect()
    }

    /// `names` without the occurrences that appear twice.
    fn distinct(names: Vec<PendingReference>) -> Vec<PendingReference> {
        let mut seen: HashSet<(Site, ReferenceKind)> = HashSet::new();
        names
            .into_iter()
            .filter(|name| seen.insert(((name.from, name.file, name.span), name.kind)))
            .collect()
    }

    /// Every name of the unchanged files.
    fn all_names(
        &self,
        snapshot: SnapshotId,
        store: &dyn ProjectRepository,
    ) -> Result<Vec<PendingReference>, CodedError> {
        Ok(self.kept(store.stored_names(snapshot, StoredNames::All)?))
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
