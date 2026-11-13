//! Occurrences of a symbol and how sure the resolver is about them.

use crate::consts::{
    CONFIDENCE_CERTAIN, CONFIDENCE_PROBABLE, REFERENCE_CALL, REFERENCE_IMPLEMENT, REFERENCE_IMPORT,
    REFERENCE_INHERIT, REFERENCE_OVERRIDE, REFERENCE_READ, REFERENCE_TEST, REFERENCE_TYPE_USE,
    REFERENCE_WRITE,
};
use crate::file::FileId;
use crate::span::Span;
use crate::symbol::SymbolId;
use crate::vocabulary::{id_type, labeled_enum};

id_type!(
    /// Identifies a reference within a snapshot.
    ReferenceId
);

labeled_enum!(
    /// How a symbol is used at a reference.
    ReferenceKind {
        /// Call of a function or method.
        Call => REFERENCE_CALL,
        /// Read access.
        Read => REFERENCE_READ,
        /// Write access.
        Write => REFERENCE_WRITE,
        /// Use as a type.
        TypeUse => REFERENCE_TYPE_USE,
        /// Inheritance from a type.
        Inherit => REFERENCE_INHERIT,
        /// Implementation of an interface or trait.
        Implement => REFERENCE_IMPLEMENT,
        /// Override of a member.
        Override => REFERENCE_OVERRIDE,
        /// Import or using directive.
        Import => REFERENCE_IMPORT,
        /// Use from test code.
        Test => REFERENCE_TEST,
    }
);

labeled_enum!(
    /// How sure the resolver is that a reference points at the right symbol.
    /// UI and MCP always show it; `Probable` is drawn visibly different (blueprint 9).
    Confidence {
        /// The name resolved unambiguously.
        Certain => CONFIDENCE_CERTAIN,
        /// The resolver chose the most likely candidate.
        Probable => CONFIDENCE_PROBABLE,
    }
);

impl Confidence {
    /// Whether the reference must be shown as uncertain.
    #[must_use]
    pub const fn is_probable(self) -> bool {
        matches!(self, Self::Probable)
    }
}

/// One occurrence of a symbol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reference {
    /// Identity.
    pub id: ReferenceId,
    /// Symbol that contains the occurrence.
    pub from: SymbolId,
    /// Symbol that is referred to.
    pub to: SymbolId,
    /// File of the occurrence.
    pub file: FileId,
    /// Range of the occurrence.
    pub span: Span,
    /// Kind of use.
    pub kind: ReferenceKind,
    /// Confidence of the resolution.
    pub confidence: Confidence,
}
