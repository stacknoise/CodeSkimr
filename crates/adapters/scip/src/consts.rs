//! Wire format numbers of `scip.proto` and the limits and markers the import uses.

use codeskimr_domain::Position;

/// Field number of `Index.metadata`.
pub(crate) const INDEX_METADATA: u32 = 1;
/// Field number of `Index.documents`.
pub(crate) const INDEX_DOCUMENTS: u32 = 2;
/// Field number of `Metadata.project_root`.
pub(crate) const METADATA_PROJECT_ROOT: u32 = 3;
/// Field number of `Document.relative_path`.
pub(crate) const DOCUMENT_RELATIVE_PATH: u32 = 1;
/// Field number of `Document.occurrences`.
pub(crate) const DOCUMENT_OCCURRENCES: u32 = 2;
/// Field number of `Document.symbols` (a `SymbolInformation` each).
pub(crate) const DOCUMENT_SYMBOLS: u32 = 3;
/// Field number of `Document.position_encoding`.
pub(crate) const DOCUMENT_POSITION_ENCODING: u32 = 6;
/// Field number of `Occurrence.range`.
pub(crate) const OCCURRENCE_RANGE: u32 = 1;
/// Field number of `Occurrence.symbol`.
pub(crate) const OCCURRENCE_SYMBOL: u32 = 2;
/// Field number of `Occurrence.symbol_roles`.
pub(crate) const OCCURRENCE_ROLES: u32 = 3;

/// Field number of `Occurrence.enclosing_range`: the range of the whole definition (a macro call
/// for generated code), in the same numbers as `range`.
pub(crate) const OCCURRENCE_ENCLOSING_RANGE: u32 = 7;
/// Field number of `SymbolInformation.symbol`.
pub(crate) const INFO_SYMBOL: u32 = 1;
/// Field number of `SymbolInformation.documentation` (repeated).
pub(crate) const INFO_DOCUMENTATION: u32 = 3;
/// Field number of `SymbolInformation.kind`.
pub(crate) const INFO_KIND: u32 = 5;
/// Field number of `SymbolInformation.display_name`.
pub(crate) const INFO_DISPLAY_NAME: u32 = 6;
/// Field number of `SymbolInformation.signature_documentation` (a `Document`).
pub(crate) const INFO_SIGNATURE: u32 = 7;
/// Field number of `Document.text`, which holds the signature in `signature_documentation`.
pub(crate) const DOCUMENT_TEXT: u32 = 5;
/// Longest accepted display name, in bytes; a longer one leaves the symbol without information.
pub(crate) const MAX_INFO_NAME_BYTES: usize = 1024;
/// Longest accepted signature, in bytes.
pub(crate) const MAX_INFO_SIGNATURE_BYTES: usize = 4096;
/// Longest accepted documentation (all parts together), in bytes.
pub(crate) const MAX_INFO_DOC_BYTES: usize = 16 * 1024;
/// Most bytes of information texts kept for all symbols of an index; past it no further symbol
/// information is kept, so such a symbol creates nothing (nothing is guessed).
pub(crate) const MAX_INFO_TOTAL_BYTES: usize = 32 * 1024 * 1024;
/// Joins the parts of the documentation of a symbol.
pub(crate) const DOC_JOIN: &str = "\n\n";
/// The SCIP kinds (`SymbolInformation.Kind`) that the packs may name in `[indexer.symbol_kinds]`,
/// by their value in `scip.proto`. Checked against the indexes of rust-analyzer (November 2026);
/// a kind that is not here has no name and creates nothing.
pub(crate) const SCIP_KIND_NAMES: [(u32, &str); 10] = [
    (3, "AssociatedType"),
    (8, "Constant"),
    (11, "Enum"),
    (12, "EnumMember"),
    (15, "Field"),
    (17, "Function"),
    (26, "Method"),
    (49, "Struct"),
    (53, "Trait"),
    (55, "TypeAlias"),
];

