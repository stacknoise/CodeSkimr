//! `codeskimr scan`: registers the source as a project and runs one full scan. The [`Session`] is
//! what watch mode scans again and again.

use std::time::Instant;

use codeskimr_application::access;
use codeskimr_application::config::Config;
use codeskimr_application::error::CodedError;
use codeskimr_application::events::EventBus;
use codeskimr_application::pipeline::{Job, JobResult, Scanner};
use codeskimr_application::ports::ProjectRepository;
use codeskimr_cli::{GlobalOptions, ScanArgs, ScanSummary};
use codeskimr_codes::ErrorCode;
use codeskimr_domain::{Permission, Principal, Project, ProjectId, ProjectSource, ScanMode};
use codeskimr_storage::SqliteStore;

use crate::config_files;
use crate::locations::Locations;
use crate::principal;
use crate::source_arg::{SourceError, SourceSpec, classify};
use crate::wiring::{build_scanner, open_store};

/// Checks the source argument before anything is scanned or printed.
///
/// # Errors
/// [`SourceError`] (`PRJ-101`) if the argument is neither a folder nor a Git URL without secrets.
pub fn prepare(args: &ScanArgs) -> Result<SourceSpec, SourceError> {
    classify(&args.source, args.branch.as_deref())
}

/// A registered project with everything needed to scan it again and again.
pub struct Session {
    scanner: Scanner,
    principal: Principal,
    project: Project,
    config: Config,
    events: EventBus,
}

impl Session {
    /// The project that is scanned.
    #[must_use]
    pub const fn project(&self) -> &Project {
        &self.project
    }

    /// The configuration of this run.
    #[must_use]
    pub const fn config(&self) -> &Config {
        &self.config
    }

    /// The bus the scanner publishes on; whoever subscribes sees every scan of this session.
    #[must_use]
    pub const fn events(&self) -> &EventBus {
        &self.events
    }

    /// Runs one scan in `mode` and reports what it found.
    ///
    /// # Errors
    /// The code of the failed stage, or `APP-901` if the scan was cancelled (nothing cancels it here).
    pub fn scan(&self, mode: ScanMode) -> Result<ScanSummary, CodedError> {
        let started = Instant::now();
        match self
            .scanner
            .run(&self.principal, Job::new(self.project.id, mode))
        {
            JobResult::Finished(job) => {
                let snapshot = job.snapshot();
                Ok(ScanSummary {
                    project: self.project.name.to_string(),
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
}

/// Registers the source as a project and wires everything a scan needs.
///
/// # Errors
/// Any coded failure of the permission check, the configuration, the storage or the language packs.
pub fn open(
    spec: SourceSpec,
    global: &GlobalOptions,
    locations: &Locations,
) -> Result<Session, CodedError> {
    open_on(spec, global, locations, EventBus::new())
}

/// Like [`open`], but the scanner publishes on `events`, so that whoever listens to that bus (the
/// server's WebSocket) sees the progress.
///
/// # Errors
/// The same as [`open`].
pub fn open_on(
    spec: SourceSpec,
    global: &GlobalOptions,
    locations: &Locations,
    events: EventBus,
) -> Result<Session, CodedError> {
    let principal = principal::resolve();
    access::require(&principal, Permission::RegisterProject)?;
    access::require(&principal, Permission::Scan)?;
    let config = config_files::load(locations, global, folder_of(&spec.source))?;
    let store = open_store(locations, &config)?;
    let project = register(&store, spec)?;
    session(project, config, &store, locations, events)
}

/// The session of a project that is already registered.
///
/// # Errors
/// `PRJ-201` if there is no such project, otherwise the same as [`open`].
pub fn reopen(
    project: ProjectId,
    global: &GlobalOptions,
    locations: &Locations,
    events: EventBus,
) -> Result<Session, CodedError> {
    let defaults = config_files::load(locations, global, None)?;
    let store = open_store(locations, &defaults)?;
    let project = store.project(project)?;
    let config = config_files::load(locations, global, folder_of(&project.source))?;
    session(project, config, &store, locations, events)
}

fn folder_of(source: &ProjectSource) -> Option<&std::path::Path> {
    match source {
        ProjectSource::Folder(folder) => Some(std::path::Path::new(folder.as_str())),
        ProjectSource::Git { .. } => None,
    }
}

fn session(
    project: Project,
    config: Config,
    store: &std::sync::Arc<SqliteStore>,
    locations: &Locations,
    events: EventBus,
) -> Result<Session, CodedError> {
    let principal = principal::resolve();
    access::require(&principal, Permission::Scan)?;
    let scanner = build_scanner(store, &config, locations, events.clone())?;
    Ok(Session {
        scanner,
        principal,
        project,
        config,
        events,
    })
}

/// Registers the source and runs one full scan.
///
/// The mode is always full: this is the first scan of a run. Watch mode continues from here with
/// incremental scans (`watch.rs`).
///
/// # Errors
/// Any coded failure of [`open`] or of the scan.
pub fn run(
    spec: SourceSpec,
    global: &GlobalOptions,
    locations: &Locations,
) -> Result<ScanSummary, CodedError> {
    open(spec, global, locations)?.scan(ScanMode::Full)
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
