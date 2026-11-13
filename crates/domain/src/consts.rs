//! Fixed texts of the domain model: wire labels of the enumerations and error messages.

/// Message of `ModelError::Empty`.
pub const MESSAGE_EMPTY: &str = "value must not be empty";
/// Message of `ModelError::InvertedSpan`.
pub const MESSAGE_INVERTED_SPAN: &str = "span ends before it starts";
/// Message of `ModelError::UnknownLabel`.
pub const MESSAGE_UNKNOWN_LABEL: &str = "unknown label";
/// Message of `ModelError::NotFinite`.
pub const MESSAGE_NOT_FINITE: &str = "value must be a finite number";

/// Separates the parts of a qualified name.
pub const NAME_SEPARATOR: char = '.';
/// Separator of qualified names in language packs that use paths of modules (Rust).
pub const MODULE_NAME_SEPARATOR: &str = "::";

/// Label of `SymbolKind::Module`.
pub const KIND_MODULE: &str = "module";
/// Label of `SymbolKind::Namespace`.
pub const KIND_NAMESPACE: &str = "namespace";
/// Label of `SymbolKind::Class`.
pub const KIND_CLASS: &str = "class";
/// Label of `SymbolKind::Interface`.
pub const KIND_INTERFACE: &str = "interface";
/// Label of `SymbolKind::Trait`.
pub const KIND_TRAIT: &str = "trait";
/// Label of `SymbolKind::Record`.
pub const KIND_RECORD: &str = "record";
/// Label of `SymbolKind::Struct`.
pub const KIND_STRUCT: &str = "struct";
/// Label of `SymbolKind::Enum`.
pub const KIND_ENUM: &str = "enum";
/// Label of `SymbolKind::Union`.
pub const KIND_UNION: &str = "union";
/// Label of `SymbolKind::Function`.
pub const KIND_FUNCTION: &str = "function";
/// Label of `SymbolKind::Method`.
pub const KIND_METHOD: &str = "method";
/// Label of `SymbolKind::Constructor`.
pub const KIND_CONSTRUCTOR: &str = "constructor";
/// Label of `SymbolKind::Property`.
pub const KIND_PROPERTY: &str = "property";
/// Label of `SymbolKind::Field`.
pub const KIND_FIELD: &str = "field";
/// Label of `SymbolKind::Constant`.
pub const KIND_CONSTANT: &str = "constant";
/// Label of `SymbolKind::Event`.
pub const KIND_EVENT: &str = "event";
/// Label of `SymbolKind::EnumMember`.
pub const KIND_ENUM_MEMBER: &str = "enum_member";
/// Label of `SymbolKind::Parameter`.
pub const KIND_PARAMETER: &str = "parameter";
/// Label of `SymbolKind::TypeAlias`.
pub const KIND_TYPE_ALIAS: &str = "type_alias";

/// Label of `Visibility::Public`.
pub const VISIBILITY_PUBLIC: &str = "public";
/// Label of `Visibility::Protected`.
pub const VISIBILITY_PROTECTED: &str = "protected";
/// Label of `Visibility::Internal`.
pub const VISIBILITY_INTERNAL: &str = "internal";
/// Label of `Visibility::Private`.
pub const VISIBILITY_PRIVATE: &str = "private";

/// Label of `ReferenceKind::Call`.
pub const REFERENCE_CALL: &str = "call";
/// Label of `ReferenceKind::Read`.
pub const REFERENCE_READ: &str = "read";
/// Label of `ReferenceKind::Write`.
pub const REFERENCE_WRITE: &str = "write";
/// Label of `ReferenceKind::TypeUse`.
pub const REFERENCE_TYPE_USE: &str = "type_use";
/// Label of `ReferenceKind::Inherit`.
pub const REFERENCE_INHERIT: &str = "inherit";
/// Label of `ReferenceKind::Implement`.
pub const REFERENCE_IMPLEMENT: &str = "implement";
/// Label of `ReferenceKind::Override`.
pub const REFERENCE_OVERRIDE: &str = "override";
/// Label of `ReferenceKind::Import`.
pub const REFERENCE_IMPORT: &str = "import";
/// Label of `ReferenceKind::Test`.
pub const REFERENCE_TEST: &str = "test";

/// Label of `Confidence::Certain`.
pub const CONFIDENCE_CERTAIN: &str = "certain";
/// Label of `Confidence::Probable`.
pub const CONFIDENCE_PROBABLE: &str = "probable";

/// Label of `MetricKind::Loc`.
pub const METRIC_LOC: &str = "loc";
/// Label of `MetricKind::CyclomaticComplexity`.
pub const METRIC_CC: &str = "cc";
/// Label of `MetricKind::CyclomaticComplexityMax`.
pub const METRIC_CC_MAX: &str = "cc_max";
/// Label of `MetricKind::MaintainabilityIndex`.
pub const METRIC_MI: &str = "mi";
/// Label of `MetricKind::EfferentCoupling`.
pub const METRIC_CE: &str = "ce";
/// Label of `MetricKind::AfferentCoupling`.
pub const METRIC_CA: &str = "ca";
/// Label of `MetricKind::Instability`.
pub const METRIC_INSTABILITY: &str = "instability";

