//! `codeskimr scan`: registers the source as a project and runs one full scan. The [`Session`] is
//! what watch mode scans again and again.

use std::sync::Arc;
use std::time::Instant;

use codeskimr_application::access;
use codeskimr_application::cancel::CancelToken;
use codeskimr_application::config::Config;
use codeskimr_application::error::CodedError;
use codeskimr_application::events::EventBus;
use codeskimr_application::level2::{LanguageReport, Level2Outcome, Skipped};
use codeskimr_application::pipeline::{Job, JobResult, Queued, Scanner};
use codeskimr_application::ports::{ProjectRepository, ProjectToolCheck};
use codeskimr_application::tools::ToolsView;
use codeskimr_cli::{GlobalOptions, Level2Line, Level2Reason, ScanArgs, ScanSummary};
use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_domain::{Permission, Principal, Project, ProjectId, ProjectSource, ScanMode};
use codeskimr_storage::SqliteStore;

use crate::config_files;
use crate::consts::LOG_TOOL_CHECK_FAILED;
use crate::locations::Locations;
use crate::principal;
use crate::source_arg::{SourceError, SourceSpec, classify};
use crate::wiring::{Secrets, build_scanner, index_runner, open_store, tool_checker};

/// Checks the source argument before anything is scanned or printed.
///
/// # Errors
/// [`SourceError`] (`PRJ-101`) if the argument is neither a folder nor a Git URL without secrets,
/// or if a credential is named for a folder.
pub fn prepare(args: &ScanArgs) -> Result<SourceSpec, SourceError> {
    let spec = classify(&args.source, args.branch.as_deref())?;
    match &args.credential {
        Some(name) => spec.with_credential(name),
        None => Ok(spec),
    }
}

/// A registered project with everything needed to scan it again and again.
pub struct Session {
    scanner: Scanner,
    tools: Arc<dyn ProjectToolCheck>,
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

    /// Checks the tools with `tools` instead of the ones this session was wired with.
    #[must_use]
    pub fn with_tools(mut self, tools: Arc<dyn ProjectToolCheck>) -> Self {
        self.tools = tools;
        self
    }

    /// Checks the tools of the languages the new snapshot holds (ADR 0033, decision 8). A failure
    /// of the check never fails the scan: it is logged and the summary says nothing was checked.
    /// An incremental scan does not check (the tools do not change when a file is saved, and
    /// watch mode times the rescan, blueprint 16.1), the stored report of the last full scan stays.
    fn check_tools(&self, mode: ScanMode) -> ToolsView {
        if mode == ScanMode::Incremental {
            return ToolsView::unchecked();
        }
        match self.tools.check(self.project.id) {
            Ok(report) => ToolsView::of(&report),
            Err(error) => {
                tracing::warn!(event = LOG_TOOL_CHECK_FAILED, code = error.code().id());
                ToolsView::unchecked()
            }
        }
    }

    /// Runs one scan in `mode` and reports what it found, with what the check of the tools found.
    ///
    /// # Errors
    /// The code of the failed stage, or `PRJ-303` if the scan was cancelled (nothing cancels it
    /// here).
    pub fn scan(&self, mode: ScanMode) -> Result<ScanSummary, CodedError> {
        self.run_job(Job::new(self.project.id, mode).with_level2())
    }

    /// Like [`Session::scan`], but level 1 only: no indexer starts. Watch mode scans again with
    /// this after its first scan.
    ///
    /// # Errors
    /// The same as [`Session::scan`].
    pub fn scan_quick(&self, mode: ScanMode) -> Result<ScanSummary, CodedError> {
        self.run_job(Job::new(self.project.id, mode).without_level2())
    }

    /// Like [`Session::scan`], but level 1 only, and it hands the token that cancels this scan to
    /// `on_token` before the scan starts. The UI uses it; only the scan of the command line runs
    /// level 2.
    ///
    /// # Errors
    /// The same as [`Session::scan`].
    pub fn scan_with(
        &self,
        mode: ScanMode,
        on_token: impl FnOnce(CancelToken),
    ) -> Result<ScanSummary, CodedError> {
        let job = Job::new(self.project.id, mode);
        on_token(job.cancel_token());
        self.run_job(job)
    }

    fn run_job(&self, job: Job<Queued>) -> Result<ScanSummary, CodedError> {
        let started = Instant::now();
        let mode = job.mode();
        match self.scanner.run(&self.principal, job) {
            JobResult::Finished(job) => {
                let snapshot = job.snapshot();
                let seconds = started.elapsed().as_secs_f64();
                Ok(ScanSummary {
                    project: self.project.name.to_string(),
                    snapshot: snapshot.id.get(),
                    files: snapshot.stats.files,
                    symbols: snapshot.stats.symbols,
                    references: snapshot.stats.references,
                    seconds,
                    level2: job.level2().iter().map(level2_line).collect(),
                    tools: self.check_tools(mode),
                })
            }
            JobResult::Failed(job) => Err(CodedError::new(job.code())),
            JobResult::Cancelled(_) => Err(CodedError::new(ErrorCode::ScanCancelled)),
        }
    }
}