/// Enum value `PositionEncoding.UTF8CodeUnitOffsetFromLineStart`.
pub(crate) const ENCODING_UTF8: u64 = 1;
/// Enum value `PositionEncoding.UTF16CodeUnitOffsetFromLineStart`.
pub(crate) const ENCODING_UTF16: u64 = 2;
/// Enum value `PositionEncoding.UTF32CodeUnitOffsetFromLineStart`.
pub(crate) const ENCODING_UTF32: u64 = 3;

/// Bit of `symbol_roles` that marks a definition (`SymbolRole.Definition`).
pub(crate) const ROLE_DEFINITION: u32 = 1;

/// Wire type: variable-length integer.
pub(crate) const WIRE_VARINT: u8 = 0;
/// Wire type: eight fixed bytes.
pub(crate) const WIRE_FIXED64: u8 = 1;
/// Wire type: bytes with a length in front.
pub(crate) const WIRE_LEN: u8 = 2;
/// Wire type: start of a group (deprecated, only skipped).
pub(crate) const WIRE_START_GROUP: u8 = 3;
/// Wire type: end of a group.
pub(crate) const WIRE_END_GROUP: u8 = 4;
/// Wire type: four fixed bytes.
pub(crate) const WIRE_FIXED32: u8 = 5;
/// Number of low tag bits that hold the wire type.
pub(crate) const TAG_WIRE_BITS: u32 = 3;
/// Mask of the wire type inside a tag.
pub(crate) const TAG_WIRE_MASK: u64 = 7;
/// Payload bits of one varint byte.
pub(crate) const VARINT_PAYLOAD_BITS: u32 = 7;
/// Mask of the payload bits of one varint byte.
pub(crate) const VARINT_PAYLOAD_MASK: u8 = 0x7f;
/// Continuation bit of a varint byte.
pub(crate) const VARINT_CONTINUE: u8 = 0x80;
/// Most bytes a varint may have.
pub(crate) const VARINT_MAX_BYTES: u32 = 10;
/// Size of a fixed 32 bit value.
pub(crate) const FIXED32_BYTES: usize = 4;
/// Size of a fixed 64 bit value.
pub(crate) const FIXED64_BYTES: usize = 8;

/// Deepest nesting of skipped groups; deeper input is refused instead of followed.
pub(crate) const MAX_GROUP_DEPTH: u32 = 8;
/// Longest accepted relative path, in bytes.
pub(crate) const MAX_PATH_BYTES: usize = 4096;
/// Longest accepted SCIP symbol, in bytes.
pub(crate) const MAX_SYMBOL_BYTES: usize = 65_536;
/// Most numbers a range has: start line, start column, end line, end column (three if it ends on its start line).
pub(crate) const RANGE_MULTI_LINE: usize = 4;

/// Separators of a relative path; both are accepted and written back as `/`.
pub(crate) const PATH_SEPARATORS: [char; 2] = ['/', '\\'];
/// The separator of normalised paths.
pub(crate) const PATH_JOIN: &str = "/";
/// Path component that stays in place.
pub(crate) const PATH_CURRENT: &str = ".";
/// Path component that leaves the folder.
pub(crate) const PATH_PARENT: &str = "..";
/// Marks a drive letter such as `C:`.
pub(crate) const DRIVE_MARK: char = ':';
/// A path component of this length with a drive mark is a drive.
pub(crate) const DRIVE_COMPONENT_CHARS: usize = 2;
/// A byte that never belongs in a path.
pub(crate) const PATH_FORBIDDEN: char = '\0';

