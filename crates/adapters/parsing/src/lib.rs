//! Tree-sitter parsing and the language-pack interpreter.
//!
//! [`TreeSitterAnalyzer`] implements the `FileAnalyzer` port: it parses one file, runs the queries
//! of the matching language pack (blueprint 8.1) and returns symbols, unresolved occurrences and
//! the lines of code. The syntax tree stays inside this crate (ADR 0013).
//!
//! # Query vocabulary
//!
//! `symbols.scm` marks a whole declaration with `@symbol.<kind>` (a symbol kind label such as
//! `class` or `enum_member`) and its simple name with `@name`. Optional captures: `@visibility`,
//! `@static`, `@doc` (repeatable) and `@signature`. `references.scm` marks an occurrence with
//! `@reference.<kind>` (a reference kind label); the target is the `@name` capture if present,
//! otherwise the marked node.
//!
//! Further captures of `symbols.scm`: `@modifier` (repeatable; every modifier keyword, the pack
//! decides in `language.toml` which ones set visibility, `static_keywords` or a kind through
//! `modifier_kinds`) and `@scope_end` (the node whose end closes the scope of a declaration that
//! does not enclose its members, such as a file-scoped namespace). Several matches of one
//! declaration are merged, so a query may repeat captures freely. `language.toml` may set
//! `doc_summary` tags to keep only the summary of a documentation comment and
//! `visibility.parent_defaults` for the visibility of declarations without a keyword.

mod analyzer;
mod captures;
mod consts;
mod encoding;
mod error;
mod fingerprints;
mod grammar;
mod lines;
mod location;
mod manifest;
mod metrics;
mod pack;
mod packs;
mod references;
mod scopes;
mod symbols;
mod visibility;

pub use analyzer::TreeSitterAnalyzer;
pub use error::ParsingError;
pub use manifest::Tier;
pub use pack::LanguagePack;
pub use packs::{LanguagePacks, PackSummary};
