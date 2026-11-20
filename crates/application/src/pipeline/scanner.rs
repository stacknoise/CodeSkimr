//! The entry point of a scan: builds the pipeline from the ports and drives one job through it.

use std::sync::Arc;

use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_domain::{
    AlertTrigger, FolderPath, Permission, Principal, ProjectId, ProjectSource, ScanMode,
    ScanOrigin, SkipReason, Snapshot,
};

use super::flow::{Chain, Halt, Pipeline, Single, StageContext};
use super::job::{Cancelled, Failed, Finished, Job, Queued, Running};
use super::report_step::ReportStep;
use super::stages::{
    AnalyzeStage, DiscoverStage, FetchStage, MeasureStage, PersistStage, Persisted, ResolveStage,
    ScanRequest,
};
use crate::access;
use crate::alerts::AlertStep;
use crate::cancel::CancelToken;
use crate::consts::{LOG_ALERT_STEP_FAILED, LOG_ALERTS_CHECKED, LOG_REPORT_NOT_STORED};
use crate::error::CodedError;
use crate::events::{Event, EventBus, Level2Finished, ScanStopped, SnapshotCreated, StopReason};
use crate::ids::IdSequence;
use crate::level2::{LanguageReport, Level2Outcome, Level2Pass, Skipped};
use crate::ports::{
    Clock, FileAnalyzer, FileDiscovery, LanguageLine, LanguageOutcome, MetricsCalculator,
    ProjectRepository, ProjectScanner, ReferenceResolver, Rescanned, ScanWarning, SnapshotStore,
    SourceProvider,
};

/// Everything the scanner needs from the adapters, wired by the composition root.
pub struct ScannerPorts {
    /// Makes the source available.
    pub source: Arc<dyn SourceProvider>,
    /// Lists and hashes files.
    pub discovery: Arc<dyn FileDiscovery>,
    /// Parses and extracts.
    pub analyzer: Arc<dyn FileAnalyzer>,
    /// Resolves references.
    pub resolver: Arc<dyn ReferenceResolver>,
    /// Computes metrics.
    pub metrics: Arc<dyn MetricsCalculator>,
    /// Reads projects and writes snapshots.
    pub projects: Arc<dyn ProjectRepository>,
    /// Writes snapshots.
    pub store: Arc<dyn SnapshotStore>,
    /// Current time.
    pub clock: Arc<dyn Clock>,
    /// Most threads that analyse files at the same time; 0 means one per core (`scan.max_workers`).
    pub max_workers: u32,
}

/// The six stages in order, with every type spelled out by the compiler.
type ScanFlow = Chain<
    Chain<
        Chain<Chain<Chain<Single<FetchStage>, DiscoverStage>, AnalyzeStage>, ResolveStage>,
        MeasureStage,
    >,
    PersistStage,
>;

/// How a job ended.
#[derive(Debug)]
pub enum JobResult {
    /// A snapshot was written.
    Finished(Job<Finished>),
    /// The user cancelled; no snapshot was written.
    Cancelled(Job<Cancelled>),
    /// A stage failed; no snapshot was written.
    Failed(Job<Failed>),
}

/// Runs scan jobs. Create one per application and reuse it.
pub struct Scanner {
    pipeline: Pipeline<ScanFlow>,
    projects: Arc<dyn ProjectRepository>,
    events: EventBus,
    level2: Option<Level2Pass>,
    alerts: Option<AlertStep>,
    reports: Option<ReportStep>,
}

impl Scanner {
    /// Wires the six stages from `ports`; `ids` must continue after the highest stored id.
    #[must_use]
    pub fn new(ports: ScannerPorts, ids: &Arc<IdSequence>, events: EventBus) -> Self {
        let pipeline = Pipeline::new(FetchStage {
            source: ports.source,
            ids: Arc::clone(ids),
        })
        .then(DiscoverStage {
            discovery: ports.discovery,
            projects: Arc::clone(&ports.projects),
            analyzer: Arc::clone(&ports.analyzer),
        })
        .then(AnalyzeStage {
            analyzer: Arc::clone(&ports.analyzer),
            ids: Arc::clone(ids),
            max_workers: ports.max_workers,
        })
        .then(ResolveStage {
            projects: Arc::clone(&ports.projects),
            resolver: ports.resolver,
            analyzer: Arc::clone(&ports.analyzer),
            ids: Arc::clone(ids),
        })
        .then(MeasureStage {
            calculator: ports.metrics,
        })
        .then(PersistStage {
            store: ports.store,
            clock: ports.clock,
        });
        Self {
            pipeline,
            projects: ports.projects,
            events,
            level2: None,
            alerts: None,
            reports: None,
        }
    }

    /// Adds the second pass: after each scan the indexers of the languages run, if the user
    /// allowed it (ADR 0033).
    #[must_use]
    pub fn with_level2(mut self, pass: Level2Pass) -> Self {
        self.level2 = Some(pass);
        self
    }

    /// Adds the alert step: after each snapshot the critical symbols are compared with their
    /// baselines (ADR 0068). It runs after the snapshot is stored, so a failure of it never
    /// undoes the snapshot.
    #[must_use]
    pub fn with_alerts(mut self, step: AlertStep) -> Self {
        self.alerts = Some(step);
        self
    }

