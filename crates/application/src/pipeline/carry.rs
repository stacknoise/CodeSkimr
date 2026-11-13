//! What an incremental scan keeps of the previous snapshot (blueprint section 7).
//!
//! Unchanged files are not analysed again, but their symbols are part of the global symbol table
//! and their references must stay correct: a reference that points into a file that changed or
//! disappeared is resolved again by name, a reference between unchanged files is kept as it is.

use std::collections::{HashMap, HashSet};

use codeskimr_domain::{
    FileId, QualifiedName, Reference, ReferenceKind, RelativePath, SourceFile, Span, Symbol,
    SymbolId,
};

use crate::ports::{DiscoveredFile, PendingReference, PreviousIndex};

/// The kept part of the previous snapshot. Empty in a full scan.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Carried {
    /// Files that did not change.
    pub files: Vec<SourceFile>,
    /// Paths of the previous snapshot that no longer exist.
    pub removed: Vec<RelativePath>,
    /// Symbols of the unchanged files.
    pub symbols: Vec<Symbol>,
    /// References between unchanged files; they stay valid.
    pub references: Vec<Reference>,
    /// References of unchanged files that pointed into a changed or removed file, as names that
    /// must be resolved again.
    pub rebuilt: Vec<PendingReference>,
}

/// The identity of one occurrence of a name, used to resolve it only once.
type Occurrence = (SymbolId, FileId, Span, ReferenceKind);

impl Carried {
    /// Removes the references of `resolved` that the kept references already contain. A name with
    /// candidates in an unchanged and in a changed file is resolved again to both; the unchanged
    /// one is still kept from before.
    pub fn drop_duplicates_of_kept(&self, resolved: &mut Vec<Reference>) {
        let kept: HashSet<_> = self.references.iter().map(edge).collect();
        resolved.retain(|reference| !kept.contains(&edge(reference)));
    }

    /// The kept base-list references as names to resolve again. The resolver learns the type
    /// hierarchy from base lists only, so without these it would not know which unchanged type
    /// derives from which. The results duplicate kept references and are dropped again.
    #[must_use]
    pub fn inheritance_seed(&self) -> Vec<PendingReference> {
        let names: HashMap<SymbolId, &QualifiedName> = self
            .symbols
            .iter()
            .map(|symbol| (symbol.id, &symbol.qualified_name))
            .collect();
        self.references
            .iter()
            .filter(|reference| {
                matches!(
                    reference.kind,
                    ReferenceKind::Inherit | ReferenceKind::Implement
                )
            })
            .filter_map(|reference| {
                let target = names.get(&reference.to)?;
                Some(resolved_again(reference, (*target).clone()))
            })
            .collect()
    }

    /// Splits the previous snapshot into what is kept and what is gone, and marks every
    /// discovered file that the previous snapshot does not know as changed.
    #[must_use]
    pub fn from_previous(
        discovered: &mut [DiscoveredFile],
        previous: Vec<SourceFile>,
        index: PreviousIndex,
    ) -> Self {
        let known: HashSet<&RelativePath> = previous.iter().map(|file| &file.path).collect();
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
        let removed = previous
            .iter()
            .filter(|file| !current.contains(&file.path))
            .map(|file| file.path.clone())
            .collect();
        let files: Vec<SourceFile> = previous
            .into_iter()
            .filter(|file| unchanged.contains(&file.path))
            .collect();
        Self {
            removed,
            ..Self::keep(files, index)
        }
    }

    /// Keeps the symbols and references of `files`; the other fields stay empty.
    fn keep(files: Vec<SourceFile>, index: PreviousIndex) -> Self {
        let kept: HashSet<FileId> = files.iter().map(|file| file.id).collect();
        let PreviousIndex {
            symbols,
            references,
        } = index;
        let names: HashMap<SymbolId, QualifiedName> = symbols
            .iter()
            .map(|symbol| (symbol.id, symbol.qualified_name.clone()))
            .collect();
        let symbols: Vec<Symbol> = symbols
            .into_iter()
            .filter(|symbol| kept.contains(&symbol.file))
            .collect();
        let alive: HashSet<SymbolId> = symbols.iter().map(|symbol| symbol.id).collect();
        let (stay, redo): (Vec<Reference>, Vec<Reference>) = references
            .into_iter()
            .filter(|reference| kept.contains(&reference.file))
            .partition(|reference| alive.contains(&reference.to));
        Self {
            files,
            symbols,
            references: stay,
            rebuilt: names_to_resolve(&redo, &names),
            ..Self::default()
        }
    }
}

/// What makes two references the same occurrence of the same target.
fn edge(reference: &Reference) -> (Occurrence, SymbolId) {
    (
        (
            reference.from,
            reference.file,
            reference.span,
            reference.kind,
        ),
        reference.to,
    )
}

/// The names to resolve again, one per occurrence and target name. An occurrence that had several
/// candidates is listed once; the resolver finds the candidates again.
fn names_to_resolve(
    references: &[Reference],
    names: &HashMap<SymbolId, QualifiedName>,
) -> Vec<PendingReference> {
    let mut seen: HashSet<(Occurrence, &QualifiedName)> = HashSet::new();
    let mut pending = Vec::new();
    for reference in references {
        let Some(target) = names.get(&reference.to) else {
            continue;
        };
        let occurrence = (
            reference.from,
            reference.file,
            reference.span,
            reference.kind,
        );
        if seen.insert((occurrence, target)) {
            pending.push(resolved_again(reference, target.clone()));
        }
    }
    pending
}

/// A reference as a name to resolve again by the qualified name of its target.
fn resolved_again(reference: &Reference, target: QualifiedName) -> PendingReference {
    PendingReference {
        from: reference.from,
        file: reference.file,
        span: reference.span,
        kind: reference.kind,
        target,
        receiver: None,
        previous: Some(reference.confidence),
    }
}
