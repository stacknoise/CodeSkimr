//! Tests of the scan pipeline, the scanner, the job typestate and the event bus with fake ports.

use std::sync::{Arc, Mutex, PoisonError};

use codeskimr_application::cancel::CancelToken;
use codeskimr_application::error::CodedError;
use codeskimr_application::events::{Event, EventBus, StopReason};
use codeskimr_application::ids::IdSequence;
use codeskimr_application::pipeline::{Job, JobResult, Scanner, ScannerPorts};
use codeskimr_application::ports::{
    AnalyzeRequest, Clock, DiscoveredFile, FetchedSource, FileAnalyzer, FileDiscovery, FileFacts,
    MetricsCalculator, PendingReference, PreviousIndex, ProjectRepository, ProjectScanner,
    ReferenceReplacement, ReferenceResolver, SnapshotData, SnapshotStore, SourceProvider,
    SymbolFingerprints,
};
use codeskimr_application::stage::StageKind;
use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_domain::{
    BranchName, CommitHash, Confidence, ContentHash, FileId, Fingerprint, Fingerprints, FolderPath,
    LanguageId, Loc, Metric, MetricKind, MetricValue, ModelError, Permission, PermissionSet,
    Position, Principal, Project, ProjectId, ProjectName, ProjectSource, QualifiedName, Reference,
    ReferenceId, ReferenceKind, RelativePath, ScanMode, Snapshot, SnapshotId, SourceFile, Span,
    Symbol, SymbolCounts, SymbolId, SymbolKind, SymbolName, Timestamp,
};

/// The newest snapshot as the repository reports it.
#[derive(Default, Clone)]
struct Previous {
    files: Vec<SourceFile>,
    index: PreviousIndex,
}

/// One fake that plays every port. `fail_at` makes that stage fail, `cancel_at` raises the token
/// when that stage runs, `previous` is the newest snapshot, `found` the files on disk as
/// `(path, changed)`; without it the disk holds `a.rs` and `b.rs`, both changed.
#[derive(Default)]
struct World {
    fail_at: Option<StageKind>,
    cancel_at: Option<(StageKind, CancelToken)>,
    previous: Previous,
    found: Option<Vec<(&'static str, bool)>>,
    missing_project: bool,
    panic_in_analyze: bool,
    persisted: Mutex<Vec<SnapshotData>>,
    measured: Mutex<Vec<(usize, usize)>>,
    counted: Mutex<Vec<usize>>,
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

    /// What the last scan handed to the store; an error stands for "nothing was persisted".
    fn last_persisted(&self) -> Result<SnapshotData, CodedError> {
        let persisted = self
            .persisted
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        persisted
            .last()
            .cloned()
            .ok_or_else(|| CodedError::new(ErrorCode::DiskFull))
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
            credential: None,
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
        let found = self
            .found
            .clone()
            .unwrap_or_else(|| vec![("a.rs", true), ("b.rs", true)]);
        found
            .into_iter()
            .map(|(path, changed)| discovered(path, changed))
            .collect()
    }
}

/// The counts the fake analyzer reports for a symbol: three code lines, no body.
fn counted(symbol: SymbolId) -> SymbolCounts {
    SymbolCounts {
        symbol,
        loc: Loc(3),
        body: None,
        counted: true,
    }
}

impl FileAnalyzer for World {
    fn analyze(&self, request: &AnalyzeRequest<'_>) -> Result<FileFacts, CodedError> {
        self.gate(StageKind::Analyze)?;
        assert!(!self.panic_in_analyze, "the analyzer panics");
        let file = SourceFile {
            id: request.ids.file(),
            snapshot: request.snapshot,
            path: request.file.path.clone(),
            language: request.file.language.clone(),
            content_hash: request.file.content_hash.clone(),
            loc: Loc(3),
        };
        let stem = request.file.path.as_str().trim_end_matches(".rs");
        let symbol = Symbol {
            id: request.ids.symbol(),
            file: file.id,
            kind: SymbolKind::Function,
            name: model(SymbolName::new(stem))?,
            qualified_name: model(QualifiedName::new(format!("m::{stem}")))?,
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
            receiver: None,
            previous: None,
        }];
        Ok(FileFacts {
            file,
            pending,
            fingerprints: vec![fingerprinted(symbol.id)],
            counts: vec![counted(symbol.id)],
            symbols: vec![symbol],
        })
    }
}

