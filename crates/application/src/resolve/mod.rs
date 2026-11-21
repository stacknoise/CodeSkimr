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

use codeskimr_codes::ErrorCode;
use codeskimr_domain::{FileId, Reference, ReferenceKind, Span, Symbol, SymbolId};

use crate::cancel::CancelToken;
use crate::error::CodedError;
use crate::ids::IdSequence;
use crate::ports::{PendingReference, ReferenceResolver, Resolution};
pub use ambiguity::ambiguous_occurrences;
use context::Context;
use lookup::Hit;
use table::SymbolTable;

/// The resolver of blueprint 9.2: local scopes, members of the enclosing type and its bases, the
/// declared type of the receiver, imports and namespaces, then the globally unique name.
#[derive(Debug, Clone, Copy)]
pub struct ScopedResolver {
    /// Most symbols a place may reach through the bare name (`scan.max_candidates`).
    max_candidates: usize,
}

impl ScopedResolver {
    /// A resolver that gives up the places whose bare name reaches more than `max_candidates`
    /// symbols (ADR 0084).
    #[must_use]
    pub const fn new(max_candidates: usize) -> Self {
        Self { max_candidates }
    }
}

impl Default for ScopedResolver {
    fn default() -> Self {
        Self::new(consts::DEFAULT_MAX_CANDIDATES)
    }
}

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
        (ids, cancel): (&IdSequence, &CancelToken),
    ) -> Result<Resolution, CodedError> {
        let mut context = Context::new(SymbolTable::new(symbols), self.max_candidates);
        let mut seen: HashSet<Edge> = HashSet::new();
        let mut references = Vec::new();
        let mut given_up = 0;
        for &pass in PASSES {
            for (done, occurrence) in pending
                .iter()
                .filter(|p| Pass::of(p.kind) == pass)
                .enumerate()
            {
                if done % consts::CANCEL_CHECK_INTERVAL == 0 && cancel.is_cancelled() {
                    return Err(CodedError::new(ErrorCode::ScanCancelled));
                }
                let hits = context.lookup(occurrence);
                if hits.given_up {
                    given_up += 1;
                }
                let hits = hits.hits;
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
        Ok(Resolution {
            references,
            given_up,
        })
    }
}

impl Context<'_> {
    /// The hit as it is stored: `None` if it points from production code into a test file (test
    /// helpers are no target of code that ships, however unique their name is, so a library call
    /// of the same name is not taken for the helper; a test file may use them), and a test
    /// reference if the occurrence is in a test file. The hits are dropped after `learn`, which
    /// sees all of them: what a file imports or a type inherits does not depend on the target
    /// being kept.
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
        let into_test_file = self
            .table
            .get(hit.to)
            .is_some_and(|target| test_files.contains(&target.file));
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
