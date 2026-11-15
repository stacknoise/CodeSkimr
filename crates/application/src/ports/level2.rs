//! Turning the index of an external indexer into references (ADR 0033; adapter: `codeskimr_scip`).

use std::path::Path;

use codeskimr_domain::{FileId, FolderPath, Reference, SourceFile, Symbol, SymbolKind};

use crate::error::CodedError;

/// What the importer reads: the index file and the snapshot it is mapped onto.
#[derive(Debug, Clone, Copy)]
pub struct ImportRequest<'a> {
    /// The index file the runner wrote.
    pub index: &'a Path,
    /// The folder of the project; the importer reads the lines of the files from there to convert
    /// the columns of the index.
    pub root: &'a FolderPath,
    /// The files whose hash is still the one of the snapshot, of the languages of the indexer.
    pub files: &'a [SourceFile],
    /// All symbols of the snapshot.
    pub symbols: &'a [Symbol],
    /// All references of the snapshot, level 1 so far.
    pub references: &'a [Reference],
    /// Largest index file that is read, in bytes.
    pub max_index_bytes: u64,
    /// Largest source file whose lines are read to convert columns, in bytes.
    pub max_file_bytes: u64,
    /// Smallest share of mapped uses among the uses of project symbols, from 0 to 1.
    pub min_mapped: f64,
    /// What the index says a definition is, by the name of its SCIP kind (`SymbolInformation.kind`),
    /// from the language pack. A kind that is not here creates no symbol, and a language without
    /// any table creates none at all (ADR 0055, decision 7).
    pub symbol_kinds: &'a [(String, SymbolKind)],
    /// The separator of qualified names of the language.
    pub separator: &'a str,
    /// Most symbols the import may create; an index that wants more fails as a whole.
    pub max_created_symbols: usize,
}

/// The references of the files that an index covers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Imported {
    /// The files whose level 1 references are replaced.
    pub files: Vec<FileId>,
    /// The symbols of origin `index` that the import creates in those files; their ids are not
    /// final, they are consecutive from the lowest. The references may point at them.
    pub symbols: Vec<Symbol>,
    /// The level 2 references of those files; their ids are not final.
    pub references: Vec<Reference>,
    /// Uses that became references.
    pub mapped: u64,
    /// Uses that were dropped: library symbols and symbols the snapshot does not know.
    pub dropped: u64,
}

/// Reads an index and maps it onto a snapshot.
pub trait ReferenceImporter: Send + Sync {
    /// Maps the index of `request` onto its files.
    ///
    /// # Errors
    /// `SCIP` failures: the index is invalid or too large, or too few of its uses could be
    /// mapped. The caller keeps level 1 then.
    fn import(&self, request: &ImportRequest<'_>) -> Result<Imported, CodedError>;
}
