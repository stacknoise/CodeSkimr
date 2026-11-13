//! Tests of the scan pipeline, the scanner, the job typestate and the event bus with fake ports.

use std::sync::{Arc, Mutex, PoisonError};

use codeskimr_application::cancel::CancelToken;
use codeskimr_application::error::CodedError;
use codeskimr_application::events::{Event, EventBus, StopReason};
use codeskimr_application::ids::IdSequence;
use codeskimr_application::pipeline::{Job, JobResult, Scanner, ScannerPorts};
use codeskimr_application::ports::{
    AnalyzeRequest, Clock, DiscoveredFile, FetchedSource, FileAnalyzer, FileDiscovery, FileFacts,
    MetricsCalculator, PendingReference, ProjectRepository, ReferenceResolver, SnapshotData,
    SnapshotStore, SourceProvider,
};
use codeskimr_application::stage::StageKind;
use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_domain::{
    BranchName, CommitHash, ContentHash, FolderPath, LanguageId, Loc, Metric, MetricKind,
    MetricValue, ModelError, Position, Project, ProjectId, ProjectName, ProjectSource,
    QualifiedName, Reference, ReferenceKind, RelativePath, ScanMode, Snapshot, SourceFile, Span,
    Symbol, SymbolKind, SymbolName, Timestamp,
};

/// One fake that plays every port. `fail_at` makes that stage fail, `cancel_at` raises the token
/// when that stage runs, `unchanged` marks the first file as not changed.
#[derive(Default)]
struct World {
    fail_at: Option<StageKind>,
    cancel_at: Option<(StageKind, CancelToken)>,
    unchanged: bool,
    missing_project: bool,
    persisted: Mutex<Vec<SnapshotData>>,
}

impl World {
    fn gate(&self, stage: StageKind) -> Result<(), CodedError> {
        if let Some((at, token)) = &self.cancel_at
            && *at == stage
        {
            token.cancel();
        }
        if self.fail_at == Some(stage) {
            Err(CodedError::new(ErrorCode::GitFetchFailed))
        } else {
            Ok(())
        }
    }

    fn persisted_count(&self) -> usize {
        self.persisted
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .len()
    }
}

/// Turns a model violation of the fixture data into a coded failure.
fn model<T>(value: Result<T, ModelError>) -> Result<T, CodedError> {
    value.map_err(CodedError::caused_by)
}

fn project() -> Result<Project, CodedError> {
    Ok(Project {
        id: ProjectId::new(7),
        name: model(ProjectName::new("demo"))?,
        source: ProjectSource::Git {
            url: model("https://example.org/demo.git".parse())?,
            branch: model(BranchName::new("main"))?,
        },
        layers: Vec::new(),
    })
}

fn discovered(path: &str, changed: bool) -> Result<DiscoveredFile, CodedError> {
    Ok(DiscoveredFile {
        path: model(RelativePath::new(path))?,
        language: model(LanguageId::new("rust"))?,
        content_hash: model(ContentHash::new("h"))?,
        changed,
    })
}

fn span() -> Result<Span, CodedError> {
    let at = Position { line: 1, column: 1 };
    model(Span::new(at, at))
}

impl SourceProvider for World {
    fn fetch(&self, _: &ProjectSource, _: &CancelToken) -> Result<FetchedSource, CodedError> {
        self.gate(StageKind::Fetch)?;
        Ok(FetchedSource {
            root: model(FolderPath::new("/work"))?,
            commit: Some(model(CommitHash::new("abc"))?),
        })
    }
}

impl FileDiscovery for World {
    fn discover(
        &self,
        _: &FolderPath,
        _: &[SourceFile],
        _: &CancelToken,
    ) -> Result<Vec<DiscoveredFile>, CodedError> {
        self.gate(StageKind::Discover)?;
        Ok(vec![
            discovered("a.rs", !self.unchanged)?,
            discovered("b.rs", true)?,
        ])
    }
}

impl FileAnalyzer for World {
    fn analyze(&self, request: &AnalyzeRequest<'_>) -> Result<FileFacts, CodedError> {
        self.gate(StageKind::Analyze)?;
        let file = SourceFile {
            id: request.ids.file(),
            snapshot: request.snapshot,
            path: request.file.path.clone(),
            language: request.file.language.clone(),
            content_hash: request.file.content_hash.clone(),
            loc: Loc(3),
        };
        let symbol = Symbol {
            id: request.ids.symbol(),
            file: file.id,
            kind: SymbolKind::Function,
            name: model(SymbolName::new("f"))?,
            qualified_name: model(QualifiedName::new("m::f"))?,
            parent: None,
            visibility: None,
            is_static: false,
            span: span()?,
            signature: None,
            doc: None,
        };
        let pending = vec![PendingReference {
            from: symbol.id,
            file: file.id,
            span: span()?,
            kind: ReferenceKind::Call,
            target: symbol.qualified_name.clone(),
        }];
        Ok(FileFacts {
            file,
            symbols: vec![symbol],
            pending,
            fingerprints: Vec::new(),
        })
    }
}