/// Prefix of symbols that are local to one document.
pub(crate) const LOCAL_PREFIX: &str = "local ";
/// Separator of the four header fields of a symbol; two in a row stand for one inside a field.
pub(crate) const SYMBOL_SEPARATOR: char = ' ';
/// Number of header fields before the descriptors (scheme, manager, package, version).
pub(crate) const SYMBOL_HEADER_FIELDS: usize = 4;
/// Quote of an escaped descriptor name; doubled inside the name.
pub(crate) const NAME_QUOTE: char = '`';
/// Characters besides letters and digits that an unquoted descriptor name may hold.
pub(crate) const NAME_EXTRA_CHARS: [char; 3] = ['_', '+', '-'];
/// Another character of unquoted descriptor names.
pub(crate) const NAME_DOLLAR: char = '$';
/// Suffix of a namespace descriptor.
pub(crate) const SUFFIX_NAMESPACE: char = '/';
/// Suffix of a type descriptor.
pub(crate) const SUFFIX_TYPE: char = '#';
/// Suffix of a term descriptor.
pub(crate) const SUFFIX_TERM: char = '.';
/// Suffix of a meta descriptor.
pub(crate) const SUFFIX_META: char = ':';
/// Suffix of a macro descriptor.
pub(crate) const SUFFIX_MACRO: char = '!';
/// Opens the disambiguator of a method and the name of a parameter.
pub(crate) const OPEN_PAREN: char = '(';
/// Closes the disambiguator of a method and the name of a parameter.
pub(crate) const CLOSE_PAREN: char = ')';
/// Opens the name of a type parameter.
pub(crate) const OPEN_BRACKET: char = '[';
/// Closes the name of a type parameter.
pub(crate) const CLOSE_BRACKET: char = ']';
/// How rust-analyzer starts the symbol of an impl block: `impl#[Type]` for the block itself (what
/// `Self` names), `impl#[Type][Trait]` for a trait implementation.
pub(crate) const IMPL_MARKER: &str = "impl#[";
/// Opens the generic arguments of a type or trait name that an indexer writes in one piece.
pub(crate) const OPEN_GENERIC: char = '<';
/// Separates the segments of a Rust path.
pub(crate) const PATH_SEPARATOR: &str = "::";
/// Most symbols kept per name: a second one already makes the answer ambiguous.
pub(crate) const MAX_SYMBOLS_PER_KEY: usize = 2;
/// Most modules of a path that are looked up; a longer path maps nothing.
pub(crate) const MAX_PATH_DEPTH: usize = 8;
/// The owner of a symbol that has none (a type is looked up without one).
pub(crate) const NO_OWNER: &str = "";
/// How scip-typescript prefixes the name of a getter and a setter; the project has one property.
pub(crate) const ACCESSOR_PREFIXES: [&str; 2] = ["<get>", "<set>"];
/// Longest definition, in bytes, whose text is read as the name of a local symbol.
pub(crate) const MAX_LOCAL_NAME_BYTES: u32 = 256;
/// Most bytes of name texts kept for all local symbols of an index; past it no new text is kept
/// (the symbol stays without a name, nothing is guessed).
pub(crate) const MAX_LOCAL_NAME_TOTAL_BYTES: usize = 16 * 1024 * 1024;
/// Characters besides letters and digits that may occur in the name of a local symbol.
pub(crate) const IDENTIFIER_EXTRA_CHARS: [char; 2] = ['_', '$'];
/// Most targets one symbol gets by adopting overload heads; more is not an overload set.
pub(crate) const MAX_OVERLOAD_TARGETS: usize = 64;
/// Most same-named symbols of one line that are weighed up to find the one declared at a place.
pub(crate) const MAX_LINE_CANDIDATES: usize = 16;
/// The first segment of a path that starts at the crate root.
pub(crate) const ROOT_SEGMENT: &str = "crate";
/// Segments of a path that are relative to the module of the block; they are not resolved.
pub(crate) const RELATIVE_SEGMENTS: [&str; 2] = ["self", "super"];
/// Separates the segments of a qualified name written as one escaped descriptor.
pub(crate) const NAME_SEGMENT_SEPARATOR: char = '.';
/// Names the indexers give constructors; a project constructor is named like its type.
pub(crate) const CONSTRUCTOR_NAMES: [&str; 3] = [".ctor", "<init>", "<constructor>"];

/// Nesting depth of a group right after its start tag was read.
pub(crate) const GROUP_OPENED_DEPTH: u32 = 1;

/// Most documents an index may hold.
pub(crate) const MAX_DOCUMENTS: usize = 1_000_000;
/// Most occurrences, over all documents, an index may hold (24 bytes each in memory).
pub(crate) const MAX_OCCURRENCES: usize = 16_000_000;
/// Most distinct symbols an index may hold.
pub(crate) const MAX_SYMBOLS: usize = 4_000_000;

