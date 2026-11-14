//! The entry point of a scan: builds the pipeline from the ports and drives one job through it.

use std::sync::Arc;

use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_domain::{Permission, Principal, ProjectId, ScanMode, Snapshot};

use super::flow::{Chain, Halt, Pipeline, Single, StageContext};
use super::job::{Cancelled, Failed, Finished, Job, Queued};
use super::stages::{
    AnalyzeStage, DiscoverStage, FetchStage, MeasureStage, PersistStage, Persisted, ResolveStage,
    ScanRequest,
};
use crate::access;
use crate::cancel::CancelToken;
use crate::error::CodedError;
use crate::events::{Event, EventBus, ScanStopped, SnapshotCreated, StopReason};
use crate::ids::IdSequence;
use crate::level2::{LanguageReport, Level2Pass};
use crate::ports::{
    Clock, FileAnalyzer, FileDiscovery, MetricsCalculator, ProjectRepository, ProjectScanner,
    ReferenceResolver, SnapshotStore, SourceProvider,
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
            analyzer: ports.analyzer,
            ids: Arc::clone(ids),
        })
        .then(ResolveStage {
            resolver: ports.resolver,
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
        }
    }

    /// Adds the second pass: after each scan the indexers of the languages run, if the user
    /// allowed it (ADR 0033).
    #[must_use]
    pub fn with_level2(mut self, pass: Level2Pass) -> Self {
        self.level2 = Some(pass);
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
        let ctx = StageContext::new(project_id, job.token(), &self.events);
        match self.pipeline.run(ScanRequest { project, mode }, &ctx) {
            Ok(persisted) => {
                // Level 1 is complete and usable now; level 2 may take minutes.
                self.created(&persisted.snapshot);
                let (snapshot, reports) = self.second_pass(
                    *principal,
                    persisted.clone(),
                    (job.token(), job.level2_wanted()),
                );
                if snapshot != persisted.snapshot {
                    self.created(&snapshot);
                }
                JobResult::Finished(job.finish(snapshot, reports))
            }
            Err(Halt::Cancelled(_)) => {
                self.stopped(project_id, StopReason::Cancelled);
                JobResult::Cancelled(job.cancel())
            }
            Err(Halt::Failed(error)) => self.fail(job.fail(error.code()), project_id),
        }
    }

    /// Level 2 runs after level 1 is stored, so a cancel or a failure of it leaves level 1 as it
    /// is. Without the pass the snapshot is final.
    fn second_pass(
        &self,
        principal: Principal,
        persisted: Persisted,
        (cancel, wanted): (&CancelToken, bool),
    ) -> (Snapshot, Vec<LanguageReport>) {
        match &self.level2 {
            Some(pass) if wanted => {
                let done = pass.run(&principal, &persisted.snapshot, &persisted.root, cancel);
                (done.snapshot, done.reports)
            }
            _ => (persisted.snapshot, Vec::new()),
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

impl ProjectScanner for Scanner {
    fn rescan(&self, principal: &Principal, project: ProjectId) -> Result<Snapshot, CodedError> {
        // A rescan asked for through MCP or the UI is level 1 only: it must not start the build tools
        // of the project. Level 2 runs with the scan of the command line (ADR 0033).
        match self.run(
            principal,
            Job::new(project, ScanMode::Incremental).without_level2(),
        ) {
            JobResult::Finished(job) => Ok(job.snapshot().clone()),
            JobResult::Failed(job) => Err(CodedError::new(job.code())),
            JobResult::Cancelled(_) => Err(CodedError::new(ErrorCode::ScanCancelled)),
        }
    }
}