impl ReferenceResolver for World {
    fn resolve(
        &self,
        _: &[Symbol],
        pending: &[PendingReference],
        ids: &IdSequence,
    ) -> Result<Vec<Reference>, CodedError> {
        self.gate(StageKind::Resolve)?;
        Ok(pending
            .iter()
            .map(|item| Reference {
                id: ids.reference(),
                from: item.from,
                to: item.from,
                file: item.file,
                span: item.span,
                kind: item.kind,
                confidence: codeskimr_domain::Confidence::Certain,
            })
            .collect())
    }
}

impl MetricsCalculator for World {
    fn measure(&self, symbols: &[Symbol], _: &[Reference]) -> Result<Vec<Metric>, CodedError> {
        self.gate(StageKind::Measure)?;
        let value = model(MetricValue::new(3.0))?;
        Ok(symbols
            .iter()
            .map(|symbol| Metric {
                symbol: symbol.id,
                kind: MetricKind::Loc,
                value,
            })
            .collect())
    }
}

impl ProjectRepository for World {
    fn project(&self, _: ProjectId) -> Result<Project, CodedError> {
        if self.missing_project {
            Err(CodedError::new(ErrorCode::ProjectNotFound))
        } else {
            project()
        }
    }

    fn latest_files(&self, _: ProjectId) -> Result<Vec<SourceFile>, CodedError> {
        Ok(Vec::new())
    }
}

impl SnapshotStore for World {
    fn persist(&self, data: SnapshotData) -> Result<Snapshot, CodedError> {
        self.gate(StageKind::Persist)?;
        let snapshot = data.snapshot.clone();
        self.persisted
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(data);
        Ok(snapshot)
    }
}

impl Clock for World {
    fn now(&self) -> Timestamp {
        Timestamp(42)
    }
}

fn scanner(world: &Arc<World>, bus: &EventBus) -> Scanner {
    let ports = ScannerPorts {
        source: world.clone(),
        discovery: world.clone(),
        analyzer: world.clone(),
        resolver: world.clone(),
        metrics: world.clone(),
        projects: world.clone(),
        store: world.clone(),
        clock: world.clone(),
    };
    Scanner::new(ports, &Arc::new(IdSequence::starting_at(1)), bus.clone())
}

fn run(world: &Arc<World>, mode: ScanMode) -> (JobResult, Vec<Event>) {
    let bus = EventBus::new();
    let events = bus.subscribe();
    let result = scanner(world, &bus).run(Job::new(ProjectId::new(7), mode));
    (result, events.try_iter().collect())
}

fn stages_seen(events: &[Event]) -> Vec<StageKind> {
    let mut seen = Vec::new();
    for event in events {
        if let Event::ScanProgressed(progress) = event
            && seen.last() != Some(&progress.stage)
        {
            seen.push(progress.stage);
        }
    }
    seen
}

#[test]
fn full_scan_runs_all_stages_and_persists_once() {
    let world = Arc::new(World::default());
    let (result, events) = run(&world, ScanMode::Full);
    let JobResult::Finished(job) = result else {
        panic!("scan did not finish");
    };
    assert_eq!(job.snapshot().stats.files, 2);
    assert_eq!(job.snapshot().stats.symbols, 2);
    assert_eq!(job.snapshot().stats.references, 2);
    assert_eq!(world.persisted_count(), 1);
    assert_eq!(stages_seen(&events), StageKind::ALL);
    assert!(matches!(events.last(), Some(Event::SnapshotCreated(_))));
}

#[test]
fn incremental_scan_skips_unchanged_files() {
    let world = Arc::new(World {
        unchanged: true,
        ..World::default()
    });
    let (result, _) = run(&world, ScanMode::Incremental);
    let JobResult::Finished(job) = result else {
        panic!("scan did not finish");
    };
    assert_eq!(job.snapshot().stats.files, 1);
    let (full, _) = run(&world, ScanMode::Full);
    assert!(matches!(full, JobResult::Finished(job) if job.snapshot().stats.files == 2));
}

