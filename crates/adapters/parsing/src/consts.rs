//! Fixed values of this crate: file names of a language pack, capture vocabulary, encodings.

use codeskimr_domain::SymbolKind;

/// File with the metadata and rules of a language pack.
pub const MANIFEST_FILE: &str = "language.toml";
/// Tree-sitter query that maps declarations onto symbol kinds.
pub const SYMBOLS_QUERY_FILE: &str = "symbols.scm";
/// Tree-sitter query for identifiers, calls and imports; optional.
pub const REFERENCES_QUERY_FILE: &str = "references.scm";

/// Tree-sitter query with the counts of the metrics (ADR 0015); optional.
pub const METRICS_QUERY_FILE: &str = "metrics.scm";
/// Capture of a comment node in `metrics.scm`; line kinds come from `comment_nodes` of the pack.
pub const COMMENT_CAPTURE: &str = "comment";
/// Capture of a decision point in `metrics.scm`.
pub const DECISION_CAPTURE: &str = "decision";
/// Capture of an operator token in `metrics.scm`.
pub const OPERATOR_CAPTURE: &str = "operator";
/// Capture of an operand (identifier or literal) in `metrics.scm`.
pub const OPERAND_CAPTURE: &str = "operand";
/// Field name that holds the body of a callable declaration in the grammar; a callable without it
/// (abstract, interface member, extern) is not measured.
pub const BODY_FIELD: &str = "body";

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
/// Capture of a block that is no symbol but belongs to the type its `@name` capture names, such as
/// a Rust `impl` block (ADR 0037).
pub const CONTAINER_CAPTURE: &str = "container";
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
/// Grammar name of TypeScript in `language.toml`.
pub const GRAMMAR_TYPESCRIPT: &str = "typescript";
/// Grammar name of TSX (TypeScript with JSX) in `language.toml`.
pub const GRAMMAR_TSX: &str = "tsx";
/// Grammar name of JavaScript (with JSX) in `language.toml`.
pub const GRAMMAR_JAVASCRIPT: &str = "javascript";
/// Grammar name of Python in `language.toml`.
pub const GRAMMAR_PYTHON: &str = "python";
/// Grammar name of Java in `language.toml`.
pub const GRAMMAR_JAVA: &str = "java";
/// Grammar name of Kotlin in `language.toml`.
pub const GRAMMAR_KOTLIN: &str = "kotlin";

/// Grammar name of Go in `language.toml` (basic tier).
#[cfg(feature = "basic-grammars")]
pub const GRAMMAR_GO: &str = "go";
/// Grammar name of C in `language.toml` (basic tier).
#[cfg(feature = "basic-grammars")]
pub const GRAMMAR_C: &str = "c";
/// Grammar name of C++ in `language.toml` (basic tier).
#[cfg(feature = "basic-grammars")]
pub const GRAMMAR_CPP: &str = "cpp";
/// Grammar name of Ruby in `language.toml` (basic tier).
#[cfg(feature = "basic-grammars")]
pub const GRAMMAR_RUBY: &str = "ruby";
/// Grammar name of PHP in `language.toml` (basic tier).
#[cfg(feature = "basic-grammars")]
pub const GRAMMAR_PHP: &str = "php";

/// Start of the reason when the `id` of a pack differs from its folder name.
pub const ID_MISMATCH: &str = "id differs from the folder name: ";
/// Reason of an invalid pack whose `[[tools]]` entry is wrong; the entry follows.
pub const TOOL_INVALID: &str = "invalid tool: ";

/// Replacement extension that removes the extension of a path.
pub const NO_EXTENSION: &str = "";

/// Symbol kinds whose bodies get syntax fingerprints.
pub const CALLABLE_KINDS: [SymbolKind; 3] = [
    SymbolKind::Function,
    SymbolKind::Method,
    SymbolKind::Constructor,
];

/// Reason of the error when the thread that loads a pack panics.
pub const LOAD_PANICKED: &str = "loading the pack panicked";

/// Label of the full tier, as the language table shows it.
pub const TIER_FULL: &str = "full";
/// Label of the basic tier.
pub const TIER_BASIC: &str = "basic";

/// Reason: the program of an `[indexer]` is empty or a path.
pub const INDEXER_BAD_PROGRAM: &str = "indexer program must be a bare name: ";
/// Reason: a version of an `[indexer]` cannot be read.
pub const INDEXER_BAD_VERSION: &str = "indexer version cannot be read: ";
/// Reason: an argument of an `[indexer]` holds a placeholder that does not exist.
pub const INDEXER_UNKNOWN_PLACEHOLDER: &str = "indexer argument has an unknown placeholder: ";
/// Reason: no argument of an `[indexer]` names the index file.
pub const INDEXER_NO_OUTPUT: &str = "indexer args must contain {output}";
/// Opens a placeholder in an indexer argument.
pub const PLACEHOLDER_OPEN: char = '{';
/// Closes a placeholder in an indexer argument.
pub const PLACEHOLDER_CLOSE: char = '}';
/// Characters that make a program name a path.
pub const PATH_SEPARATORS: [char; 2] = ['/', '\\'];
/// Nothing: what a placeholder is replaced with when the rest of an argument is checked.
pub const NOTHING: &str = "";

/// Between a language and its rule in the text of the naming rules.
pub const NAMING_RULE_SEPARATOR: &str = "=";
/// Between the rules of two languages in the text of the naming rules.
pub const NAMING_RULES_SEPARATOR: &str = ";";
/// Rule of a pack whose qualified names start with the file.
pub const NAMING_FILE_IN_NAMES: &str = "file";
/// Rule of a pack whose qualified names start with its own namespaces (ADR 0036).
pub const NAMING_NAMESPACED: &str = "namespace";

/// Revision of the naming rules of the engine itself, part of the text of the naming rules: it
/// changes when names change for reasons other than a pack key (2: members of an `impl` carry
/// their type, ADR 0037).
pub const NAMING_REVISION: &str = "revision=2";

/// Ends a signature that was cut at `scan.max_signature_chars`.
pub const SIGNATURE_CUT: &str = "…";
