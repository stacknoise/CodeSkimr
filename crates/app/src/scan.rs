//! `codeskimr scan`: registers the source as a project and runs one full scan. The [`Session`] is
//! what watch mode scans again and again.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use codeskimr_application::access;
use codeskimr_application::cancel::CancelToken;
use codeskimr_application::config::Config;
use codeskimr_application::error::CodedError;
use codeskimr_application::events::{Event, EventBus};
use codeskimr_application::level2::{LanguageReport, Level2Outcome, Skipped};
use codeskimr_application::pipeline::Level2Run;
use codeskimr_application::pipeline::{Job, JobResult, Queued, Scanner};
use codeskimr_application::ports::{
    CodeStatistics, FileAnalyzer, LanguageOutcome, PathsReport, ProjectRepository,
    ProjectToolCheck, ScanWarning, StatisticsSetup,
};
use codeskimr_application::stage::StageKind;
use codeskimr_application::tools::ToolsView;
use codeskimr_cli::{
    GlobalOptions, LeftOutView, Level2Line, Level2Reason, ScanArgs, ScanSummary, StageLine,
    WarningLine,
};
use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_domain::{
    FolderPath, LanguageId, Permission, Principal, Project, ProjectId, ProjectSource, ScanMode,
    ScanOrigin, SkipReason, Snapshot,
};
use codeskimr_storage::SqliteStore;

use crate::config_files;
use crate::consts::{
    LEVEL2_THREAD_NAME, LOG_STATISTICS_FAILED, LOG_TOOL_CHECK_FAILED, STATISTICS_THREAD_NAME,
};
use crate::cpu_time::{process_cpu_time, seconds_since};
use crate::level2_runs::{Level2Runs, RunLimits, RunSlot};
use crate::locations::Locations;
use crate::principal;
use crate::running::ScanGuard;
use crate::source_arg::{SourceError, SourceSpec, classify};
use crate::wiring::{
    Secrets, build_analysis, index_runner, open_store, scanner_over, tool_checker,
};

/// Checks the source argument before anything is scanned or printed.
///
/// # Errors
/// [`SourceError`] (`PRJ-101`) if the argument is neither a folder nor a Git URL without secrets,
/// or if a credential is named for a folder; `PRJ-102` if the display name is invalid.
pub fn prepare(args: &ScanArgs) -> Result<SourceSpec, SourceError> {
    let spec = classify(&args.source, args.branch.as_deref())?;
    let spec = match &args.credential {
        Some(name) => spec.with_credential(name)?,
        None => spec,
    };
    match &args.name {
        Some(name) => spec.with_name(name),
        None => Ok(spec),
    }
}

