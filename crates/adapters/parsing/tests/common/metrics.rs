//! Helpers of the golden tests of `metrics.scm` (ADR 0015), shared by the languages.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use codeskimr_application::ids::IdSequence;
use codeskimr_application::ports::FileFacts;
use codeskimr_domain::{BodyCounts, FolderPath, SymbolCounts};
use codeskimr_parsing::{LanguagePacks, TreeSitterAnalyzer};
use tree_sitter::{Language, Query};

use super::common::analyze_file;

/// The folder with the language packs.
pub fn languages() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../languages")
}

/// Analyzes `languages/<language>/tests/<file>`.
pub fn facts(language: &str, file: &str) -> FileFacts {
    let analyzer = TreeSitterAnalyzer::new(
        LanguagePacks::load(&languages()).expect("packs load"),
        Duration::from_secs(10),
    );
    let tests = languages().join(language).join("tests");
    let root = FolderPath::new(tests.to_string_lossy()).expect("root");
    analyze_file(
        &analyzer,
        &IdSequence::starting_at(1),
        &root,
        file,
        language,
    )
}

/// Counts by the simple name of the symbol.
pub fn by_name(facts: &FileFacts) -> HashMap<String, SymbolCounts> {
    let names: HashMap<_, _> = facts
        .symbols
        .iter()
        .map(|s| (s.id, s.name.to_string()))
        .collect();
    facts
        .counts
        .iter()
        .map(|c| (names[&c.symbol].clone(), *c))
        .collect()
}

/// The body counts of the callable `name`.
pub fn body(counts: &HashMap<String, SymbolCounts>, name: &str) -> BodyCounts {
    counts[name]
        .body
        .unwrap_or_else(|| panic!("{name} has no body counts"))
}

/// Checks that `languages/<language>/highlights.scm` compiles against the grammar.
pub fn highlights_compile(language: &str, grammar: &Language) {
    let path = languages().join(language).join("highlights.scm");
    let text = std::fs::read_to_string(path).expect("query");
    Query::new(grammar, &text).expect("highlights.scm compiles");
}
