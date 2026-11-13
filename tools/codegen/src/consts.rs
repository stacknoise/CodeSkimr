//! Fixed names and paths used by the code generator.

/// Registry file, relative to the repository root.
pub const REGISTRY_PATH: &str = "errors/error-codes.toml";
/// Folder with one sub-folder per language, relative to the repository root.
pub const LOCALES_DIR: &str = "locales";
/// File inside each language folder that holds the error messages.
pub const ERRORS_FTL: &str = "errors.ftl";
/// Prefix of Fluent message keys for error codes.
pub const MESSAGE_KEY_PREFIX: &str = "error-";
/// Generated TypeScript file, relative to the repository root.
pub const TS_PATH: &str = "web/src/shared/api/error-codes.generated.ts";
/// Generated Markdown catalogue, relative to the repository root.
pub const DOCS_PATH: &str = "docs/error-codes.md";
/// Separator between prefix and number in an error code id.
pub const ID_SEPARATOR: char = '-';
/// Command-line flag: verify instead of write.
pub const CHECK_FLAG: &str = "--check";
/// Smallest and largest prefix length.
pub const PREFIX_LEN: std::ops::RangeInclusive<usize> = 2..=4;
/// Allowed class digits (blueprint 11.2).
pub const CLASS_DIGITS: [u8; 7] = [1, 2, 3, 4, 5, 6, 9];
/// Marker line at the top of every generated file.
pub const GENERATED_NOTICE: &str =
    "Generated from errors/error-codes.toml by tools/codegen. Do not edit.";