/// A registered project with everything needed to scan it again and again.
pub struct Session {
    scanner: Scanner,
    analyzer: Arc<dyn FileAnalyzer>,
    tools: Arc<dyn ProjectToolCheck>,
    statistics: Arc<dyn CodeStatistics>,
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
        self.run_job(
            Job::new(self.project.id, mode)
                .without_level2()
                .from_watcher(),
        )
    }

    /// An incremental scan of level 1 that looks for changes only in `paths` (the way MCP
    /// `rescan` with `paths` does), for the tests.
    ///
    /// # Errors
    /// The same as [`Session::scan`].
    #[cfg(test)]
    pub fn scan_paths(&self, paths: &[String]) -> Result<ScanSummary, CodedError> {
        self.run_job(
            Job::new(self.project.id, ScanMode::Incremental)
                .without_level2()
                .with_paths(paths),
        )
    }

    /// Like [`Session::scan`], but level 1 only, and it hands the token that cancels this scan to
    /// `on_token` before the scan starts, for the tests of the UI commands.
    ///
    /// # Errors
    /// The same as [`Session::scan`].
    #[cfg(test)]
    pub fn scan_with(
        &self,
        mode: ScanMode,
        on_token: impl FnOnce(CancelToken),
    ) -> Result<ScanSummary, CodedError> {
        let job = Job::new(self.project.id, mode).started_from(ScanOrigin::Browser);
        on_token(job.cancel_token());
        self.run_job(job)
    }

    /// Counts the numbers of the statistics screen for `snapshot` in a thread of their own, so
    /// that they are there when the screen opens (ADR 0091). The scan does not wait for them; a
    /// question that comes earlier waits for the same count. A failure is logged: the question
    /// then counts them itself and reports the failure.
    fn count_statistics(&self, snapshot: &Snapshot) {
        let statistics = Arc::clone(&self.statistics);
        let names = self.config.conventions.test_folders.clone();
        let approximate = self.config.ui.approximate_languages.clone();
        let id = snapshot.id;
        let spawned = std::thread::Builder::new()
            .name(STATISTICS_THREAD_NAME.to_owned())
            .spawn(move || {
                let setup = StatisticsSetup {
                    test_names: &names,
                    approximate: &approximate,
                };
                if let Err(error) = statistics.prepare(id, setup) {
                    tracing::warn!(event = LOG_STATISTICS_FAILED, code = error.code().id());
                }
            });
        if spawned.is_err() {
            tracing::warn!(event = LOG_STATISTICS_FAILED);
        }
    }

    fn run_job(&self, job: Job<Queued>) -> Result<ScanSummary, CodedError> {
        self.run_ran(job).map(|ran| ran.summary)
    }

    /// Runs a scan for the browser (`projects.add`, `projects.rescan`): level 1 now, and, if
    /// `run.level2` is set, level 2 in a background task after it (ADR 0072). The command returns
    /// with the summary of level 1; the guard of `running` stays until the task has ended, so
    /// the project counts as scanning until then.
    ///
    /// # Errors
    /// The same as [`Session::scan`], for level 1. Level 2 never fails the call.
    pub fn scan_in_browser(
        self,
        mode: ScanMode,
        run: BrowserRun,
    ) -> Result<BrowserScan, CodedError> {
        let job = Job::new(self.project.id, mode).started_from(ScanOrigin::Browser);
        let cancel = job.cancel_token();
        (run.on_token)(cancel.clone());
        let ran = self.run_ran(job)?;
        self.count_statistics(&ran.snapshot);
        let level2 = if run.level2 {
            self.start_level2(
                Level2Handoff {
                    snapshot: ran.snapshot,
                    root: ran.root,
                    cancel,
                    only: run.language,
                },
                (run.guard, run.runs),
            )
        } else {
            Level2Start::NotRequested
        };
        Ok(BrowserScan {
            summary: ran.summary,
            level2,
        })
    }

    /// Asks for the permission, takes a place and continues in a task. Whatever stops it is stored
    /// as the report of the run: level 1 stays.
    fn start_level2(
        self,
        handoff: Level2Handoff,
        (guard, runs): (ScanGuard, Level2Runs),
    ) -> Level2Start {
        let origin = ScanOrigin::Browser;
        match self.take_place(&handoff, &runs) {
            Ok(slot) => self.spawn_level2(handoff, (guard, slot)),
            Err(reason) => {
                self.scanner.skip_level2(
                    &handoff.snapshot,
                    origin,
                    LanguageOutcome::Skipped(reason),
                    handoff.only.as_ref(),
                );
                Level2Start::NotStarted(reason)
            }
        }
    }

    /// The checks before a run from the browser: the right and the consent (as `authorize_run`
    /// asks them), the master switch of the user configuration, then the place and the pause.
    fn take_place(
        &self,
        handoff: &Level2Handoff,
        runs: &Level2Runs,
    ) -> Result<RunSlot, SkipReason> {
        let gate = self
            .scanner
            .level2_gate(&self.principal, self.project.id, &handoff.root)
            .err()
            .map(|skipped| skipped.reason().unwrap_or(SkipReason::NotConfirmed));
        // The switch is read from the user configuration of this command, which the project file
        // cannot set; a confirmation alone does not start a run from the browser (ADR 0071,
        // decision 8).
        let switched_off = (!self.config.level2.enabled).then_some(SkipReason::NotConfirmed);
        if let Some(reason) = gate.or(switched_off) {
            return Err(reason);
        }
        let limits = RunLimits {
            max_parallel: self.config.level2.max_parallel_runs,
            cooldown: Duration::from_secs(u64::from(self.config.level2.ui_cooldown_seconds)),
        };
        runs.begin(self.project.id, limits, Instant::now())
    }

    fn spawn_level2(
        self,
        handoff: Level2Handoff,
        (guard, slot): (ScanGuard, RunSlot),
    ) -> Level2Start {
        let snapshot = handoff.snapshot.clone();
        let only = handoff.only.clone();
        let session = Arc::new(self);
        let worker = Arc::clone(&session);
        let spawned = std::thread::Builder::new()
            .name(LEVEL2_THREAD_NAME.to_owned())
            .spawn(move || {
                let _slot = slot;
                let _guard = guard;
                let _ = worker.scanner.run_level2(&Level2Run {
                    principal: &worker.principal,
                    snapshot: &handoff.snapshot,
                    root: &handoff.root,
                    cancel: &handoff.cancel,
                    origin: ScanOrigin::Browser,
                    only: handoff.only.as_ref(),
                });
                // Level 2 replaced references, so the numbers are counted again (ADR 0091).
                worker.count_statistics(&handoff.snapshot);
            });
        if spawned.is_ok() {
            return Level2Start::Started;
        }
        // The thread did not start, so the report says that the run failed.
        session.scanner.skip_level2(
            &snapshot,
            ScanOrigin::Browser,
            LanguageOutcome::Failed(ErrorCode::UnexpectedInternalError),
            only.as_ref(),
        );
        Level2Start::NotStarted(SkipReason::Busy)
    }

    fn run_ran(&self, job: Job<Queued>) -> Result<Ran, CodedError> {
        let started = Instant::now();
        let progress = self.events.subscribe();
        let loading_before = self.analyzer.pack_load_time();
        let cpu_start = process_cpu_time();
        let mode = job.mode();
        match self.scanner.run(&self.principal, job) {
            JobResult::Finished(job) => {
                let snapshot = job.snapshot();
                let loading = self
                    .analyzer
                    .pack_load_time()
                    .saturating_sub(loading_before);
                let seconds = without_loading(started.elapsed(), loading);
                let summary = ScanSummary {
                    project: self.project.name.to_string(),
                    snapshot: snapshot.id.get(),
                    files: snapshot.stats.files,
                    symbols: snapshot.stats.symbols,
                    references: snapshot.stats.references,
                    seconds,
                    cpu_seconds: seconds_since(cpu_start),
                    pack_load_seconds: loading.as_secs_f64(),
                    stages: stage_lines(progress.try_iter(), self.project.id),
                    level2: job.level2().iter().map(level2_line).collect(),
                    warnings: job.warnings().iter().map(warning_line).collect(),
                    left_out: left_out_view(job.paths_report()),
                    tools: self.check_tools(mode),
                };
                Ok(Ran {
                    summary,
                    snapshot: snapshot.clone(),
                    root: job.root().clone(),
                })
            }
            JobResult::Failed(job) => Err(CodedError::new(job.code())),
            JobResult::Cancelled(_) => Err(CodedError::new(ErrorCode::ScanCancelled)),
        }
    }
}

