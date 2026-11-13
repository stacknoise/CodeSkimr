//! The capture vocabulary of language pack queries, resolved once when a pack loads.

use std::str::FromStr;

use codeskimr_domain::{ReferenceKind, SymbolKind};
use tree_sitter::Query;

use crate::consts::{
    DOC_CAPTURE, MODIFIER_CAPTURE, NAME_CAPTURE, REFERENCE_CAPTURE_PREFIX, SCOPE_END_CAPTURE,
    SIGNATURE_CAPTURE, STATIC_CAPTURE, SYMBOL_CAPTURE_PREFIX, VISIBILITY_CAPTURE,
};

/// What a capture of a query stands for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Capture {
    /// A whole declaration of this kind.
    Symbol(SymbolKind),
    /// An occurrence of this kind.
    Reference(ReferenceKind),
    /// Simple name of a declaration or target of a reference.
    Name,
    /// Visibility keyword; may occur several times, such as `protected` and `internal`.
    Visibility,
    /// Any modifier keyword; the pack decides which ones set visibility or static.
    Modifier,
    /// Marker for static declarations.
    Static,
    /// Documentation comment.
    Doc,
    /// Signature text.
    Signature,
    /// Node whose end is the end of the symbol's scope.
    ScopeEnd,
}

impl Capture {
    /// Interprets a capture name; `None` if it is not part of the vocabulary.
    #[must_use]
    pub fn parse(name: &str) -> Option<Self> {
        if let Some(label) = name.strip_prefix(SYMBOL_CAPTURE_PREFIX) {
            return SymbolKind::from_str(label).ok().map(Self::Symbol);
        }
        if let Some(label) = name.strip_prefix(REFERENCE_CAPTURE_PREFIX) {
            return ReferenceKind::from_str(label).ok().map(Self::Reference);
        }
        match name {
            NAME_CAPTURE => Some(Self::Name),
            VISIBILITY_CAPTURE => Some(Self::Visibility),
            MODIFIER_CAPTURE => Some(Self::Modifier),
            STATIC_CAPTURE => Some(Self::Static),
            DOC_CAPTURE => Some(Self::Doc),
            SIGNATURE_CAPTURE => Some(Self::Signature),
            SCOPE_END_CAPTURE => Some(Self::ScopeEnd),
            _ => None,
        }
    }
}

/// A query together with the meaning of each of its captures.
pub struct Vocabulary {
    /// The compiled query.
    pub query: Query,
    roles: Vec<Capture>,
}

impl Vocabulary {
    /// Resolves every capture name of `query`.
    ///
    /// # Errors
    /// The offending capture name if it is not part of the vocabulary.
    pub fn new(query: Query) -> Result<Self, String> {
        let roles = query
            .capture_names()
            .iter()
            .map(|name| Capture::parse(name).ok_or_else(|| (*name).to_owned()))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { query, roles })
    }

    /// Meaning of the capture with this index.
    #[must_use]
    pub fn role(&self, index: u32) -> Option<Capture> {
        self.roles.get(usize::try_from(index).ok()?).copied()
    }
}
