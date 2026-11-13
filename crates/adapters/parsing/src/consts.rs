//! Fixed values of this crate: file names of a language pack, capture vocabulary, encodings.

use codeskimr_domain::SymbolKind;

/// File with the metadata and rules of a language pack.
pub const MANIFEST_FILE: &str = "language.toml";
/// Tree-sitter query that maps declarations onto symbol kinds.
pub const SYMBOLS_QUERY_FILE: &str = "symbols.scm";
/// Tree-sitter query for identifiers, calls and imports; optional.
pub const REFERENCES_QUERY_FILE: &str = "references.scm";

/// Prefix of a capture that marks a whole declaration; the rest is the symbol kind label.
pub const SYMBOL_CAPTURE_PREFIX: &str = "symbol.";
/// Prefix of a capture that marks an occurrence; the rest is the reference kind label.
pub const REFERENCE_CAPTURE_PREFIX: &str = "reference.";
/// Capture of the simple name of a declaration or of the target of a reference.
pub const NAME_CAPTURE: &str = "name";
/// Capture of the keyword that sets the visibility.
pub const VISIBILITY_CAPTURE: &str = "visibility";
/// Capture of any modifier keyword (`public`, `static`, `async` …); may occur several times.
pub const MODIFIER_CAPTURE: &str = "modifier";
/// Capture whose presence marks a static declaration.
pub const STATIC_CAPTURE: &str = "static";
/// Capture of a documentation comment; may occur several times.
pub const DOC_CAPTURE: &str = "doc";
/// Capture of the signature text.
pub const SIGNATURE_CAPTURE: &str = "signature";
/// Capture of the node whose end closes the scope of a declaration that does not enclose its
/// members itself, such as a file-scoped namespace.
pub const SCOPE_END_CAPTURE: &str = "scope_end";
/// Capture of a receiver that is a name: a variable, a field or a type.
pub const RECEIVER_CAPTURE: &str = "receiver";
/// Capture of the type name of a receiver that is a creation expression.
pub const RECEIVER_TYPE_CAPTURE: &str = "receiver.type";
/// Capture of a receiver that is the enclosing object itself.
pub const RECEIVER_OWN_CAPTURE: &str = "receiver.own";
/// Capture of a receiver that is the base of the enclosing object.
pub const RECEIVER_BASE_CAPTURE: &str = "receiver.base";
/// Capture of any other receiver; the syntax says nothing more about it.
pub const RECEIVER_OTHER_CAPTURE: &str = "receiver.other";
/// Capture of the name of a local variable or parameter that hides members of the same name.
pub const LOCAL_CAPTURE: &str = "local";
/// Capture of the name of a field or property whose declared type tells the type of a receiver.
pub const FIELD_CAPTURE: &str = "field";
/// Capture of the type name written in a declaration or created by its initializer.
pub const DECLARED_TYPE_CAPTURE: &str = "declared_type";
/// Capture of a node that opens a scope for the names declared in it.
pub const SCOPE_CAPTURE: &str = "scope";
/// Separator between visibility keywords that together name one visibility.
pub const KEYWORD_SEPARATOR: &str = " ";

/// Separator between the lines of a joined documentation comment.
pub const LINE_SEPARATOR: &str = "\n";
/// Separator between the words of a collapsed documentation summary.
pub const WORD_SEPARATOR: &str = " ";
/// Separator of folders in a path inside the project.
pub const PATH_SEPARATOR: char = '/';
/// Byte order mark of UTF-8 files.
pub const UTF8_BOM: &[u8] = &[0xEF, 0xBB, 0xBF];
/// Byte order mark of little-endian UTF-16 files.
pub const UTF16_LE_BOM: &[u8] = &[0xFF, 0xFE];
/// Byte order mark of big-endian UTF-16 files.
pub const UTF16_BE_BOM: &[u8] = &[0xFE, 0xFF];
/// Bytes per UTF-16 code unit.
pub const UTF16_UNIT_BYTES: usize = 2;

/// Grammar name of Rust in `language.toml`.
pub const GRAMMAR_RUST: &str = "rust";
/// Grammar name of C# in `language.toml`.
pub const GRAMMAR_CSHARP: &str = "csharp";

/// Start of the reason when the `id` of a pack differs from its folder name.
pub const ID_MISMATCH: &str = "id differs from the folder name: ";

/// Replacement extension that removes the extension of a path.
pub const NO_EXTENSION: &str = "";

/// Symbol kinds whose bodies get syntax fingerprints.
pub const CALLABLE_KINDS: [SymbolKind; 3] = [
    SymbolKind::Function,
    SymbolKind::Method,
    SymbolKind::Constructor,
];
