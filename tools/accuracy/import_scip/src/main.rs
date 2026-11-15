//! Measuring tool for `tools/accuracy/measure.py --scip`: maps a SCIP index onto a database that
//! `codeskimr scan` wrote (level 1) and stores the level 2 references in it.
//!
//! It is not part of the product. It never asks for or changes the consent of a project: it calls
//! the public `codeskimr_scip` library on an index that a person made by hand, and writes only into
//! the throw-away database of the measurement.
//!
//! `import_scip <database> <project id> <index.scip> <min_mapped> <source folder> <max_index_bytes> <max_file_bytes> <max_created_symbols> <separator> [SCIP kind=symbol kind ...]`
//!
//! Like the product (`ScipImporter`), it converts the columns of documents that are not UTF-8 with
//! the lines of the files in the source folder before the mapping. The limits come from the caller
//! (`measure.py` reads them from `config/default.toml`), and so does the table of the kinds that create
//! symbols (ADR 0055) from the language pack; without a pair nothing is created.

use std::collections::HashSet;
use std::num::NonZeroU32;
use std::path::Path;
use std::process::ExitCode;

use codeskimr_application::level2::renumbered;
use codeskimr_application::ports::{
    Level2Replacement, ProjectRepository, SnapshotStore, SourceText,
};
use codeskimr_domain::{
    FolderPath, ProjectId, RelativePath, SnapshotId, SymbolKind,
};
use codeskimr_scip::{
    Creation, ImportOutcome, SnapshotView, convert_columns, import_with_creation, read_index,
};
use codeskimr_source::FolderText;
use codeskimr_storage::SqliteStore;

/// The arguments, in order: database, project id, index, `min_mapped`, source folder,
/// `max_index_bytes`, `max_file_bytes`.
const ARGUMENTS: usize = 9;
/// Lines of a file from the first one to the last.
const ALL_LINES: (u32, u32) = (1, u32::MAX);

/// What the caller asks for.
struct Job<'a> {
    project: ProjectId,
    index: &'a Path,
    min_mapped: f64,
    root: &'a str,
    max_index_bytes: u64,
    max_file_bytes: u64,
    max_created: usize,
    separator: &'a str,
    kinds: Vec<(String, SymbolKind)>,
}

/// The lines of a file below the source folder, as the product reads them for the conversion.
fn lines(job: &Job<'_>, path: &RelativePath) -> Option<Vec<String>> {
    let root = FolderPath::new(job.root).ok()?;
    FolderText
        .excerpt(&root, path, ALL_LINES, (u32::MAX, job.max_file_bytes))
        .ok()
        .map(|excerpt| excerpt.lines)
}

/// Reads the snapshot, maps the index onto it and returns the outcome and the snapshot id.
fn import(store: &SqliteStore, job: &Job<'_>) -> Result<(SnapshotId, ImportOutcome), String> {
    let files = store.latest_files(job.project).map_err(|e| e.to_string())?;
    let previous = store.latest_index(job.project).map_err(|e| e.to_string())?;
    let snapshot = files.first().map(|file| file.snapshot).ok_or("no files")?;
    let mut index = read_index(job.index, job.max_index_bytes).map_err(|e| e.to_string())?;
    let known: HashSet<&RelativePath> = files.iter().map(|file| &file.path).collect();
    convert_columns(&mut index, &mut |path| {
        known.contains(path).then(|| lines(job, path)).flatten()
    });
    let view = SnapshotView {
        files: &files,
        symbols: &previous.symbols,
        references: &previous.references,
    };
    let creation = Creation {
        kinds: &job.kinds,
        separator: job.separator,
        max_symbols: job.max_created,
    };
    let outcome = import_with_creation(&index, &view, job.min_mapped, &creation)
        .map_err(|e| e.to_string())?;
    Ok((snapshot, outcome))
}

/// Stores the references of the covered files with fresh ids.
fn replace(
    store: &SqliteStore,
    snapshot: SnapshotId,
    outcome: ImportOutcome,
) -> Result<(), String> {
    let first = store.next_free_id().map_err(|e| e.to_string())?;
    let symbol_count = u64::try_from(outcome.symbols.len()).map_err(|e| e.to_string())?;
    let symbol_first = first;
    let first = first.saturating_add(symbol_count);
    let covered = &outcome.covered;
    let (symbols, references) = renumbered(
        &outcome.symbols,
        outcome
            .references
            .iter()
            .filter(|reference| covered.binary_search(&reference.file).is_ok()),
        symbol_first,
        first,
    );
    let replacement = Level2Replacement {
        snapshot,
        files: outcome.covered,
        symbols,
        references,
    };
    store
        .replace_level2(replacement)
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn number<T: std::str::FromStr>(text: &str) -> Result<T, String> {
    text.parse().map_err(|_| format!("not a number: {text}"))
}

/// `SCIP kind=symbol kind` pairs.
fn kinds(pairs: &[String]) -> Result<Vec<(String, SymbolKind)>, String> {
    pairs
        .iter()
        .map(|pair| {
            let (scip, kind) = pair.split_once('=').ok_or(format!("not a pair: {pair}"))?;
            let kind = kind
                .parse::<SymbolKind>()
                .map_err(|_| format!("not a kind: {kind}"))?;
            Ok((scip.to_owned(), kind))
        })
        .collect()
}

fn run(args: &[String]) -> Result<String, String> {
    let [
        database,
        project,
        index,
        min_mapped,
        root,
        max_index,
        max_file,
        max_created,
        separator,
        pairs @ ..,
    ] = args
    else {
        return Err(format!("expected {ARGUMENTS} arguments"));
    };
    let job = Job {
        project: ProjectId::new(number(project)?),
        index: Path::new(index),
        min_mapped: number(min_mapped)?,
        root,
        max_index_bytes: number(max_index)?,
        max_file_bytes: number(max_file)?,
        max_created: number(max_created)?,
        separator,
        kinds: kinds(pairs)?,
    };
    let store =
        SqliteStore::open(Path::new(database), NonZeroU32::MIN).map_err(|e| e.to_string())?;
    let (snapshot, outcome) = import(&store, &job)?;
    let stats = outcome.stats;
    let created = outcome.symbols.len();
    replace(&store, snapshot, outcome)?;
    Ok(format!(
        "{{\"mapped\": {}, \"library\": {}, \"unmapped\": {}, \"ignored\": {}, \"files_replaced\": {}, \"files_unmatched\": {}, \"created\": {}}}",
        stats.mapped,
        stats.library,
        stats.unmapped,
        stats.ignored,
        stats.files_replaced,
        stats.files_unmatched,
        created
    ))
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(line) => {
            println!("{line}");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}