/// The line of the summary for one language of the second pass.
fn level2_line(report: &LanguageReport) -> Level2Line {
    let (files, stale, reason) = match &report.outcome {
        Level2Outcome::Checked { files, stale, .. } => (*files, *stale, None),
        Level2Outcome::Skipped(why) => (u64::default(), u64::default(), Some(reason_of(why))),
    };
    Level2Line {
        language: report.language.to_string(),
        files,
        stale,
        reason,
    }
}

fn reason_of(skipped: &Skipped) -> Level2Reason {
    match skipped {
        Skipped::NotPermitted => Level2Reason::NotPermitted,
        Skipped::NotConfirmed => Level2Reason::NotConfirmed,
        Skipped::Unavailable(_) => Level2Reason::Unavailable,
        Skipped::Stale => Level2Reason::Stale,
        Skipped::Cancelled => Level2Reason::Cancelled,
        Skipped::Failed(code) => Level2Reason::Failed(code.id().to_owned()),
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
    secrets: &Arc<Secrets>,
) -> Result<Session, CodedError> {
    open_on(
        principal::resolve(),
        spec,
        global,
        locations,
        (EventBus::new(), secrets),
    )
}

/// Like [`open`], but the scanner publishes on `events`, so that whoever listens to that bus (the
/// server's WebSocket) sees the progress.
///
/// # Errors
/// The same as [`open`].
pub fn open_on(
    principal: Principal,
    spec: SourceSpec,
    global: &GlobalOptions,
    locations: &Locations,
    (events, secrets): (EventBus, &Arc<Secrets>),
) -> Result<Session, CodedError> {
    access::require(&principal, Permission::RegisterProject)?;
    access::require(&principal, Permission::Scan)?;
    let config = config_files::load(locations, global, folder_of(&spec.source))?;
    let store = open_store(locations, &config)?;
    let project = with_layers(&store, register(&store, spec)?, &config)?;
    session(
        principal,
        project,
        config,
        Wiring {
            store: &store,
            locations,
            events,
            secrets,
        },
    )
}

/// The session of a project that is already registered.
///
/// # Errors
/// `PRJ-201` if there is no such project, otherwise the same as [`open`].
pub fn reopen(
    principal: Principal,
    project: ProjectId,
    global: &GlobalOptions,
    locations: &Locations,
    (events, secrets): (EventBus, &Arc<Secrets>),
) -> Result<Session, CodedError> {
    let defaults = config_files::load(locations, global, None)?;
    let store = open_store(locations, &defaults)?;
    let project = store.project(project)?;
    let config = config_files::load(locations, global, folder_of(&project.source))?;
    let project = with_layers(&store, project, &config)?;
    session(
        principal,
        project,
        config,
        Wiring {
            store: &store,
            locations,
            events,
            secrets,
        },
    )
}

fn folder_of(source: &ProjectSource) -> Option<&std::path::Path> {
    match source {
        ProjectSource::Folder(folder) => Some(std::path::Path::new(folder.as_str())),
        ProjectSource::Git { .. } => None,
    }
}

/// What a session is built on.
struct Wiring<'a> {
    store: &'a std::sync::Arc<SqliteStore>,
    locations: &'a Locations,
    events: EventBus,
    secrets: &'a Arc<Secrets>,
}

fn session(
    principal: Principal,
    project: Project,
    config: Config,
    wiring: Wiring<'_>,
) -> Result<Session, CodedError> {
    access::require(&principal, Permission::Scan)?;
    let Wiring {
        store,
        locations,
        events,
        secrets,
    } = wiring;
    let scanner = build_scanner(store, &config, locations, events.clone(), secrets)?;
    let tools = tool_checker(store, (locations, index_runner(locations)))?;
    Ok(Session {
        scanner,
        tools,
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
    secrets: &Arc<Secrets>,
) -> Result<ScanSummary, CodedError> {
    open(spec, global, locations, secrets)?.scan(ScanMode::Full)
}

/// The project with the layers of the configuration (ADR 0031); saved when they changed, so that
/// the read models of the next scan are built with them.
fn with_layers(
    store: &SqliteStore,
    mut project: Project,
    config: &Config,
) -> Result<Project, CodedError> {
    let layers = config.project_layers();
    if project.layers != layers {
        project.layers = layers;
        store.save_project(&project)?;
    }
    Ok(project)
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
