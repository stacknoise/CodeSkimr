//! Splits the uses of namespaces by cause (ADR 0061), so that the split of the Rust uses that once
//! had no level 1 symbol is repeatable: of the 659 uses of the measurement of ADR 0055, 448 were
//! uses of a crate root, 131 of a module in a `mod.rs` file, 24 of a module in a folder the scan
//! skips and 56 of another item there. It reads only what the public library gives: the index, the
//! files and symbols of the snapshot, and the outcome of the import.

use std::collections::{HashMap, HashSet};

use codeskimr_domain::{FileId, Position, SourceFile, Symbol, SymbolId};
use codeskimr_scip::{ImportOutcome, ScipDocument, ScipIndex, SymbolRef};

/// The text that marks a local symbol of SCIP.
const LOCAL_PREFIX: &str = "local ";
/// The suffix of a namespace descriptor.
const NAMESPACE_SUFFIX: char = '/';
/// The characters that end a descriptor name from the left.
const NAME_START: [char; 2] = [' ', '/'];
/// The name of the root of a crate in the index of rust-analyzer.
const CRATE_ROOT_NAME: &str = "crate";

/// The occurrences that are uses, counted by what they needed.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Causes {
    /// Uses of the root of a crate (`crate/`), mapped onto the module of a file by position.
    pub crate_root: u64,
    /// Uses of a module that lives in a file of another name (`ports/` in `ports/mod.rs`), mapped
    /// onto the module of that file by position.
    pub module_file: u64,
    /// Uses of a module or crate root that the index defines only in files the scan skips.
    pub skipped_namespace: u64,
    /// Uses of another item (type, function, constant) that the index defines only in such files.
    pub skipped_item: u64,
}

/// The name of the last descriptor of a namespace symbol (`.../ports/` is `ports`).
fn namespace_name(text: &str) -> Option<&str> {
    let body = text.strip_suffix(NAMESPACE_SUFFIX)?;
    body.rsplit(NAME_START).next()
}

/// The documents that match a file of the snapshot, with that file.
fn covered_documents<'a>(
    index: &'a ScipIndex,
    files: &[SourceFile],
) -> Vec<(&'a ScipDocument, FileId)> {
    let by_path: HashMap<&str, FileId> = files
        .iter()
        .map(|file| (file.path.as_str(), file.id))
        .collect();
    index
        .documents
        .iter()
        .filter_map(|document| Some((document, *by_path.get(document.path.as_str())?)))
        .collect()
}

/// The targets of the references of the outcome, by file and place.
fn targets_by_place(outcome: &ImportOutcome) -> HashMap<(FileId, Position), Vec<SymbolId>> {
    let mut targets: HashMap<(FileId, Position), Vec<SymbolId>> = HashMap::new();
    for reference in &outcome.references {
        targets
            .entry((reference.file, reference.span.start()))
            .or_default()
            .push(reference.to);
    }
    targets
}

/// What the import did with the uses of the covered documents, by cause.
pub fn causes(
    index: &ScipIndex,
    files: &[SourceFile],
    symbols: &[Symbol],
    outcome: &ImportOutcome,
) -> Causes {
    let covered = covered_documents(index, files);
    let defined = defined_in(index.documents.iter());
    let scanned = defined_in(covered.iter().map(|(document, _)| *document));
    let names: HashMap<SymbolId, &str> = symbols
        .iter()
        .map(|symbol| (symbol.id, symbol.name.as_str()))
        .collect();
    let targets = targets_by_place(outcome);
    let mut found = Causes::default();
    for (document, file) in covered {
        for occurrence in document.occurrences.iter().filter(|o| !o.is_definition()) {
            let Some(text) = index.symbol(occurrence.symbol) else {
                continue;
            };
            if text.starts_with(LOCAL_PREFIX) || !defined.contains(&occurrence.symbol) {
                continue;
            }
            let namespace = namespace_name(text);
            if scanned.contains(&occurrence.symbol) {
                let place = (file, occurrence.range.start());
                let by_position = namespace.is_some_and(|name| {
                    targets
                        .get(&place)
                        .is_some_and(|ids| ids.iter().any(|id| names.get(id) != Some(&name)))
                });
                match namespace {
                    Some(CRATE_ROOT_NAME) if by_position => found.crate_root += 1,
                    Some(_) if by_position => found.module_file += 1,
                    _ => {}
                }
            } else if namespace.is_some() {
                found.skipped_namespace += 1;
            } else {
                found.skipped_item += 1;
            }
        }
    }
    found
}

/// The symbols that some document of `documents` defines.
fn defined_in<'a>(documents: impl Iterator<Item = &'a ScipDocument>) -> HashSet<SymbolRef> {
    documents
        .flat_map(|document| document.occurrences.iter())
        .filter(|occurrence| occurrence.is_definition())
        .map(|occurrence| occurrence.symbol)
        .collect()
}
