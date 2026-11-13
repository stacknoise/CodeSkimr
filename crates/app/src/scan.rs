//! `codeskimr scan`: registers the source as a project and runs one full scan.

use std::time::Instant;

use codeskimr_application::error::CodedError;
use codeskimr_application::events::EventBus;
use codeskimr_application::pipeline::{Job, JobResult};
use codeskimr_cli::{GlobalOptions, ScanArgs, ScanSummary};
use codeskimr_codes::ErrorCode;
use codeskimr_domain::{Project, ProjectSource, ScanMode};
use codeskimr_storage::SqliteStore;

use crate::config_files;
use crate::locations::Locations;
use crate::source_arg::{SourceSpec, classify};
use crate::wiring::{build_scanner, open_store};

/// Checks the source argument before anything is scanned or printed.
///
/// # Errors
/// `PRJ-101` if the argument is neither a folder nor a Git URL without credentials.
pub fn prepare(args: &ScanArgs) -> Result<SourceSpec, CodedError> {
    classify(&args.source, args.branch.as_deref())
}

/// Runs the scan and reports what it found.
///
/// The mode is always full: the incremental mode is not complete yet (issues #20 and #29).
///
/// # Errors
/// Any coded failure of the configuration, the source, the parsing or the storage.
pub fn run(
    spec: SourceSpec,
    global: &GlobalOptions,
    locations: &Locations,
) -> Result<ScanSummary, CodedError> {
    let project_dir = match &spec.source {
        ProjectSource::Folder(folder) => Some(std::path::Path::new(folder.as_str())),
        ProjectSource::Git { .. } => None,
    };
    let config = config_files::load(locations, global, project_dir)?;
    let store = open_store(locations, &config)?;
    let project = register(&store, spec)?;
    let scanner = build_scanner(&store, &config, locations, EventBus::new())?;
    let started = Instant::now();
    match scanner.run(Job::new(project.id, ScanMode::Full)) {
        JobResult::Finished(job) => {
            let snapshot = job.snapshot();
            Ok(ScanSummary {
                project: project.name.to_string(),
                snapshot: snapshot.id.get(),
                files: snapshot.stats.files,
                symbols: snapshot.stats.symbols,
                references: snapshot.stats.references,
                seconds: started.elapsed().as_secs_f64(),
            })
        }
        JobResult::Failed(job) => Err(CodedError::new(job.code())),
        JobResult::Cancelled(_) => Err(CodedError::new(ErrorCode::UnexpectedInternalError)),
    }
}

/// The project of this source: the registered one, or a new one.
fn register(store: &SqliteStore, spec: SourceSpec) -> Result<Project, CodedError> {
    if let Some(known) = store.project_by_source(&spec.source)? {
        return Ok(known);
    }
    let project = Project {
        id: store.next_project_id()?,
        name: spec.name,
        source: spec.source,
        layers: Vec::new(),
    };
    store.save_project(&project)?;
    Ok(project)
}
