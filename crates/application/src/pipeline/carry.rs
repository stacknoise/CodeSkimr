//! What an incremental scan keeps of the previous snapshot (blueprint section 7).
//!
//! Unchanged files are not analysed again, but their symbols are part of the global symbol table
//! and their names are resolved again against it, together with the new names. The result must be
//! the one a full scan would give (issue #49): the references that come out equal are kept, new
//! ones are written, kept ones that are no longer produced are dropped. References that level 2
//! verified are the exception, see [`Carried::reconcile`].

use std::collections::HashSet;

use codeskimr_domain::{
    Confidence, FileId, Reference, ReferenceId, ReferenceKind, RelativePath, SourceFile, Span,
    Symbol, SymbolId,
};

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
    /// Occurrences whose verified reference was discarded.
    lost: HashSet<Occurrence>,
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
    lost: HashSet<Occurrence>,
    discarded: u32,
}

impl Level2 {
    /// Reads it from the references that stay and those whose target is gone.
    fn of(stay: &[Reference], gone: &[Reference]) -> Self {
        let lost: HashSet<Occurrence> = gone
            .iter()
            .filter(|reference| reference.verified)
            .map(occurrence)
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

    /// Keeps the symbols, references and names of `files`; the other fields stay empty.
    fn keep(files: Vec<SourceFile>, index: PreviousIndex, pending: Vec<PendingReference>) -> Self {
        let kept: HashSet<FileId> = files.iter().map(|file| file.id).collect();
        let PreviousIndex {
            symbols,
            references,
        } = index;
        let symbols: Vec<Symbol> = symbols
            .into_iter()
            .filter(|symbol| kept.contains(&symbol.file))
            .collect();
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
            names: pending
                .into_iter()
                .filter(|name| kept.contains(&name.file))
                .collect(),
            discarded_verified: level_two.discarded,
            verified_files: level_two.files,
            lost: level_two.lost,
            ..Self::default()
        }
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
        !self.verified_files.contains(&reference.file) || self.lost.contains(&occurrence(reference))
    }
}