fn fingerprinted(symbol: SymbolId) -> SymbolFingerprints {
    SymbolFingerprints {
        symbol,
        fingerprints: Fingerprints::from_entries(vec![Fingerprint {
            hash: 1,
            position: 0,
        }]),
    }
}

impl ReferenceResolver for World {
    fn resolve(
        &self,
        symbols: &[Symbol],
        pending: &[PendingReference],
        ids: &IdSequence,
    ) -> Result<Vec<Reference>, CodedError> {
        self.gate(StageKind::Resolve)?;
        Ok(pending
            .iter()
            .filter_map(|item| {
                let target = symbols
                    .iter()
                    .find(|symbol| symbol.qualified_name == item.target)?;
                Some(Reference {
                    id: ids.reference(),
                    from: item.from,
                    to: target.id,
                    file: item.file,
                    span: item.span,
                    kind: item.kind,
                    confidence: codeskimr_domain::Confidence::Certain,
                    verified: false,
                })
            })
            .collect())
    }
}

impl MetricsCalculator for World {
    fn measure(
        &self,
        symbols: &[Symbol],
        references: &[Reference],
        counts: &[SymbolCounts],
    ) -> Result<Vec<Metric>, CodedError> {
        self.gate(StageKind::Measure)?;
        self.counted
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(counts.len());
        self.measured
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push((symbols.len(), references.len()));
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
        Ok(self.previous.files.clone())
    }

