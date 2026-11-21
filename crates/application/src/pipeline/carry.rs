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
//!
//! The references of the previous snapshot are read only as far as the scan needs them
//! (ADR 0086). An unchanged file is touched if one of its names is resolved again or one of its
//! references lost its target; only the references of touched files come into memory. Of the
//! untouched files the scan takes what coupling needs ([`Carried::pairs`]) and the number of
//! ambiguous places, which the previous snapshot counted.

use std::collections::{BTreeSet, HashSet};

use codeskimr_domain::{
    Confidence, FileId, Reference, ReferenceId, ReferenceKind, RelativePath, SnapshotId,
    SourceFile, Span, Symbol, SymbolId, SymbolOrigin, SymbolPair,
};

use super::shape::{Delta, Shape, is_structural};
use crate::error::CodedError;
use crate::ports::{
    DiscoveredFile, PendingReference, PreviousState, ProjectRepository, StoredNames,
    StoredReferences,
};
use crate::resolve::ambiguous_occurrences;

/// The kept part of the previous snapshot. Empty in a full scan.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Carried {
    /// Files that did not change.
    pub files: Vec<SourceFile>,
    /// Paths of the previous snapshot that no longer exist.
    pub removed: Vec<RelativePath>,
    /// Symbols of the unchanged files.
    pub symbols: Vec<Symbol>,
    /// References from the touched unchanged files to symbols that still exist, read by
    /// [`Carried::reconcile`], which removes those that a full scan would no longer produce. The
    /// references of the untouched files stay in the store.
    pub references: Vec<Reference>,
    /// What coupling needs of the references of the untouched files: start and target, each pair
    /// once, references of kind `test` left out.
    pub pairs: Vec<SymbolPair>,
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
    /// The files of the previous snapshot that changed or are gone.
    left: Vec<FileId>,
    /// How many places of the previous snapshot match several symbols, `None` if it did not count.
    counted: Option<u64>,
    /// How many places of the untouched files match several symbols.
    elsewhere: u64,
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

impl Carried {
    /// The previous snapshot this was split from; `None` in a full scan.
    #[must_use]
    pub const fn snapshot(&self) -> Option<SnapshotId> {
        self.snapshot
    }

    /// Splits the previous snapshot into what is kept and what is gone, and marks every
    /// discovered file that the previous snapshot does not know as changed. Of the references it
    /// reads those that lost their target and which files level 2 verified.
    ///
    /// # Errors
    /// A coded failure of the store.
    pub fn from_previous(
        discovered: &mut [DiscoveredFile],
        previous: PreviousState,
        bases: Vec<PendingReference>,
        store: &dyn ProjectRepository,
    ) -> Result<Self, CodedError> {
        let files = previous.files;
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
        let (kept, left): (Vec<SourceFile>, Vec<SourceFile>) = files
            .into_iter()
            .partition(|file| unchanged.contains(&file.path));
        let (mut carried, vanished) = Self::keep(kept, previous.symbols, bases);
        carried.removed = removed;
        carried.left = left.iter().map(|file| file.id).collect();
        carried.counted = previous.ambiguous_places;
        if let Some(snapshot) = previous.snapshot {
            carried.snapshot = Some(snapshot);
            carried.mark_gone(snapshot, &vanished, store)?;
        }
        Ok(carried)
    }

    /// Keeps the symbols and names of `files`; the other fields stay empty, except what the
    /// resolver saw of the files that are not kept. Returns the symbols of those files with it.
    fn keep(
        files: Vec<SourceFile>,
        symbols: Vec<Symbol>,
        bases: Vec<PendingReference>,
    ) -> (Self, Vec<SymbolId>) {
        let kept: HashSet<FileId> = files.iter().map(|file| file.id).collect();
        let (symbols, vanished): (Vec<Symbol>, Vec<Symbol>) = symbols
            .into_iter()
            .partition(|symbol| kept.contains(&symbol.file));
        let (names, vanished_names): (Vec<PendingReference>, Vec<PendingReference>) = bases
            .into_iter()
            .partition(|name| kept.contains(&name.file));
        let gone = vanished.iter().map(|symbol| symbol.id).collect();
        let carried = Self {
            files,
            symbols,
            bases: names,
            // The symbols level 2 created are not part of what level 1 sees (ADR 0055).
            previous_shape: Shape::of(&level_one(vanished), &vanished_names),
            ..Self::default()
        };
        (carried, gone)
    }

