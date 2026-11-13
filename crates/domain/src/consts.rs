//! Fixed texts of the domain model: wire labels of the enumerations and error messages.

/// Message of `ModelError::Empty`.
pub const MESSAGE_EMPTY: &str = "value must not be empty";
/// Message of `ModelError::InvertedSpan`.
pub const MESSAGE_INVERTED_SPAN: &str = "span ends before it starts";
/// Message of `ModelError::UnknownLabel`.
pub const MESSAGE_UNKNOWN_LABEL: &str = "unknown label";
/// Message of `ModelError::NotFinite`.
pub const MESSAGE_NOT_FINITE: &str = "value must be a finite number";

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