    /// Adds the report step: after each scan its level, channel and level 2 outcomes are stored
    /// (ADR 0072). A failure to store them becomes the warning `PRJ-502`; the snapshot stays.
    #[must_use]
    pub fn with_reports(mut self, step: ReportStep) -> Self {
        self.reports = Some(step);
        self
    }

    /// Runs `job` for `principal` to its end and publishes `SnapshotCreated` or `ScanStopped`.
    ///
    /// A caller without `Permission::Scan` gets `Failed` with `AUTH-401`; nothing is read and no
    /// event is published, so a denied caller learns nothing about the project. A job that fails
    /// to load its project ends as `Failed`, like any stage failure.
    #[must_use]
    pub fn run(&self, principal: &Principal, job: Job<Queued>) -> JobResult {
        let project_id = job.project();
        let mode = job.mode();
        let job = job.start();
        if let Err(denied) = access::require(principal, Permission::Scan) {
            return JobResult::Failed(job.fail(denied.code()));
        }
        let project = match self.projects.project(project_id) {
            Ok(project) => project,
            Err(cause) => return self.fail(job.fail(cause.code()), project_id),
        };
        let trigger = trigger_of(&project.source, job.trigger());
        let ctx = StageContext::new(project_id, job.token(), &self.events);
        match self.pipeline.run(
            ScanRequest {
                project,
                mode,
                paths: job.paths().clone(),
            },
            &ctx,
        ) {
            Ok(persisted) => self.finished(*principal, job, (persisted, trigger)),
            Err(Halt::Cancelled(_)) => {
                self.stopped(project_id, StopReason::Cancelled);
                JobResult::Cancelled(job.cancel())
            }
            Err(Halt::Failed(error)) => self.fail(job.fail(error.code()), project_id),
        }
    }

    /// Level 1 is complete and usable now; level 2 may take minutes. Tells so, runs the second
    /// pass and ends the job.
    fn finished(
        &self,
        principal: Principal,
        job: Job<Running>,
        (persisted, trigger): (Persisted, AlertTrigger),
    ) -> JobResult {
        self.created(&persisted.snapshot);
        self.check_alerts(&persisted.snapshot, trigger);
        let (snapshot, level2) = self.second_pass(
            principal,
            persisted.clone(),
            (job.token(), job.level2_wanted()),
        );
        if snapshot != persisted.snapshot {
            self.created(&snapshot);
        }
        let mut warnings = persisted.warnings;
        warnings.extend(self.store_report(&snapshot, job.origin(), level2.as_deref()));
        JobResult::Finished(job.finish(
            snapshot,
            level2.unwrap_or_default(),
            warnings,
            (persisted.paths, persisted.root),
        ))
    }

    /// Level 2 runs after level 1 is stored, so a cancel or a failure of it leaves level 1 as it
    /// is. Without the pass the snapshot is final and there is no level 2 report (`None`).
    fn second_pass(
        &self,
        principal: Principal,
        persisted: Persisted,
        (cancel, wanted): (&CancelToken, bool),
    ) -> (Snapshot, Option<Vec<LanguageReport>>) {
        match &self.level2 {
            Some(pass) if wanted => {
                let done = pass.run(&principal, &persisted.snapshot, &persisted.root, cancel);
                (done.snapshot, Some(done.reports))
            }
            _ => (persisted.snapshot, None),
        }
    }

    /// Whether `principal` may run the indexers of `project` now, without running anything.
    ///
    /// # Errors
    /// `NotPermitted`, `NotConfirmed`, or `Failed` if the project cannot be read. A scanner
    /// without a second pass cannot run level 2, which is `NotConfirmed`.
    pub fn level2_gate(
        &self,
        principal: &Principal,
        project: ProjectId,
        root: &FolderPath,
    ) -> Result<(), Skipped> {
        let pass = self.level2.as_ref().ok_or(Skipped::NotConfirmed)?;
        let project = self
            .projects
            .project(project)
            .map_err(|cause| Skipped::Failed(cause.code()))?;
        pass.authorize(principal, &project, root)
    }

    /// Runs the second pass on a snapshot that level 1 stored, stores the report and tells the
    /// event bus. It asks for the permission again when each indexer starts, so what the owner
    /// revokes before then holds. Level 1 stays as it is whatever happens.
    #[must_use]
    pub fn run_level2(&self, run: &Level2Run<'_>) -> Vec<LanguageReport> {
        let Some(pass) = &self.level2 else {
            return Vec::new();
        };
        let done = pass.run(run.principal, run.snapshot, run.root, run.cancel);
        if done.snapshot != *run.snapshot {
            self.created(&done.snapshot);
        }
        // A failure to store is logged by `store_report`; the run itself has ended.
        self.store_report(&done.snapshot, run.origin, Some(&done.reports));
        self.finished_level2(run.snapshot, tally(&done.reports));
        done.reports
    }

