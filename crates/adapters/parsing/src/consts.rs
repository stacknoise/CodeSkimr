//! Fixed values of this crate: file names of a language pack, capture vocabulary, encodings.

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
/// Capture whose presence marks a static declaration.
pub const STATIC_CAPTURE: &str = "static";
/// Capture of a documentation comment; may occur several times.
pub const DOC_CAPTURE: &str = "doc";
/// Capture of the signature text.
pub const SIGNATURE_CAPTURE: &str = "signature";

/// Separator between the lines of a joined documentation comment.
pub const LINE_SEPARATOR: &str = "\n";
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
