//! Wire format numbers of `scip.proto` and the limits and markers the import uses.

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
/// Field number of `Document.position_encoding`.
pub(crate) const DOCUMENT_POSITION_ENCODING: u32 = 6;
/// Field number of `Occurrence.range`.
pub(crate) const OCCURRENCE_RANGE: u32 = 1;
/// Field number of `Occurrence.symbol`.
pub(crate) const OCCURRENCE_SYMBOL: u32 = 2;
/// Field number of `Occurrence.symbol_roles`.
pub(crate) const OCCURRENCE_ROLES: u32 = 3;

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