/// Label of `ScanMode::Full`.
pub const SCAN_FULL: &str = "full";
/// Label of `ScanMode::Incremental`.
pub const SCAN_INCREMENTAL: &str = "incremental";

/// Cyclomatic complexity of a body without any decision point.
pub const CC_BASE: u32 = 1;
/// Constant term of the maintainability index formula.
pub const MI_BASE: f64 = 171.0;
/// Weight of the natural logarithm of the Halstead volume.
pub const MI_VOLUME_WEIGHT: f64 = 5.2;
/// Weight of the cyclomatic complexity.
pub const MI_COMPLEXITY_WEIGHT: f64 = 0.23;
/// Weight of the natural logarithm of the lines of code.
pub const MI_LOC_WEIGHT: f64 = 16.2;
/// Upper end of the maintainability index scale; the raw value is rescaled to it.
pub const MI_MAX: f64 = 100.0;
/// Lower end of the maintainability index scale.
pub const MI_MIN: f64 = 0.0;
/// Smallest value the logarithms of the maintainability index are taken of.
pub const MI_LOG_FLOOR: f64 = 1.0;
/// Smallest vocabulary (distinct operators plus operands) for which a Halstead volume exists.
pub const HALSTEAD_MIN_VOCABULARY: f64 = 1.0;
/// Halstead volume of a body without any vocabulary.
pub const HALSTEAD_EMPTY_VOLUME: f64 = 0.0;

/// Message of `ModelError::NotPositive`.
pub const MESSAGE_NOT_POSITIVE: &str = "value must be greater than zero";

/// Offset basis of the 64-bit FNV-1a hash that turns token text into a token id.
pub const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
/// Prime of the 64-bit FNV-1a hash.
pub const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
/// Odd base of the rolling hash over a run of token ids.
pub const ROLLING_BASE: u64 = 0x9e37_79b9_7f4a_7c15;
/// First multiplier of the finalising mix that spreads the bits of a rolling hash.
pub const MIX_MULTIPLIER_FIRST: u64 = 0xbf58_476d_1ce4_e5b9;
/// Second multiplier of the finalising mix.
pub const MIX_MULTIPLIER_SECOND: u64 = 0x94d0_49bb_1331_11eb;
/// Shift of the first xor-shift step of the finalising mix.
pub const MIX_SHIFT_FIRST: u32 = 30;
/// Shift of the second xor-shift step of the finalising mix.
pub const MIX_SHIFT_SECOND: u32 = 27;
/// Shift of the last xor-shift step of the finalising mix.
pub const MIX_SHIFT_LAST: u32 = 31;
/// Text hashed for every identifier, so names do not influence a fingerprint.
pub const TOKEN_IDENTIFIER: &str = "<identifier>";
/// Text hashed for every literal, so values do not influence a fingerprint.
pub const TOKEN_LITERAL: &str = "<literal>";
/// Similarity of two fingerprint sets that share nothing.
pub const SIMILARITY_NONE: f64 = 0.0;
/// Start value of the power `ROLLING_BASE^(n-1)`, the weight of the token that leaves the window.
pub const POWER_START: u64 = 1;
/// Start value of the rolling hash of an n-gram.
pub const HASH_START: u64 = 0;

/// Marker that separates the scheme of a URL from the rest.
pub const URL_SCHEME_MARKER: &str = "://";
/// Character that ends the credentials part of a URL.
pub const URL_USERINFO_END: char = '@';
/// Character that starts the password inside the credentials part of a URL.
pub const URL_PASSWORD_START: char = ':';
/// Characters that end the authority part of a URL. `\` counts like `/` for browsers.
pub const URL_AUTHORITY_END: [char; 4] = ['/', '?', '#', '\\'];
/// Opens an IPv6 host, whose colons are no password separators.
pub const URL_IPV6_START: char = '[';
/// Schemes whose user name can be a secret (a token), so the whole credentials part goes.
pub const SECRET_USER_SCHEMES: [&str; 2] = ["http", "https"];
/// Characters besides letters and digits that a URL scheme may contain.
pub const URL_SCHEME_EXTRA: &str = "+-.";

/// Number of metrics of a measured callable: `loc`, `cc` and `mi`.
pub const CALLABLE_METRICS: usize = 3;
/// Message of `ModelError::UnknownNode`.
pub const MESSAGE_UNKNOWN_NODE: &str = "edge names a node that does not exist";

/// Label of `Permission::Read`.
pub const PERMISSION_READ: &str = "read";
/// Label of `Permission::ReadSource`.
pub const PERMISSION_READ_SOURCE: &str = "read_source";
/// Label of `Permission::Scan`.
pub const PERMISSION_SCAN: &str = "scan";
/// Label of `Permission::Manage`.
pub const PERMISSION_MANAGE: &str = "manage";
/// Label of `Permission::RegisterProject`.
pub const PERMISSION_REGISTER_PROJECT: &str = "register_project";
/// Label of `Permission::Admin`.
pub const PERMISSION_ADMIN: &str = "admin";

/// Whole percent of a share of 1.0, for integer percent arithmetic.
pub const PERCENT_BASE: usize = 100;
