//! Measuring tool for `tools/accuracy/measure.py --scip`: maps a SCIP index onto a database that
//! `codeskimr scan` wrote (level 1) and stores the level 2 references in it.
//!
//! It is not part of the product. It never asks for or changes the consent of a project: it calls
//! the public `codeskimr_scip` library on an index that a person made by hand, and writes only into
//! the throw-away database of the measurement.
//!
//! `import_scip <database> <project id> <index.scip> <min_mapped> <source folder> <max_index_bytes> <max_file_bytes>`
//!
//! Like the product (`ScipImporter`), it converts the columns of documents that are not UTF-8 with
//! the lines of the files in the source folder before the mapping. The limits come from the caller
//! (`measure.py` reads them from `config/default.toml`).

use std::collections::HashSet;
use std::num::NonZeroU32;
use std::path::Path;
use std::process::ExitCode;

use codeskimr_application::ports::{
    ProjectRepository, ReferenceReplacement, SnapshotStore, SourceText,
};
use codeskimr_domain::{FolderPath, ProjectId, Reference, ReferenceId, RelativePath, SnapshotId};
use codeskimr_scip::{ImportOutcome, SnapshotView, convert_columns, import_references, read_index};
use codeskimr_source::FolderText;
use codeskimr_storage::SqliteStore;

/// The arguments, in order: database, project id, index, `min_mapped`, source folder,
/// `max_index_bytes`, `max_file_bytes`.
const ARGUMENTS: usize = 7;
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
    let outcome = import_references(&index, &view, job.min_mapped).map_err(|e| e.to_string())?;
    Ok((snapshot, outcome))
}

/// Stores the references of the covered files with fresh ids.
fn replace(
    store: &SqliteStore,
    snapshot: SnapshotId,
    outcome: ImportOutcome,
) -> Result<(), String> {
    let first = store.next_free_id().map_err(|e| e.to_string())?;
    let references: Vec<Reference> = outcome
        .references
        .iter()
        .filter(|reference| outcome.covered.binary_search(&reference.file).is_ok())
        .zip(first..)
        .map(|(reference, id)| Reference {
            id: ReferenceId::new(id),
            ..*reference
        })
        .collect();
    let replacement = ReferenceReplacement {
        snapshot,
        files: outcome.covered,
        references,
    };
    store
        .replace_references(replacement)
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn number<T: std::str::FromStr>(text: &str) -> Result<T, String> {
    text.parse().map_err(|_| format!("not a number: {text}"))
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
    };
    let store =
        SqliteStore::open(Path::new(database), NonZeroU32::MIN).map_err(|e| e.to_string())?;
    let (snapshot, outcome) = import(&store, &job)?;
    let stats = outcome.stats;
    replace(&store, snapshot, outcome)?;
    Ok(format!(
        "{{\"mapped\": {}, \"library\": {}, \"unmapped\": {}, \"ignored\": {}, \"files_replaced\": {}, \"files_unmatched\": {}}}",
        stats.mapped,
        stats.library,
        stats.unmapped,
        stats.ignored,
        stats.files_replaced,
        stats.files_unmatched
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