    /// Stores that level 2 did not run for any language, for `reason` (`busy` or `paused`), and
    /// tells the event bus that the run is over.
    pub fn skip_level2(&self, snapshot: &Snapshot, origin: ScanOrigin, reason: SkipReason) {
        let Some(pass) = &self.level2 else {
            return;
        };
        let lines: Vec<LanguageLine> = pass
            .languages()
            .map(|language| LanguageLine {
                language: language.clone(),
                outcome: LanguageOutcome::Skipped(reason),
            })
            .collect();
        let skipped = u32::try_from(lines.len()).unwrap_or(u32::MAX);
        if let Some(step) = &self.reports
            && let Err(code) = step.store_lines(snapshot, origin, Some(lines))
        {
            tracing::warn!(event = LOG_REPORT_NOT_STORED, code = code.id());
        }
        self.finished_level2(snapshot, (0, skipped, 0));
    }

    fn finished_level2(&self, snapshot: &Snapshot, (checked, skipped, failed): (u32, u32, u32)) {
        self.events.publish(&Event::Level2Finished(Level2Finished {
            project: snapshot.project,
            snapshot: snapshot.id,
            checked,
            skipped,
            failed,
        }));
    }

    /// Stores the report of the scan. A failure is a warning of the scan, never the end of it.
    fn store_report(
        &self,
        snapshot: &Snapshot,
        origin: ScanOrigin,
        level2: Option<&[LanguageReport]>,
    ) -> Option<ScanWarning> {
        let step = self.reports.as_ref()?;
        let code = step.store(snapshot, origin, level2).err()?;
        tracing::warn!(event = LOG_REPORT_NOT_STORED, code = code.id());
        Some(ScanWarning { code, count: 1 })
    }

    /// Runs the alert step on the snapshot that was just stored. A failure is logged and goes no
    /// further: the snapshot is written, and the next scan checks again.
    fn check_alerts(&self, snapshot: &Snapshot, trigger: AlertTrigger) {
        let Some(step) = &self.alerts else {
            return;
        };
        match step.run(snapshot, trigger) {
            Ok(report) => tracing::info!(
                event = LOG_ALERTS_CHECKED,
                checked = report.checked,
                baselines = report.baselines,
                raised = report.raised
            ),
            Err(error) => tracing::warn!(event = LOG_ALERT_STEP_FAILED, code = error.code().id()),
        }
    }

    fn created(&self, snapshot: &Snapshot) {
        self.events
            .publish(&Event::SnapshotCreated(SnapshotCreated {
                snapshot: snapshot.clone(),
            }));
    }

    fn fail(&self, job: Job<Failed>, project: ProjectId) -> JobResult {
        self.stopped(project, StopReason::Failed(job.code()));
        JobResult::Failed(job)
    }

    fn stopped(&self, project: ProjectId, reason: StopReason) {
        self.events
            .publish(&Event::ScanStopped(ScanStopped { project, reason }));
    }
}

/// What the second pass needs after level 1 is stored.
#[derive(Debug)]
pub struct Level2Run<'a> {
    /// The caller that asked for level 2.
    pub principal: &'a Principal,
    /// The snapshot level 1 stored.
    pub snapshot: &'a Snapshot,
    /// The folder with its files.
    pub root: &'a FolderPath,
    /// Stops the run.
    pub cancel: &'a CancelToken,
    /// The channel that started the scan.
    pub origin: ScanOrigin,
}

/// Languages checked, skipped and failed.
fn tally(reports: &[LanguageReport]) -> (u32, u32, u32) {
    let (mut checked, mut skipped, mut failed) = (0, 0, 0);
    for report in reports {
        match &report.outcome {
            Level2Outcome::Checked { .. } => checked += 1,
            Level2Outcome::Skipped(Skipped::Failed(_)) => failed += 1,
            Level2Outcome::Skipped(_) => skipped += 1,
        }
    }
    (checked, skipped, failed)
}

/// What started the scan: the trigger of the job, but a project that lives in a Git repository
/// only changes through a fetch.
const fn trigger_of(source: &ProjectSource, started_by: AlertTrigger) -> AlertTrigger {
    match source {
        ProjectSource::Git { .. } => AlertTrigger::Fetch,
        ProjectSource::Folder(_) => started_by,
    }
}

impl ProjectScanner for Scanner {
    fn rescan_paths(
        &self,
        principal: &Principal,
        project: ProjectId,
        paths: &[String],
    ) -> Result<Rescanned, CodedError> {
        // A rescan asked for through MCP or the UI is level 1 only: it must not start the build tools
        // of the project. Level 2 runs with the scan of the command line (ADR 0033).
        match self.run(
            principal,
            Job::new(project, ScanMode::Incremental)
                .without_level2()
                .started_from(ScanOrigin::Mcp)
                .with_paths(paths),
        ) {
            JobResult::Finished(job) => Ok(Rescanned {
                snapshot: job.snapshot().clone(),
                paths: job.paths_report().clone(),
            }),
            JobResult::Failed(job) => Err(CodedError::new(job.code())),
            JobResult::Cancelled(_) => Err(CodedError::new(ErrorCode::ScanCancelled)),
        }
    }
}