/// The time each stage of the scan of `project` took, in the order the stages run: the last report
/// of a stage carries its whole time. The bus carries the scans of every project, so the reports
/// of others are left out; a stage that did not report is left out too.
pub(crate) fn stage_lines(
    events: impl IntoIterator<Item = Event>,
    project: ProjectId,
) -> Vec<StageLine> {
    let mut last: BTreeMap<StageKind, u64> = BTreeMap::new();
    for event in events {
        if let Event::ScanProgressed(report) = event
            && report.project == project
        {
            last.insert(report.stage, report.elapsed_ms);
        }
    }
    last.into_iter()
        .map(|(stage, millis)| StageLine {
            stage: stage.label().to_owned(),
            seconds: Duration::from_millis(millis).as_secs_f64(),
        })
        .collect()
}

/// What a finished scan leaves for the caller.
struct Ran {
    summary: ScanSummary,
    snapshot: Snapshot,
    root: FolderPath,
}

/// What the background task of level 2 works on.
struct Level2Handoff {
    snapshot: Snapshot,
    root: FolderPath,
    cancel: CancelToken,
    /// The one language to check, `None` for all.
    only: Option<LanguageId>,
}

/// What the browser hands to a scan: the registry places and what the caller wants.
pub struct BrowserRun {
    /// Whether level 2 should follow level 1 (the command input `level2`).
    pub level2: bool,
    /// Limits level 2 to this language (the command input `language`).
    pub language: Option<LanguageId>,
    /// Receives the token that cancels the scan, and level 2 after it.
    pub on_token: Box<dyn FnOnce(CancelToken)>,
    /// Marks the project as scanning; it moves into the task of level 2.
    pub guard: ScanGuard,
    /// The registry of level 2 runs.
    pub runs: Level2Runs,
}

/// Whether level 2 follows a scan from the browser.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level2Start {
    /// A task runs the second pass now.
    Started,
    /// The caller asked for level 1 only.
    NotRequested,
    /// Level 2 did not start, for this reason; level 1 is stored.
    NotStarted(SkipReason),
}

/// The result of a scan from the browser.
#[derive(Debug)]
pub struct BrowserScan {
    /// What level 1 found.
    pub summary: ScanSummary,
    /// What became of level 2.
    pub level2: Level2Start,
}

/// The time of a scan without loading language packs (ADR 0059), so that it stays comparable
/// between runs that load different packs. Not below zero.
fn without_loading(elapsed: Duration, loading: Duration) -> f64 {
    elapsed.saturating_sub(loading).as_secs_f64()
}