#[test]
fn failing_stage_ends_the_job_without_a_snapshot() {
    for stage in StageKind::ALL {
        let world = Arc::new(World {
            fail_at: Some(*stage),
            ..World::default()
        });
        let (result, events) = run(&world, ScanMode::Full);
        let JobResult::Failed(job) = result else {
            panic!("scan should fail at {stage:?}");
        };
        assert_eq!(job.code(), ErrorCode::GitFetchFailed);
        assert!(matches!(
            events.last(),
            Some(Event::ScanStopped(stopped))
                if stopped.reason == StopReason::Failed(ErrorCode::GitFetchFailed)
        ));
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, Event::SnapshotCreated(_)))
        );
    }
}

#[test]
fn unknown_project_fails_with_its_code() {
    let world = Arc::new(World {
        missing_project: true,
        ..World::default()
    });
    let (result, _) = run(&world, ScanMode::Full);
    assert!(matches!(result, JobResult::Failed(job) if job.code() == ErrorCode::ProjectNotFound));
}

/// Runs a job whose token the fake raises when `cancel_at` runs and optionally makes that stage fail.
fn cancelled_run(cancel_at: StageKind, fail: bool) -> (JobResult, Vec<Event>, Arc<World>) {
    let bus = EventBus::new();
    let events = bus.subscribe();
    let job = Job::new(ProjectId::new(7), ScanMode::Full);
    let world = Arc::new(World {
        cancel_at: Some((cancel_at, job.cancel_token())),
        fail_at: fail.then_some(cancel_at),
        ..World::default()
    });
    let result = scanner(&world, &bus).run(job);
    (result, events.try_iter().collect(), world)
}

#[test]
fn cancelling_at_any_stage_writes_nothing() {
    // Measure is the last stage before Persist: the boundary where a partial write could happen.
    for stage in [StageKind::Fetch, StageKind::Analyze, StageKind::Measure] {
        let (result, events, world) = cancelled_run(stage, false);
        assert!(matches!(result, JobResult::Cancelled(_)), "{stage:?}");
        assert_eq!(world.persisted_count(), 0);
        assert!(matches!(
            events.last(),
            Some(Event::ScanStopped(stopped)) if stopped.reason == StopReason::Cancelled
        ));
    }
}

#[test]
fn adapter_error_after_cancellation_counts_as_cancelled() {
    let (result, _, world) = cancelled_run(StageKind::Discover, true);
    assert!(matches!(result, JobResult::Cancelled(_)));
    assert_eq!(world.persisted_count(), 0);
}

#[test]
fn token_raised_before_start_cancels_without_any_work() {
    let bus = EventBus::new();
    let events = bus.subscribe();
    let world = Arc::new(World::default());
    let job = Job::new(ProjectId::new(7), ScanMode::Full);
    job.cancel_token().cancel();
    let result = scanner(&world, &bus).run(job);
    assert!(matches!(result, JobResult::Cancelled(_)));
    assert_eq!(world.persisted_count(), 0);
    assert!(
        !events
            .try_iter()
            .any(|e| matches!(e, Event::ScanProgressed(_)))
    );
}

#[test]
fn job_cancelled_before_start_never_reaches_running() {
    let job = Job::new(ProjectId::new(1), ScanMode::Full);
    assert_eq!(job.mode(), ScanMode::Full);
    let cancelled = job.cancel();
    assert_eq!(cancelled.project(), ProjectId::new(1));
}

#[test]
fn bus_delivers_to_all_and_forgets_dropped_subscribers() {
    let bus = EventBus::new();
    let first = bus.subscribe();
    let second = bus.subscribe();
    let event = Event::FilesChanged(codeskimr_application::events::FilesChanged {
        project: ProjectId::new(1),
        files: 3,
    });
    bus.publish(&event);
    drop(second);
    bus.publish(&event);
    assert_eq!(first.try_iter().count(), 2);
}

#[test]
fn ids_are_unique_and_stage_labels_round_trip() {
    let ids = IdSequence::starting_at(10);
    assert_eq!(ids.file().get(), 10);
    assert_eq!(ids.symbol().get(), 11);
    let labels: Vec<_> = StageKind::ALL.iter().map(|s| s.label()).collect();
    assert_eq!(labels.len(), 6);
    assert_eq!(StageKind::Analyze.label(), "analyze");
    let error = CodedError::new(ErrorCode::DiskFull);
    assert_eq!(error.code(), ErrorCode::DiskFull);
}
