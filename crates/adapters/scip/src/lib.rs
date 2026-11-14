//! Level 2 references from SCIP indexes (ADR 0033, `docs/level2-import.md`).
//!
//! [`decode_index`] and [`read_index`] turn the bytes of an index (the Protobuf format of
//! Sourcegraph's `scip.proto`) into a [`ScipIndex`]; the index is untrusted input. The pure
//! function [`import_references`] maps its occurrences onto the symbols of a snapshot and
//! replaces the level 1 references of the files the index covers. [`convert_columns`] turns the columns of
//! UTF-16 or UTF-32 indexes into UTF-8 bytes. The `ScipImporter` adapter is what the scan uses.

mod columns;
mod consts;
mod decode;
mod error;
mod fitting;
mod impl_symbol;
mod importer;
mod mapping;
mod model;
mod path;
mod symbol;
mod table;
mod wire;

pub use columns::convert_columns;
pub use decode::{decode_index, read_index};
pub use error::{Corruption, ScipError};
pub use importer::ScipImporter;
pub use mapping::{ImportOutcome, ImportStats, SnapshotView, import_references};
pub use model::{Occurrence, PositionEncoding, ScipDocument, ScipIndex, SymbolRef};
pub use path::normalize_path;