    fn latest_index(&self, _: ProjectId) -> Result<PreviousIndex, CodedError> {
        Ok(self.previous.index.clone())
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

    fn replace_references(&self, _: ReferenceReplacement) -> Result<Snapshot, CodedError> {
        Err(CodedError::new(ErrorCode::UnexpectedInternalError))
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
    let result =
        scanner(world, &bus).run(&Principal::local_owner(), Job::new(ProjectId::new(7), mode));
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

/// The previous snapshot with `a.rs` unchanged and `b.rs` changed; `c.rs` is gone.
fn a_unchanged_b_changed() -> Result<Arc<World>, CodedError> {
    Ok(Arc::new(World {
        previous: previous_snapshot()?,
        found: Some(vec![("a.rs", false), ("b.rs", true)]),
        ..World::default()
    }))
}

#[test]
fn full_scan_ignores_the_previous_snapshot() -> Result<(), CodedError> {
    let world = a_unchanged_b_changed()?;
    let (result, _) = run(&world, ScanMode::Full);
    let JobResult::Finished(job) = result else {
        panic!("scan did not finish");
    };
    assert_eq!(job.snapshot().stats.files, 2);
    let persisted = world.last_persisted()?;
    assert_eq!(persisted.removed, Vec::<RelativePath>::new());
    assert_eq!(persisted.files.len(), 2);
    assert_eq!(
        persisted.fingerprints.len(),
        2,
        "every analysed callable is fingerprinted"
    );
    Ok(())
}

fn old_file(id: u64, path: &str) -> Result<SourceFile, CodedError> {
    Ok(SourceFile {
        id: FileId::new(id),
        snapshot: SnapshotId::new(9),
        path: model(RelativePath::new(path))?,
        language: model(LanguageId::new("rust"))?,
        content_hash: model(ContentHash::new("old"))?,
        loc: Loc(3),
    })
}

fn old_symbol(id: u64, file: u64, name: &str) -> Result<Symbol, CodedError> {
    Ok(Symbol {
        id: SymbolId::new(id),
        file: FileId::new(file),
        kind: SymbolKind::Function,
        name: model(SymbolName::new(name))?,
        qualified_name: model(QualifiedName::new(format!("m::{name}")))?,
        parent: None,
        visibility: None,
        is_static: false,
        span: span()?,
        signature: None,
        doc: None,
    })
}

/// A call from symbol 200 in file 100.
fn old_call(id: u64, to: u64) -> Result<Reference, CodedError> {
    Ok(Reference {
        id: ReferenceId::new(id),
        from: SymbolId::new(200),
        to: SymbolId::new(to),
        file: FileId::new(100),
        span: span()?,
        kind: ReferenceKind::Call,
        confidence: Confidence::Certain,
        verified: false,
    })
}

/// The newest snapshot: `a.rs` (file 100) calls `b.rs` (file 101), the removed `c.rs` (file 102)
/// and itself.
fn previous_snapshot() -> Result<Previous, CodedError> {
    Ok(Previous {
        files: vec![
            old_file(100, "a.rs")?,
            old_file(101, "b.rs")?,
            old_file(102, "c.rs")?,
        ],
        index: PreviousIndex {
            symbols: vec![
                old_symbol(200, 100, "a")?,
                old_symbol(201, 101, "b")?,
                old_symbol(202, 102, "c")?,
            ],
            references: vec![
                old_call(300, 201)?,
                old_call(301, 202)?,
                old_call(302, 200)?,
            ],
        },
    })
}

#[test]
fn incremental_scan_analyses_only_changed_files_and_resolves_dependents_again()
-> Result<(), CodedError> {
    let world = a_unchanged_b_changed()?;
    let (result, _) = run(&world, ScanMode::Incremental);
    let JobResult::Finished(job) = result else {
        panic!("scan did not finish");
    };
    let persisted = world.last_persisted()?;
    let paths: Vec<_> = persisted.files.iter().map(|f| f.path.as_str()).collect();
    assert_eq!(paths, ["b.rs"], "only the changed file is analysed");
    assert_eq!(
        persisted.fingerprints.len(),
        1,
        "the store carries the fingerprints of the unchanged files"
    );
    assert_eq!(
        persisted
            .removed
            .iter()
            .map(RelativePath::as_str)
            .collect::<Vec<_>>(),
        ["c.rs"]
    );
    // The new `b` and the call from `a` to it, which pointed at the old `b`; the call to the
    // removed `c` finds no target any more and is dropped.
    assert_eq!(persisted.references.len(), 2);
    let new_b = persisted.symbols[0].id;
    assert!(
        persisted
            .references
            .iter()
            .any(|r| r.from.get() == 200 && r.to == new_b)
    );
    assert_eq!(
        (
            job.snapshot().stats.files,
            job.snapshot().stats.symbols,
            job.snapshot().stats.references
        ),
        (2, 2, 3),
        "a.rs and b.rs; a and b; two new references and the call from a to itself carried over"
    );
    Ok(())
}

#[test]
fn metrics_and_resolver_see_the_whole_project_in_an_incremental_scan() -> Result<(), CodedError> {
    let world = a_unchanged_b_changed()?;
    let _ = run(&world, ScanMode::Incremental);
    let measured = world
        .measured
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clone();
    assert_eq!(measured, [(2, 3)]);
    Ok(())
}

#[test]
fn a_file_the_previous_snapshot_does_not_know_is_analysed_even_if_marked_unchanged()
-> Result<(), CodedError> {
    let world = Arc::new(World {
        found: Some(vec![("a.rs", false)]),
        ..World::default()
    });
    let (result, _) = run(&world, ScanMode::Incremental);
    assert!(matches!(result, JobResult::Finished(job) if job.snapshot().stats.files == 1));
    assert_eq!(world.last_persisted()?.files.len(), 1);
    Ok(())
}

#[test]
fn every_stage_reports_how_much_work_it_has_and_finishes_it() {
    let world = Arc::new(World::default());
    let (_, events) = run(&world, ScanMode::Full);
    for stage in StageKind::ALL {
        let last = events.iter().rev().find_map(|event| match event {
            Event::ScanProgressed(p) if p.stage == *stage => Some((p.done, p.total)),
            _ => None,
        });
        let Some((done, total)) = last else {
            panic!("{stage:?} reported nothing");
        };
        assert!(
            total > 0 && done == total,
            "{stage:?} ended at {done}/{total}"
        );
    }
}

#[test]
fn a_stage_reports_its_time_and_repeats_its_last_counts_when_it_is_over() {
    let world = Arc::new(World::default());
    let (_, events) = run(&world, ScanMode::Full);
    for stage in StageKind::ALL {
        let reports: Vec<_> = events
            .iter()
            .filter_map(|event| match event {
                Event::ScanProgressed(p) if p.stage == *stage => Some(p),
                _ => None,
            })
            .collect();
        let [.., before, closing] = reports.as_slice() else {
            panic!("{stage:?} reported less than twice");
        };
        assert_eq!((closing.done, closing.total), (before.done, before.total));
        assert!(
            closing.elapsed_ms >= before.elapsed_ms,
            "{stage:?} ran backwards in time"
        );
    }
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
    let result = scanner(&world, &bus).run(&Principal::local_owner(), job);
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
    let result = scanner(&world, &bus).run(&Principal::local_owner(), job);
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
        rescan: false,
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

#[test]
fn a_reference_that_is_kept_is_not_resolved_a_second_time() -> Result<(), CodedError> {
    let mut previous = previous_snapshot()?;
    // `m::b` is declared in the unchanged a.rs as well; a call to it stays valid, and the call to
    // the `b` in the changed file is resolved again to the same unchanged candidate.
    previous.index.symbols.push(old_symbol(203, 100, "b")?);
    previous
        .index
        .references
        .extend([old_call(303, 203)?, old_call(304, 201)?]);
    let world = Arc::new(World {
        previous,
        found: Some(vec![("a.rs", false), ("b.rs", true)]),
        ..World::default()
    });
    let _ = run(&world, ScanMode::Incremental);
    let calls_to_203 = world
        .last_persisted()?
        .references
        .iter()
        .filter(|r| r.to.get() == 203 && r.from.get() == 200)
        .count();
    assert_eq!(
        calls_to_203, 0,
        "the kept call to 203 is not delivered a second time"
    );
    Ok(())
}

#[test]
fn the_calculator_gets_the_counts_of_the_analysed_files_only() -> Result<(), CodedError> {
    let world = a_unchanged_b_changed()?;
    let _ = run(&world, ScanMode::Incremental);
    let counted = world
        .counted
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clone();
    assert_eq!(
        counted,
        [1],
        "only b.rs was analysed, so only its counts exist"
    );
    Ok(())
}

#[test]
fn a_caller_without_the_scan_permission_is_denied_and_nothing_happens() {
    let world = Arc::new(World::default());
    let bus = EventBus::new();
    let events = bus.subscribe();
    let reader = Principal::limited(PermissionSet::none().with(Permission::Read));
    let result = scanner(&world, &bus).run(&reader, Job::new(ProjectId::new(7), ScanMode::Full));
    let JobResult::Failed(job) = result else {
        panic!("a caller without permission must not scan");
    };
    assert_eq!(job.code(), ErrorCode::PermissionDenied);
    assert_eq!(world.persisted_count(), 0);
    assert_eq!(events.try_iter().count(), 0);
}

#[test]
fn rescan_runs_an_incremental_scan_and_returns_its_snapshot() -> Result<(), CodedError> {
    let world = a_unchanged_b_changed()?;
    let snapshot =
        scanner(&world, &EventBus::new()).rescan(&Principal::local_owner(), ProjectId::new(7))?;
    assert_eq!(snapshot.mode, ScanMode::Incremental);
    assert_eq!(world.persisted_count(), 1);
    Ok(())
}

#[test]
fn rescan_keeps_the_code_of_a_denied_caller_and_of_a_failed_stage() {
    let world = Arc::new(World::default());
    let reader = Principal::limited(PermissionSet::none().with(Permission::Read));
    let denied = scanner(&world, &EventBus::new()).rescan(&reader, ProjectId::new(7));
    assert_eq!(denied.unwrap_err().code(), ErrorCode::PermissionDenied);
    assert_eq!(world.persisted_count(), 0);
    let broken = Arc::new(World {
        fail_at: Some(StageKind::ALL[0]),
        ..World::default()
    });
    let failed =
        scanner(&broken, &EventBus::new()).rescan(&Principal::local_owner(), ProjectId::new(7));
    assert_eq!(failed.unwrap_err().code(), ErrorCode::GitFetchFailed);
}

/// A disk with `count` changed files, `f0000.rs` to `f{count-1}.rs`.
fn many_files(count: usize) -> Vec<(&'static str, bool)> {
    (0..count)
        .map(|n| -> (&'static str, bool) {
            (Box::leak(format!("f{n:04}.rs").into_boxed_str()), true)
        })
        .collect()
}

fn many_world() -> World {
    World {
        found: Some(many_files(200)),
        ..World::default()
    }
}

#[test]
fn many_files_are_analysed_in_discovery_order_with_the_same_ids_every_time()
-> Result<(), CodedError> {
    let mut runs = Vec::new();
    for _ in 0..3 {
        let world = Arc::new(many_world());
        let (result, _) = run(&world, ScanMode::Full);
        assert!(matches!(result, JobResult::Finished(_)));
        let data = world.last_persisted()?;
        let paths: Vec<&str> = data.files.iter().map(|f| f.path.as_str()).collect();
        let expected: Vec<String> = (0..200).map(|n| format!("f{n:04}.rs")).collect();
        assert_eq!(paths, expected);
        let ids: Vec<(u64, u64)> = data
            .files
            .iter()
            .zip(&data.symbols)
            .map(|(file, symbol)| (file.id.get(), symbol.id.get()))
            .collect();
        runs.push(ids);
    }
    assert_eq!(runs.first(), runs.get(1));
    assert_eq!(runs.get(1), runs.get(2));
    Ok(())
}

#[test]
fn progress_of_the_analyze_stage_never_runs_backwards_and_ends_at_the_total() {
    for _ in 0..20 {
        let world = Arc::new(many_world());
        let (_, events) = run(&world, ScanMode::Full);
        let counts: Vec<(u32, u32)> = events
            .iter()
            .filter_map(|event| match event {
                Event::ScanProgressed(p) if p.stage == StageKind::Analyze => {
                    Some((p.done, p.total))
                }
                _ => None,
            })
            .collect();
        assert!(counts.windows(2).all(|pair| pair[0].0 <= pair[1].0));
        assert_eq!(counts.last(), Some(&(200, 200)));
    }
}

#[test]
fn a_failing_file_among_many_ends_the_scan_with_its_code_and_no_snapshot() {
    let world = Arc::new(World {
        fail_at: Some(StageKind::Analyze),
        ..many_world()
    });
    let (result, _) = run(&world, ScanMode::Full);
    assert!(matches!(result, JobResult::Failed(job) if job.code() == ErrorCode::GitFetchFailed));
    assert_eq!(world.persisted_count(), 0);
}

#[test]
fn cancelling_during_the_analysis_of_many_files_writes_nothing() {
    let bus = EventBus::new();
    let job = Job::new(ProjectId::new(7), ScanMode::Full);
    let world = Arc::new(World {
        cancel_at: Some((StageKind::Analyze, job.cancel_token())),
        ..many_world()
    });
    let result = scanner(&world, &bus).run(&Principal::local_owner(), job);
    assert!(matches!(result, JobResult::Cancelled(_)));
    assert_eq!(world.persisted_count(), 0);
}

#[test]
fn a_panicking_analysis_worker_fails_the_scan_with_an_internal_error() {
    let world = Arc::new(World {
        panic_in_analyze: true,
        ..many_world()
    });
    let (result, _) = run(&world, ScanMode::Full);
    assert!(
        matches!(result, JobResult::Failed(job) if job.code() == ErrorCode::UnexpectedInternalError)
    );
    assert_eq!(world.persisted_count(), 0);
}
