//! Level-1 reference resolution (blueprint section 9.2): names to symbols with a confidence.
//!
//! The resolver sees only the global symbol table and the pending names that the analysis
//! reported, never a syntax tree (ADR 0013). It works in three passes, because later passes need
//! what earlier ones found: imports, then inheritance, then every other occurrence.

mod ambiguity;
mod consts;
mod context;
mod hierarchy;
mod kinds;
mod lookup;
mod table;

use std::collections::HashSet;

use codeskimr_domain::{FileId, Reference, ReferenceKind, Span, Symbol, SymbolId};

use crate::error::CodedError;
use crate::ids::IdSequence;
use crate::ports::{PendingReference, ReferenceResolver};
pub use ambiguity::ambiguous_occurrences;
use context::Context;
use lookup::Hit;
use table::SymbolTable;

/// The resolver of blueprint 9.2: local scopes, members of the enclosing type and its bases, the
/// declared type of the receiver, imports and namespaces, then the globally unique name.
#[derive(Debug, Clone, Copy, Default)]
pub struct ScopedResolver;

/// The order in which occurrences are resolved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pass {
    /// Imports, which decide what a file sees.
    Imports,
    /// Base types, which decide what a type inherits.
    Inheritance,
    /// Everything else.
    Usages,
}

const PASSES: &[Pass] = &[Pass::Imports, Pass::Inheritance, Pass::Usages];

impl Pass {
    const fn of(kind: ReferenceKind) -> Self {
        match kind {
            ReferenceKind::Import => Self::Imports,
            ReferenceKind::Inherit | ReferenceKind::Implement => Self::Inheritance,
            _ => Self::Usages,
        }
    }
}

/// What makes two references the same edge.
type Edge = (SymbolId, FileId, Span, ReferenceKind, SymbolId);

impl ReferenceResolver for ScopedResolver {
    fn resolve(
        &self,
        symbols: &[Symbol],
        pending: &[PendingReference],
        test_files: &HashSet<FileId>,
        ids: &IdSequence,
    ) -> Result<Vec<Reference>, CodedError> {
        let mut context = Context::new(SymbolTable::new(symbols));
        let mut seen: HashSet<Edge> = HashSet::new();
        let mut references = Vec::new();
        for &pass in PASSES {
            for occurrence in pending.iter().filter(|p| Pass::of(p.kind) == pass) {
                let hits = context.lookup(occurrence);
                context.learn(occurrence, &hits);
                for hit in hits {
                    let Some(hit) = context.placed(occurrence, hit, test_files) else {
                        continue;
                    };
                    if seen.insert(edge_of(occurrence, &hit)) {
                        references.push(reference_of(occurrence, hit, ids));
                    }
                }
            }
        }
        Ok(references)
    }
}

impl Context<'_> {
    /// The hit as it is stored: `None` if it points from production code into a test file (test
    /// helpers are no target of code that ships, however unique their name is, so a library call
    /// of the same name is not taken for the helper; a test file may use them, and a file always
    /// sees its own symbols), and a test reference if the occurrence is in a test file.
    fn placed(
        &self,
        occurrence: &PendingReference,
        mut hit: Hit,
        test_files: &HashSet<FileId>,
    ) -> Option<Hit> {
        if test_files.contains(&occurrence.file) {
            hit.kind = kinds::in_test(hit.kind);
            return Some(hit);
        }
        let into_test_file = self.table.get(hit.to).is_some_and(|target| {
            target.file != occurrence.file && test_files.contains(&target.file)
        });
        (!into_test_file).then_some(hit)
    }
}

fn edge_of(occurrence: &PendingReference, hit: &Hit) -> Edge {
    (
        occurrence.from,
        occurrence.file,
        occurrence.span,
        hit.kind,
        hit.to,
    )
}

fn reference_of(occurrence: &PendingReference, hit: Hit, ids: &IdSequence) -> Reference {
    Reference {
        id: ids.reference(),
        from: occurrence.from,
        to: hit.to,
        file: occurrence.file,
        span: occurrence.span,
        kind: hit.kind,
        confidence: hit.confidence,
        verified: false,
    }
}