    /// Reads the references of unchanged files whose target `vanished` is gone, and what level 2
    /// left in the unchanged files.
    fn mark_gone(
        &mut self,
        snapshot: SnapshotId,
        vanished: &[SymbolId],
        store: &dyn ProjectRepository,
    ) -> Result<(), CodedError> {
        let kept: HashSet<FileId> = self.files.iter().map(|file| file.id).collect();
        let gone: Vec<Reference> = store
            .stored_references(snapshot, StoredReferences::To(vanished))?
            .into_iter()
            .filter(|reference| kept.contains(&reference.file))
            .collect();
        self.verified_files = store
            .verified_files(snapshot)?
            .into_iter()
            .filter(|file| kept.contains(file))
            .collect();
        self.lost = gone.iter().filter(|r| r.verified).map(site).collect();
        self.discarded_verified =
            u32::try_from(gone.iter().filter(|r| r.verified).count()).unwrap_or(u32::MAX);
        self.dead = gone.iter().map(site).collect();
        Ok(())
    }

    /// Reads the references that [`Carried::reconcile`] compares: those that start in a touched
    /// symbol, or all if every name is resolved again or the previous snapshot did not count its
    /// ambiguous places. Of the other references it takes the pairs for coupling and the number
    /// of ambiguous places, which is the count of the snapshot without what is read (ADR 0086).
    fn read_touched(&mut self, store: &dyn ProjectRepository) -> Result<(), CodedError> {
        let Some(snapshot) = self.snapshot else {
            return Ok(());
        };
        let kept: Vec<FileId> = self.files.iter().map(|file| file.id).collect();
        let old = if let (Some(counted), Some(touched)) = (self.counted, self.touched()) {
            let old = store.stored_references(snapshot, StoredReferences::From(&touched))?;
            let left = store.stored_references(snapshot, StoredReferences::In(&self.left))?;
            let read = ambiguous_occurrences(old.iter().chain(&left));
            self.elsewhere = counted.saturating_sub(read);
            self.pairs = store.reference_pairs(snapshot, &kept, &touched)?;
            old
        } else {
            store.stored_references(snapshot, StoredReferences::All)?
        };
        let kept: HashSet<FileId> = kept.into_iter().collect();
        let alive: HashSet<SymbolId> = self.symbols.iter().map(|symbol| symbol.id).collect();
        self.references = old
            .into_iter()
            .filter(|r| kept.contains(&r.file) && alive.contains(&r.to))
            .collect();
        Ok(())
    }

    /// The touched symbols in ascending order: those in which a name is resolved again or a
    /// reference lost its target. `None` if all names are resolved again.
    fn touched(&self) -> Option<Vec<SymbolId>> {
        let scope = self.scope.as_ref()?;
        let touched: BTreeSet<SymbolId> = scope
            .iter()
            .chain(&self.dead)
            .map(|(from, _, _)| *from)
            .collect();
        Some(touched.into_iter().collect())
    }

    /// How many places of the whole snapshot match several symbols, given the references
    /// `written` by this scan.
    #[must_use]
    pub fn ambiguous_places(&self, written: &[Reference]) -> u64 {
        let here = ambiguous_occurrences(written.iter().chain(&self.references));
        self.elsewhere.saturating_add(here)
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
    ///
    /// # Errors
    /// A coded failure of the store, which the kept references are read from.
    pub fn reconcile(
        &mut self,
        resolved: Vec<Reference>,
        store: &dyn ProjectRepository,
    ) -> Result<Vec<Reference>, CodedError> {
        self.read_touched(store)?;
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
        Ok(resolved
            .into_iter()
            .filter(|reference| {
                !kept_files.contains(&reference.file)
                    || self.is_wanted(reference) && !held.contains(&edge(reference))
            })
            .collect())
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
