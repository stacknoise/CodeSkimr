//! Level-1 reference resolution (blueprint section 9.2): names to symbols with a confidence.
//!
//! The resolver sees only the global symbol table and the pending names that the analysis
//! reported, never a syntax tree (ADR 0013). It works in three passes, because later passes need
//! what earlier ones found: imports, then inheritance, then every other occurrence.

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
                    if seen.insert(edge_of(occurrence, &hit)) {
                        references.push(reference_of(occurrence, hit, ids));
                    }
                }
            }
        }
        Ok(references)
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
    }
}
