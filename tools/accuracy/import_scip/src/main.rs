//! Measuring tool for `tools/accuracy/measure.py --scip`: maps a SCIP index onto a database that
//! `codeskimr scan` wrote (level 1) and stores the level 2 references in it.
//!
//! It is not part of the product. It never asks for or changes the consent of a project: it calls
//! the public `codeskimr_scip` library on an index that a person made by hand, and writes only into
//! the throw-away database of the measurement.
//!
//! `import_scip <database> <project id> <index.scip> <min_mapped>`

use std::num::NonZeroU32;
use std::path::Path;
use std::process::ExitCode;

use codeskimr_application::ports::{ProjectRepository, ReferenceReplacement, SnapshotStore};
use codeskimr_domain::{ProjectId, Reference, ReferenceId, SnapshotId};
use codeskimr_scip::{ImportOutcome, SnapshotView, import_references, read_index};
use codeskimr_storage::SqliteStore;

/// Same value as `level2.max_index_bytes` in `config/default.toml`.
const MAX_INDEX_BYTES: u64 = 268_435_456;
/// The arguments, in order: database, project id, index, `min_mapped`.
const ARGUMENTS: usize = 4;

/// Reads the snapshot, maps the index onto it and returns the outcome and the snapshot id.
fn import(
    store: &SqliteStore,
    project: ProjectId,
    index: &Path,
    min_mapped: f64,
) -> Result<(SnapshotId, ImportOutcome), String> {
    let files = store.latest_files(project).map_err(|e| e.to_string())?;
    let previous = store.latest_index(project).map_err(|e| e.to_string())?;
    let snapshot = files.first().map(|file| file.snapshot).ok_or("no files")?;
    let index = read_index(index, MAX_INDEX_BYTES).map_err(|e| e.to_string())?;
    let view = SnapshotView {
        files: &files,
        symbols: &previous.symbols,
        references: &previous.references,
    };
    let outcome = import_references(&index, &view, min_mapped).map_err(|e| e.to_string())?;
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

fn run(args: &[String]) -> Result<String, String> {
    let [database, project, index, min_mapped] = args else {
        return Err(format!("expected {ARGUMENTS} arguments"));
    };
    let project = ProjectId::new(project.parse::<u64>().map_err(|e| e.to_string())?);
    let min_mapped: f64 = min_mapped.parse().map_err(|e| format!("{e}"))?;
    let store =
        SqliteStore::open(Path::new(database), NonZeroU32::MIN).map_err(|e| e.to_string())?;
    let (snapshot, outcome) = import(&store, project, Path::new(index), min_mapped)?;
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
