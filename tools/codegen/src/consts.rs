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
/// File extension of Fluent files.
pub const FTL_EXTENSION: &str = "ftl";
/// Language whose files define the key set; every other language is checked against it.
pub const REFERENCE_LANGUAGE: &str = "en";
/// Generated TypeScript i18n keys, relative to the repository root.
pub const I18N_TS_PATH: &str = "web/src/shared/i18n/keys.ts";
/// Marker line at the top of generated i18n files.
pub const GENERATED_NOTICE_I18N: &str =
    "Generated from locales/en/*.ftl by tools/codegen. Do not edit.";
/// Design tokens, relative to the repository root.
pub const TOKENS_PATH: &str = "design/tokens.toml";
/// Generated CSS variables of all themes.
pub const TOKENS_CSS_PATH: &str = "web/src/shared/theme/tokens.css";
/// Generated TypeScript names of themes and tokens.
pub const TOKENS_TS_PATH: &str = "web/src/shared/theme/tokens.ts";
/// Generated script that sets the theme before the first paint.
pub const THEME_INIT_PATH: &str = "web/public/theme-init.js";
/// Marker line at the top of generated token files.
pub const TOKENS_NOTICE: &str = "Generated from design/tokens.toml by tools/codegen. Do not edit.";
/// The class of an error code by its digit: the name in TypeScript and the `Class` variant in
/// Rust (blueprint 11.2). A digit that is not listed is an internal error.
pub const CLASS_NAMES: [(u8, &str, &str); 6] = [
    (1, "invalid_input", "InvalidInput"),
    (2, "not_found", "NotFound"),
    (3, "conflict", "Conflict"),
    (4, "permission", "Permission"),
    (5, "limit", "Limit"),
    (6, "external", "External"),
];
/// The name of every class that is not in [`CLASS_NAMES`], in TypeScript and in Rust.
pub const INTERNAL_CLASS: (&str, &str) = ("internal", "Internal");
/// Folders with production code that can use an error code, relative to the repository root.
pub const SCANNED_DIRS: [&str; 2] = ["crates", "web/src"];
/// Folders that hold no production code: build output, dependencies and tests.
pub const SKIPPED_DIRS: [&str; 4] = ["target", "node_modules", "tests", "e2e"];
/// Parts of a file name that mark a test or a mock.
pub const SKIPPED_FILE_PARTS: [&str; 2] = [".test.", "test-setup"];
/// Extensions of the source files that are scanned for error codes.
pub const SOURCE_EXTENSIONS: [&str; 3] = ["rs", "ts", "tsx"];
/// Text that every generated file carries near its top.
pub const GENERATED_MARKER: &str = "Do not edit.";
/// How production Rust names a code: `ErrorCode::` in front of the variant.
pub const VARIANT_PATH: &str = "ErrorCode::";
