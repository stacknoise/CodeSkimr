//! What the decoder keeps of an index: the project root, the documents and their occurrences.

use codeskimr_domain::{RelativePath, Span};

use crate::consts::ROLE_DEFINITION;

/// Handle of a distinct SCIP symbol string of one index. Occurrences hold this instead of the
/// text, so a symbol that occurs ten thousand times is stored once.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SymbolRef(pub(crate) u32);

/// Unit of the columns of a document (`PositionEncoding` of `scip.proto`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PositionEncoding {
    /// Not stated. Some indexers that leave it out count UTF-16 code units. Columns are never
    /// converted, so the same caveat as for `Utf16` applies.
    #[default]
    Unspecified,
    /// UTF-8 bytes from the start of the line, as the parsing adapter counts them.
    Utf8,
    /// UTF-16 code units from the start of the line. They equal UTF-8 bytes only on lines
    /// without characters beyond ASCII before the column; the import does not convert them.
    Utf16,
    /// UTF-32 code units (characters) from the start of the line; not converted either.
    Utf32,
}

/// One place where a symbol occurs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Occurrence {
    /// Where the occurrence is, zero-based, in the encoding of its document.
    pub range: Span,
    /// The symbol that occurs.
    pub symbol: SymbolRef,
    /// Bit set of `SymbolRole`.
    pub roles: u32,
}

impl Occurrence {
    /// Whether the occurrence is the definition of its symbol (bit `Definition`).
    #[must_use]
    pub const fn is_definition(&self) -> bool {
        self.roles & ROLE_DEFINITION != 0
    }
}

/// One file of the index with its occurrences.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScipDocument {
    /// Path relative to the project root: forward slashes, no `..`, not absolute.
    pub path: RelativePath,
    /// Unit of the columns of the occurrences.
    pub encoding: PositionEncoding,
    /// Occurrences in the order of the index.
    pub occurrences: Vec<Occurrence>,
}

/// A decoded SCIP index, reduced to what the import needs.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ScipIndex {
    /// `Metadata.project_root`, a URI as the indexer wrote it. The import matches documents by
    /// relative path and does not interpret it.
    pub project_root: String,
    /// The documents.
    pub documents: Vec<ScipDocument>,
    pub(crate) symbols: Vec<String>,
}

impl ScipIndex {
    /// The text of a symbol, or `None` for a handle that does not belong to this index.
    #[must_use]
    pub fn symbol(&self, symbol: SymbolRef) -> Option<&str> {
        let position = usize::try_from(symbol.0).ok()?;
        self.symbols.get(position).map(String::as_str)
    }
}
