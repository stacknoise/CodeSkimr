//! What the decoder keeps of an index: the project root, the documents and their occurrences.

use std::collections::HashMap;
use std::sync::Arc;

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

/// What the index states about a symbol it defines in a document (`SymbolInformation`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolInfo {
    /// The symbol.
    pub symbol: SymbolRef,
    /// `SymbolInformation.kind` as the number of `scip.proto`; 0 if the index does not say.
    pub kind: u32,
    /// The name the indexer shows; empty if it does not say.
    pub display_name: String,
    /// The signature of the definition (`signature_documentation.text`), if stated.
    pub signature: Option<String>,
    /// The documentation, its parts joined by a blank line, if stated.
    pub documentation: Option<String>,
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
    /// What the index states about the symbols this document defines.
    pub infos: Vec<SymbolInfo>,
    /// For a definition (by its range), the range of the whole definition (`enclosing_range`):
    /// a macro call for code that a macro generates.
    pub extents: HashMap<Span, Span>,
    /// What the local symbols that the document defines are called, read from the text of their
    /// definitions by `convert_columns`. Empty while the text is not known.
    pub local_names: HashMap<SymbolRef, Arc<str>>,
    /// What the text of the file adds to the index (Java); empty while the text is not known.
    pub facts: TextFacts,
}

/// How many parameters a constructor declares.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arity {
    /// None.
    Zero,
    /// One, which may take a variable number of arguments (`String... names`).
    One {
        /// Whether the parameter is a varargs parameter.
        varargs: bool,
    },
    /// Two or more.
    Several,
}

/// A constructor that a document defines, with what only its text tells.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConstructorSite {
    /// The symbol of the constructor.
    pub symbol: SymbolRef,
    /// Whether the source declares it; the default constructor of a class without one is only
    /// defined by the index.
    pub declared: bool,
    /// Its parameters.
    pub arity: Arity,
    /// Where the compiler puts the call of the constructor of the superclass that the source does
    /// not write: the opening brace of a declared constructor, the name of the class for a default
    /// constructor.
    pub call_at: Span,
    /// Whether the body starts with `this(...)` or `super(...)`, so that no call is implied.
    pub explicit_call: bool,
}

/// `new Type(...) { ... }`: the creation of an anonymous subclass.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnonymousCreation {
    /// The symbol of the type that is created.
    pub symbol: SymbolRef,
    /// Where the name of the type stands.
    pub range: Span,
    /// The opening brace of the body, where the compiler puts the call of the constructor.
    pub body: Span,
}

/// What the text of a file adds to the index: calls that the compiler counts and the indexer
/// does not write.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TextFacts {
    /// The constructors the document defines.
    pub constructors: Vec<ConstructorSite>,
    /// The anonymous subclasses the document creates.
    pub anonymous: Vec<AnonymousCreation>,
    /// The names of the enum constants that have a body (`NAME(args) { ... }`); the compiler
    /// reads each of them as the creation of a subclass of the enum.
    pub enum_bodies: Vec<Span>,
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