/// The files the scan left out, as the summary shows them.
fn left_out_view(paths: &PathsReport) -> LeftOutView {
    LeftOutView::of(&paths.left_out, paths.left_out_more)
}

/// The line of the summary for one warning of the scan.
fn warning_line(warning: &ScanWarning) -> WarningLine {
    WarningLine {
        code: warning.code.id().to_owned(),
        count: warning.count,
    }
}

/// The line of the summary for one language of the second pass.
fn level2_line(report: &LanguageReport) -> Level2Line {
    let language = report.language.to_string();
    match &report.outcome {
        Level2Outcome::Checked {
            files,
            stale,
            outside_scan,
            failed_workspaces,
            ..
        } => Level2Line {
            language,
            files: *files,
            stale: *stale,
            outside_scan: *outside_scan,
            failed_workspaces: *failed_workspaces,
            reason: None,
        },
        Level2Outcome::Skipped(why) => Level2Line {
            language,
            files: u64::default(),
            stale: u64::default(),
            outside_scan: u64::default(),
            failed_workspaces: u64::default(),
            reason: Some(reason_of(why)),
        },
    }
}

fn reason_of(skipped: &Skipped) -> Level2Reason {
    match skipped {
        Skipped::NotPermitted => Level2Reason::NotPermitted,
        Skipped::NotConfirmed => Level2Reason::NotConfirmed,
        Skipped::Unavailable(_) => Level2Reason::Unavailable,
        Skipped::NoProjectFile => Level2Reason::NoProjectFile,
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
    let analysis = build_analysis(&config, locations)?;
    let scanner = scanner_over(
        &analysis,
        (store, &config, locations),
        (events.clone(), secrets),
        None,
    );
    let tools = tool_checker(store, (locations, index_runner(locations)))?;
    Ok(Session {
        scanner,
        analyzer: analysis.analyzer,
        tools,
        statistics: store.clone(),
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

/// The project of this source: the registered one, or a new one (one transaction in the store,
/// so concurrent scans of different sources never share an id).
///
/// A name the user chose (`--name`) also renames a project that is registered already.
fn register(store: &SqliteStore, spec: SourceSpec) -> Result<Project, CodedError> {
    let (name, named) = (spec.name.clone(), spec.named);
    let project = store.register_project(spec.name, spec.source)?;
    if named && project.name != name {
        return store.rename_project(project.id, name);
    }
    Ok(project)
}

#[cfg(test)]
mod stage_time_tests {
    use codeskimr_application::events::{Event, ScanProgressed};
    use codeskimr_application::stage::StageKind;
    use codeskimr_domain::ProjectId;

    use super::stage_lines;

    fn report(project: u64, stage: StageKind, elapsed_ms: u64) -> Event {
        Event::ScanProgressed(ScanProgressed {
            project: ProjectId::new(project),
            stage,
            done: 1,
            total: 1,
            elapsed_ms,
        })
    }

    fn seen(lines: &[codeskimr_cli::StageLine]) -> Vec<(&str, f64)> {
        lines
            .iter()
            .map(|line| (line.stage.as_str(), line.seconds))
            .collect()
    }

    #[test]
    fn the_last_report_of_a_stage_gives_its_time_in_the_order_of_the_stages() {
        let events = [
            report(1, StageKind::Persist, 4000),
            report(1, StageKind::Discover, 100),
            report(1, StageKind::Discover, 1500),
            report(1, StageKind::Analyze, 250),
        ];
        let lines = stage_lines(events, ProjectId::new(1));
        assert_eq!(
            seen(&lines),
            [("discover", 1.5), ("analyze", 0.25), ("persist", 4.0)]
        );
    }

    #[test]
    fn the_reports_of_another_project_and_other_events_are_left_out() {
        let events = [
            report(2, StageKind::Resolve, 9000),
            report(1, StageKind::Measure, 70),
        ];
        let lines = stage_lines(events, ProjectId::new(1));
        assert_eq!(seen(&lines), [("measure", 0.07)]);
        assert_eq!(
            stage_lines([], ProjectId::new(1)),
            Vec::<codeskimr_cli::StageLine>::new()
        );
    }
}

#[cfg(test)]
mod load_time_tests {
    use super::*;

    #[test]
    fn the_load_time_is_taken_out_of_the_scan_time_and_never_goes_below_zero() {
        let second = Duration::from_secs(1);
        assert!((without_loading(second * 3, second) - 2.0).abs() < f64::EPSILON);
        assert!(without_loading(second, second * 2).abs() < f64::EPSILON);
    }
}