/// The first line of a file when a range of lines is asked for.
pub(crate) const FIRST_LINE: u32 = 1;

/// Extension of the files whose text is read for what the indexer leaves out (Java).
pub(crate) const JAVA_EXTENSION: &str = ".java";
/// The word that starts an import declaration.
pub(crate) const IMPORT_KEYWORD: &str = "import";
/// Ends a declaration or a statement.
pub(crate) const SEMICOLON: char = ';';
/// The word before the type of an object creation.
pub(crate) const NEW_KEYWORD: &str = "new";
/// The words that start an explicit call of another constructor.
pub(crate) const CONSTRUCTOR_CALL_KEYWORDS: [&str; 2] = ["this", "super"];
/// The word that starts the list of exceptions of a method or constructor.
pub(crate) const THROWS_KEYWORD: &str = "throws";
/// Marks the last parameter of a method as a variable number of arguments.
pub(crate) const VARARGS_MARK: &str = "...";
/// Starts a comment up to the end of the line.
pub(crate) const LINE_COMMENT: &str = "//";
/// Starts a block comment.
pub(crate) const BLOCK_COMMENT_OPEN: &str = "/*";
/// Ends a block comment.
pub(crate) const BLOCK_COMMENT_CLOSE: &str = "*/";
/// Starts a comment (first character of both comment marks).
pub(crate) const SLASH: char = '/';
/// Separates parameters.
pub(crate) const COMMA: char = ',';
/// Opens a body.
pub(crate) const OPEN_BRACE: char = '{';
/// Closes a list of type arguments.
pub(crate) const CLOSE_ANGLE: char = '>';
/// Quotes of strings and of characters.
pub(crate) const LITERAL_QUOTES: [char; 2] = ['"', '\''];
/// Escapes the next character of a string.
pub(crate) const LITERAL_ESCAPE: char = '\\';
/// Most characters one scan of the text of a file looks at: a hostile file cannot make the
/// import read on without end.
pub(crate) const MAX_SCAN_CHARS: usize = 8192;
/// Characters that may be part of a Java identifier besides letters and digits.
pub(crate) const JAVA_IDENTIFIER_EXTRA: [char; 2] = ['_', '$'];
/// The end of a line, which the scan of the text of a file reads as a character.
pub(crate) const NEWLINE: char = '\n';
/// What opens a group inside a list of parameters.
pub(crate) const GROUP_OPENERS: [char; 4] = ['(', '[', '{', '<'];
/// What closes a group inside a list of parameters.
pub(crate) const GROUP_CLOSERS: [char; 4] = [')', ']', '}', '>'];
/// Starts and ends a Java text block.
pub(crate) const TEXT_BLOCK_MARK: &str = "\"\"\"";
/// Ends the qualified call of the constructor of an inner superclass (`outer.super(...)`).
pub(crate) const QUALIFIED_SUPER: &str = ".super";
/// Separates the parts of a qualified name.
pub(crate) const PATH_DOT: char = '.';
/// How many characters all scans of a file may read together, as a multiple of its size (never
/// less than `MAX_SCAN_CHARS`): a normal file needs a small part of it, a file built to make the
/// scans overlap is not read on.
pub(crate) const SCAN_BUDGET_FACTOR: usize = 16;
/// The separator of a pack that creates nothing.
pub(crate) const NO_SEPARATOR: &str = "";
/// The start of a file: the range of a file root (the definition of a module that is a whole file)
/// begins here (ADR 0061).
pub(crate) const FILE_START: Position = Position { line: 0, column: 0 };
/// Most roots one symbol keeps. A symbol with more definitions of that shape is not a package with
/// a library and a build script, and every use of it would have to be weighed against all of them.
pub(crate) const MAX_ROOTS_PER_SYMBOL: usize = 64;
/// The folder of a file in the root of the project.
pub(crate) const ROOT_FOLDER: &str = "";
/// The library root of a pack that names none: the folders alone decide between several roots.
pub(crate) const NO_LIBRARY_ROOT: &str = "";
