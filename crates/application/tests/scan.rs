//! Tests of the scan pipeline, the scanner, the job typestate and the event bus with fake ports.

use std::collections::{BTreeSet, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

#[cfg(test)]
mod common;
#[cfg(test)]
#[path = "common/alerts.rs"]
mod fakes;

use codeskimr_application::cancel::CancelToken;
use codeskimr_application::config::{Config, ConfigLayer, LayerSource};
use codeskimr_application::error::CodedError;
use codeskimr_application::events::{Event, EventBus, StopReason};
use codeskimr_application::ids::{IdSequence, ProjectIds};
use codeskimr_application::level2::{Level2Pass, confirm};
use codeskimr_application::pipeline::{
    Job, JobResult, Level2Run, ReportStep, Scanner, ScannerPorts,
};
use codeskimr_application::ports::{
    AnalyzeRequest, Clock, Confirmation, Confirmed, ConsentStore, DiscoveredFile, FetchedSource,
    FileAnalyzer, FileDiscovery, FileFacts, ImportRequest, Imported, IndexArtifact, IndexJob,
    IndexRunner, IndexerSpec, IndexerStatus, LeftOut, Level2Replacement, MemoryGauge, MemoryLimit,
    MetricsCalculator, PathsReport, PendingReference, PreviousIndex, PreviousState,
    ProjectRepository, ProjectScanner, Receiver, ReferenceImporter, ReferenceResolver, ReportWrite,
    Resolution, ScanReport, ScanReportStore, ScanWarning, SkippedFile, SnapshotData, SnapshotStore,
    SourceProvider, StoredNames, StoredReferences, SymbolFingerprints, ToolSpec, ToolVersion,
};
use codeskimr_application::stage::StageKind;
use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_domain::{
    AlertTrigger, BranchName, CommitHash, Confidence, ContentHash, FileId, Fingerprint,
    Fingerprints, FolderPath, LanguageId, Loc, Metric, MetricKind, MetricValue, ModelError,
    Permission, PermissionSet, Position, Principal, Project, ProjectId, ProjectName, ProjectSource,
    QualifiedName, Reference, ReferenceId, ReferenceKind, RelativePath, ScanLevel, ScanMode,
    ScanOrigin, Snapshot, SnapshotId, SnapshotStats, SourceFile, Span, Symbol, SymbolCounts,
    SymbolId, SymbolKind, SymbolName, SymbolOrigin, SymbolPair, Timestamp,
};
use common::InMemoryConsent;
use fakes::{MemoryAnnotations, MemoryStates, step, symbol};

/// The newest snapshot as the repository reports it.
#[derive(Default, Clone)]
struct Previous {
    files: Vec<SourceFile>,
    index: PreviousIndex,
    /// The names as written in the files, as the store keeps them.
    names: Vec<PendingReference>,
    names_stored: bool,
    /// The number of ambiguous places the snapshot stores; without it a scan reads every reference.
    ambiguous: Option<u64>,
    /// The files the scan of the snapshot left out for good.
    skipped: Vec<SkippedFile>,
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
    /// The project is a folder, not a Git repository.
    folder_project: bool,
    /// The naming rules of the packs, and those the previous snapshot was scanned with.
    rules: (&'static str, Option<&'static str>),
    persisted: Mutex<Vec<SnapshotData>>,
    /// How often a scan confirmed the newest snapshot instead of writing one.
    confirmed: Mutex<Vec<SnapshotId>>,
    /// How many snapshots another process wrote after this scan read the newest one.
    written_meanwhile: u32,
    measured: Mutex<Vec<(usize, usize)>>,
    /// The pairs each measurement got for the references that stayed in the store.
    pairs: Mutex<Vec<Vec<SymbolPair>>>,
    /// Which references each read asked for.
    reads: Mutex<Vec<String>>,
    /// How many names each resolution got.
    resolved: Mutex<Vec<usize>>,
    counted: Mutex<Vec<usize>>,
    /// Milliseconds the fake analyzer says it spent loading packs; each question adds a step.
    pack_load_ms: AtomicU64,
    pack_load_step_ms: u64,
    /// Files whose analysis fails, each with the code it fails with.
    broken: Vec<(&'static str, ErrorCode)>,
    /// The first id the store says is free; another process may have written rows.
    first_free_id: u64,
    /// How many steps of ten milliseconds the resolution waits for the token to be raised, as a
    /// long one would, before it stops; zero for no waiting.
    resolve_wait_steps: u32,
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

/// A file as discovery finds it: an unchanged one has the hash the previous snapshot stores
/// (`old_file`), a changed one another.
fn discovered(path: &str, changed: bool) -> Result<DiscoveredFile, CodedError> {
    let hash = if changed { "h" } else { "old" };
    Ok(DiscoveredFile {
        path: model(RelativePath::new(path))?,
        language: model(LanguageId::new("rust"))?,
        content_hash: model(ContentHash::new(hash))?,
        changed,
    })
}

fn span() -> Result<Span, CodedError> {
    span_at(1)
}

fn span_at(line: u32) -> Result<Span, CodedError> {
    let at = Position { line, column: 1 };
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
    fn pack_load_time(&self) -> Duration {
        let step = self.pack_load_step_ms;
        Duration::from_millis(self.pack_load_ms.fetch_add(step, Ordering::SeqCst))
    }

    fn naming_rules(&self) -> String {
        self.rules.0.to_owned()
    }

    fn analyze(&self, request: &AnalyzeRequest<'_>) -> Result<FileFacts, CodedError> {
        self.gate(StageKind::Analyze)?;
        assert!(!self.panic_in_analyze, "the analyzer panics");
        if let Some((_, code)) = self
            .broken
            .iter()
            .find(|(path, _)| *path == request.file.path.as_str())
        {
            return Err(CodedError::new(*code));
        }
        let file = SourceFile {
            id: request.ids.file(),
            snapshot: request.snapshot,
            path: request.file.path.clone(),
            language: request.file.language.clone(),
            content_hash: request.file.content_hash.clone(),
            loc: Loc(3),
        };
        let symbol = stem_symbol(request, &file)?;
        let pending = vec![PendingReference {
            from: symbol.id,
            file: file.id,
            span: span()?,
            kind: ReferenceKind::Call,
            target: model(QualifiedName::new(symbol.name.as_str()))?,
            receiver: None,
        }];
        Ok(FileFacts {
            file,
            pending,
            fingerprints: vec![fingerprinted(symbol.id)],
            counts: vec![counted(symbol.id)],
            hashes: Vec::new(),
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
        _test_files: &HashSet<FileId>,
        (ids, cancel): (&IdSequence, &CancelToken),
    ) -> Result<Resolution, CodedError> {
        self.gate(StageKind::Resolve)?;
        if self.resolve_wait_steps > 0 {
            for _ in 0..self.resolve_wait_steps {
                if cancel.is_cancelled() {
                    return Err(CodedError::new(ErrorCode::ScanCancelled));
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }
        self.resolved
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(pending.len());
        let references = pending
            .iter()
            .filter_map(|item| {
                let target = symbols
                    .iter()
                    .find(|symbol| symbol.name.as_str() == item.target.as_str())?;
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
            .collect();
        Ok(Resolution {
            references,
            given_up: 0,
        })
    }
}

impl MetricsCalculator for World {
    fn measure(
        &self,
        symbols: &[Symbol],
        references: &[Reference],
        pairs: &[SymbolPair],
        counts: &[SymbolCounts],
    ) -> Result<Vec<Metric>, CodedError> {
        self.gate(StageKind::Measure)?;
        self.pairs
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(pairs.to_vec());
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
        } else if self.folder_project {
            Ok(Project {
                source: ProjectSource::Folder(model(FolderPath::new("/work/demo"))?),
                ..project()?
            })
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

    fn latest_state(&self, _: ProjectId) -> Result<PreviousState, CodedError> {
        Ok(PreviousState {
            files: self.previous.files.clone(),
            symbols: self.previous.index.symbols.clone(),
            snapshot: Some(SnapshotId::new(9)),
            names_stored: self.previous.names_stored,
            ambiguous_places: self.previous.ambiguous,
            skipped: self.previous.skipped.clone(),
        })
    }

    fn stored_references(
        &self,
        _: SnapshotId,
        which: StoredReferences<'_>,
    ) -> Result<Vec<Reference>, CodedError> {
        self.reads
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(format!("{which:?}"));
        let held: HashSet<SymbolId> = self.previous.index.symbols.iter().map(|s| s.id).collect();
        let references = self.previous.index.references.iter();
        Ok(references
            .filter(|r| held.contains(&r.to))
            .filter(|r| match which {
                StoredReferences::To(symbols) => symbols.contains(&r.to),
                StoredReferences::From(symbols) => symbols.contains(&r.from),
                StoredReferences::In(files) => files.contains(&r.file),
                StoredReferences::All => true,
            })
            .copied()
            .collect())
    }

    fn verified_files(&self, _: SnapshotId) -> Result<Vec<FileId>, CodedError> {
        let files: BTreeSet<FileId> = self
            .previous
            .index
            .references
            .iter()
            .filter(|r| r.verified)
            .map(|r| r.file)
            .collect();
        Ok(files.into_iter().collect())
    }

    fn reference_pairs(
        &self,
        _: SnapshotId,
        files: &[FileId],
        without: &[SymbolId],
    ) -> Result<Vec<SymbolPair>, CodedError> {
        let pairs: BTreeSet<SymbolPair> = self
            .previous
            .index
            .references
            .iter()
            .filter(|r| files.contains(&r.file) && !without.contains(&r.from))
            .filter(|r| r.kind != ReferenceKind::Test)
            .map(|r| (r.from, r.to))
            .collect();
        Ok(pairs.into_iter().collect())
    }

    fn stored_names(
        &self,
        _: SnapshotId,
        which: StoredNames<'_>,
    ) -> Result<Vec<PendingReference>, CodedError> {
        Ok(self
            .previous
            .names
            .iter()
            .filter(|name| match which {
                StoredNames::Bases => {
                    matches!(name.kind, ReferenceKind::Inherit | ReferenceKind::Implement)
                }
                StoredNames::ImportsOf(files) => {
                    name.kind == ReferenceKind::Import && files.contains(&name.file)
                }
                StoredNames::Callers(files, callers) => {
                    files.contains(&name.file) && callers.contains(&name.from)
                }
                StoredNames::Targeting(names) => {
                    name.kind != ReferenceKind::Import
                        && (names.iter().any(|n| n == name.target.as_str())
                            || matches!(&name.receiver,
                                Some(Receiver::Typed(r) | Receiver::Named(r))
                                    if names.iter().any(|n| n == r.as_str())))
                }
                StoredNames::ImportsTargeting(names) => {
                    name.kind == ReferenceKind::Import
                        && names.iter().any(|n| n == name.target.as_str())
                }
                StoredNames::OfFiles(files) => files.contains(&name.file),
                StoredNames::All => true,
            })
            .cloned()
            .collect())
    }

    fn latest_naming_rules(&self, _: ProjectId) -> Result<Option<String>, CodedError> {
        Ok(self.rules.1.map(str::to_owned))
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

    fn replace_level2(&self, _: Level2Replacement) -> Result<Snapshot, CodedError> {
        Err(CodedError::new(ErrorCode::UnexpectedInternalError))
    }

    fn confirm(&self, scan: &Confirmation) -> Result<Option<Confirmed>, CodedError> {
        self.gate(StageKind::Persist)?;
        if self.written_meanwhile > 0 {
            return Ok(None);
        }
        self.confirmed
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(scan.snapshot);
        Ok(Some(Confirmed {
            left_out: self.previous.skipped.iter().map(still_named).collect(),
            snapshot: Snapshot {
                id: scan.snapshot,
                project: scan.project,
                commit: scan.commit.clone(),
                created: scan.at,
                mode: ScanMode::Incremental,
                stats: SnapshotStats {
                    files: 3,
                    symbols: 3,
                    references: 3,
                },
            },
            warnings: vec![ScanWarning {
                code: ErrorCode::AmbiguousReference,
                count: 2,
            }],
        }))
    }

    fn next_free_ids(&self, _project: ProjectId) -> Result<(u64, u64), CodedError> {
        Ok((self.first_free_id, 0))
    }
}

impl Clock for World {
    fn now(&self) -> Timestamp {
        Timestamp(42)
    }
}

/// A gauge that reports a fixed value.
struct FixedGauge(u64);

impl MemoryGauge for FixedGauge {
    fn used_mb(&self) -> Option<u64> {
        Some(self.0)
    }
}

/// A gauge that reports `before` once, at the start of the watch, and `now` from then on.
struct GrowingGauge {
    reads: std::sync::atomic::AtomicU32,
    before: u64,
    now: u64,
}

impl GrowingGauge {
    fn new(before: u64, now: u64) -> Self {
        Self {
            reads: std::sync::atomic::AtomicU32::new(0),
            before,
            now,
        }
    }
}

impl MemoryGauge for GrowingGauge {
    fn used_mb(&self) -> Option<u64> {
        let reads = self.reads.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Some(if reads == 0 { self.before } else { self.now })
    }
}

fn scanner(world: &Arc<World>, bus: &EventBus) -> Scanner {
    scanner_with(world, bus, None)
}

fn scanner_with(world: &Arc<World>, bus: &EventBus, memory: Option<MemoryLimit>) -> Scanner {
    let ports = ScannerPorts {
        source: world.clone(),
        discovery: world.clone(),
        analyzer: world.clone(),
        resolver: world.clone(),
        metrics: world.clone(),
        projects: world.clone(),
        store: world.clone(),
        clock: world.clone(),
        max_workers: 0,
        max_left_out_listed: 2,
    };
    let scanner = Scanner::new(ports, &Arc::new(ProjectIds::default()), bus.clone());
    match memory {
        Some(limit) => scanner.with_memory(limit),
        None => scanner,
    }
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

/// The paths, codes and `kept` marks of the files a finished scan left out; none for a scan that
/// did not finish.
fn left_out(result: &JobResult) -> Vec<(String, ErrorCode, bool)> {
    let JobResult::Finished(job) = result else {
        return Vec::new();
    };
    job.paths_report()
        .left_out
        .iter()
        .map(|left| (left.path.as_str().to_owned(), left.code, left.kept))
        .collect()
}

#[test]
fn a_file_that_cannot_be_analysed_is_left_out_and_the_scan_finishes() -> Result<(), CodedError> {
    for code in [
        ErrorCode::FileEncodingUnsupported,
        ErrorCode::ParseTimeout,
        ErrorCode::FileUnreadable,
    ] {
        let world = Arc::new(World {
            broken: vec![("b.rs", code)],
            ..World::default()
        });
        let (result, _) = run(&world, ScanMode::Full);
        assert_eq!(left_out(&result), [("b.rs".to_owned(), code, false)]);
        let JobResult::Finished(job) = result else {
            panic!("scan did not finish for {}", code.id());
        };
        assert_eq!(job.snapshot().stats.files, 1);
        assert_eq!(job.warnings(), [ScanWarning { code, count: 1 }]);
        let stored = world.last_persisted()?;
        assert_eq!(stored.files.len(), 1);
        assert_eq!(stored.warnings, [ScanWarning { code, count: 1 }]);
        let named: Vec<_> = stored
            .left_out
            .iter()
            .map(|left| (left.path.as_str().to_owned(), left.code, left.kept))
            .collect();
        assert_eq!(named, [("b.rs".to_owned(), code, false)]);
    }
    Ok(())
}

#[test]
fn the_store_gets_only_as_many_left_out_files_as_the_limit_allows() -> Result<(), CodedError> {
    let code = ErrorCode::FileEncodingUnsupported;
    let world = Arc::new(World {
        found: Some(vec![
            ("a.rs", true),
            ("b.rs", true),
            ("c.rs", true),
            ("d.rs", true),
            ("e.rs", true),
        ]),
        broken: vec![
            ("b.rs", code),
            ("c.rs", code),
            ("d.rs", code),
            ("e.rs", code),
        ],
        ..World::default()
    });
    let (_, _) = run(&world, ScanMode::Full);
    let stored = world.last_persisted()?;
    assert_eq!(stored.left_out.len(), 2);
    assert_eq!(stored.warnings, [ScanWarning { code, count: 4 }]);
    Ok(())
}

#[test]
fn files_left_out_for_the_same_reason_are_one_warning_with_their_number() {
    let code = ErrorCode::FileEncodingUnsupported;
    let world = Arc::new(World {
        found: Some(vec![("a.rs", true), ("b.rs", true), ("c.rs", true)]),
        broken: vec![("a.rs", code), ("b.rs", code)],
        ..World::default()
    });
    let (result, _) = run(&world, ScanMode::Full);
    let JobResult::Finished(job) = result else {
        panic!("scan did not finish");
    };
    assert_eq!(job.snapshot().stats.files, 1);
    assert_eq!(job.warnings(), [ScanWarning { code, count: 2 }]);
}

#[test]
fn a_scan_in_which_no_file_can_be_analysed_fails_with_the_reason() {
    let code = ErrorCode::FileEncodingUnsupported;
    let world = Arc::new(World {
        broken: vec![("a.rs", code), ("b.rs", code)],
        ..World::default()
    });
    let (result, _) = run(&world, ScanMode::Full);
    let JobResult::Failed(job) = result else {
        panic!("a scan without any file must fail");
    };
    assert_eq!(job.code(), code);
    assert_eq!(world.persisted_count(), 0);
}

#[test]
fn a_failure_that_is_not_about_one_file_still_stops_the_scan() {
    for code in [
        ErrorCode::LanguagePackInvalid,
        ErrorCode::LanguagePackMissing,
        ErrorCode::SourcePathNotFound,
    ] {
        let world = Arc::new(World {
            broken: vec![("b.rs", code)],
            ..World::default()
        });
        let (result, _) = run(&world, ScanMode::Full);
        let JobResult::Failed(job) = result else {
            panic!("scan did not fail for {}", code.id());
        };
        assert_eq!(job.code(), code);
        assert_eq!(world.persisted_count(), 0);
    }
}

/// `a.rs` unchanged, `b.rs` changed and failing with `code` in an incremental scan.
fn b_changed_and_broken(code: ErrorCode) -> Result<Arc<World>, CodedError> {
    Ok(Arc::new(World {
        previous: previous_snapshot()?,
        found: Some(vec![("a.rs", false), ("b.rs", true)]),
        broken: vec![("b.rs", code)],
        ..World::default()
    }))
}

#[test]
fn an_incremental_scan_drops_a_file_whose_failure_will_not_pass() -> Result<(), CodedError> {
    let world = b_changed_and_broken(ErrorCode::FileEncodingUnsupported)?;
    let (result, _) = run(&world, ScanMode::Incremental);
    assert_eq!(
        left_out(&result),
        [("b.rs".to_owned(), ErrorCode::FileEncodingUnsupported, false)]
    );
    let stored = world.last_persisted()?;
    assert_eq!(stored.files, Vec::<SourceFile>::new());
    assert!(stored.removed.iter().any(|path| path.as_str() == "b.rs"));
    Ok(())
}

#[test]
fn an_incremental_scan_keeps_what_it_knew_of_a_file_whose_failure_may_pass()
-> Result<(), CodedError> {
    for code in [ErrorCode::ParseTimeout, ErrorCode::FileUnreadable] {
        let world = b_changed_and_broken(code)?;
        let (result, _) = run(&world, ScanMode::Incremental);
        assert_eq!(left_out(&result), [("b.rs".to_owned(), code, true)]);
        let JobResult::Finished(job) = result else {
            panic!("scan did not finish");
        };
        // Both files of the previous snapshot are still there, with their symbols.
        assert_eq!(job.snapshot().stats.files, 2);
        assert_eq!(job.snapshot().stats.symbols, 2);
        let stored = world.last_persisted()?;
        assert_eq!(stored.files, Vec::<SourceFile>::new());
        assert!(!stored.removed.iter().any(|path| path.as_str() == "b.rs"));
        assert_eq!(stored.warnings, [ScanWarning { code, count: 1 }]);
    }
    Ok(())
}

/// `b.rs` failing with `code` in a full scan after a snapshot made under the rules `kept`.
fn b_broken_in_a_full_scan(
    code: ErrorCode,
    kept: Option<&'static str>,
) -> Result<Arc<World>, CodedError> {
    Ok(Arc::new(World {
        previous: previous_snapshot()?,
        found: Some(vec![("a.rs", false), ("b.rs", false)]),
        broken: vec![("b.rs", code)],
        rules: ("csharp=namespace", kept),
        ..World::default()
    }))
}

#[test]
fn a_full_scan_keeps_what_it_knew_of_a_file_whose_failure_may_pass() -> Result<(), CodedError> {
    for code in [ErrorCode::ParseTimeout, ErrorCode::FileUnreadable] {
        let world = b_broken_in_a_full_scan(code, Some("csharp=namespace"))?;
        let (result, _) = run(&world, ScanMode::Full);
        assert_eq!(left_out(&result), [("b.rs".to_owned(), code, true)]);
        let JobResult::Finished(job) = result else {
            panic!("scan did not finish");
        };
        assert_eq!(job.snapshot().stats.files, 2);
        assert_eq!(job.snapshot().stats.symbols, 2);
        // The store copies the former rows of `b.rs` only into an incremental snapshot.
        assert_eq!(job.snapshot().mode, ScanMode::Incremental);
        let stored = world.last_persisted()?;
        let paths: Vec<_> = stored.files.iter().map(|f| f.path.as_str()).collect();
        assert_eq!(paths, ["a.rs"], "every other file is analysed again");
        assert!(!stored.removed.iter().any(|path| path.as_str() == "b.rs"));
        assert_eq!(stored.parse_micros, Vec::new());
    }
    Ok(())
}

#[test]
fn a_full_scan_cannot_keep_a_file_of_a_snapshot_with_other_naming_rules() -> Result<(), CodedError>
{
    for kept in [None, Some("csharp=file")] {
        let world = b_broken_in_a_full_scan(ErrorCode::ParseTimeout, kept)?;
        let (result, _) = run(&world, ScanMode::Full);
        assert_eq!(
            left_out(&result),
            [("b.rs".to_owned(), ErrorCode::ParseTimeout, false)]
        );
        let JobResult::Finished(job) = result else {
            panic!("scan did not finish");
        };
        assert_eq!(job.snapshot().mode, ScanMode::Full);
        assert_eq!(job.snapshot().stats.files, 1);
    }
    Ok(())
}

#[test]
fn a_full_scan_drops_a_file_whose_failure_will_not_pass() -> Result<(), CodedError> {
    let code = ErrorCode::FileEncodingUnsupported;
    let world = b_broken_in_a_full_scan(code, Some("csharp=namespace"))?;
    let (result, _) = run(&world, ScanMode::Full);
    assert_eq!(left_out(&result), [("b.rs".to_owned(), code, false)]);
    let JobResult::Finished(job) = result else {
        panic!("scan did not finish");
    };
    assert_eq!(job.snapshot().mode, ScanMode::Full);
    assert_eq!(job.snapshot().stats.files, 1);
    Ok(())
}

#[test]
fn a_scan_starts_above_the_ids_another_process_stored() -> Result<(), CodedError> {
    let world = Arc::new(World {
        first_free_id: 5_000,
        ..World::default()
    });
    let (result, _) = run(&world, ScanMode::Full);
    let JobResult::Finished(job) = result else {
        panic!("scan did not finish");
    };
    assert!(job.snapshot().id.get() >= 5_000);
    let stored = world.last_persisted()?;
    assert!(stored.files.iter().all(|file| file.id.get() >= 5_000));
    assert!(stored.symbols.iter().all(|symbol| symbol.id.get() >= 5_000));
    Ok(())
}

#[test]
fn the_end_event_tells_how_long_loading_packs_took() {
    let world = Arc::new(World {
        pack_load_step_ms: 40,
        ..World::default()
    });
    let (_, events) = run(&world, ScanMode::Full);
    let Some(Event::SnapshotCreated(created)) = events.last() else {
        panic!("no end event");
    };
    assert_eq!(created.pack_load, Duration::from_millis(40));
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

fn stem_symbol(request: &AnalyzeRequest<'_>, file: &SourceFile) -> Result<Symbol, CodedError> {
    let stem = request.file.path.as_str().trim_end_matches(".rs");
    Ok(Symbol {
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
        origin: SymbolOrigin::Syntax,
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
        origin: SymbolOrigin::Syntax,
    })
}

/// A call from symbol 200 in file 100.
fn old_call(id: u64, to: u64) -> Result<Reference, CodedError> {
    Ok(Reference {
        id: ReferenceId::new(id),
        from: SymbolId::new(200),
        to: SymbolId::new(to),
        file: FileId::new(100),
        span: span_at(u32::try_from(id).unwrap_or_default())?,
        kind: ReferenceKind::Call,
        confidence: Confidence::Certain,
        verified: false,
    })
}

/// The name `a` (symbol 200 in file 100) wrote for a call of `target`, in the place of the call
/// `id`.
fn old_name(target: &str, id: u32) -> Result<PendingReference, CodedError> {
    Ok(PendingReference {
        from: SymbolId::new(200),
        file: FileId::new(100),
        span: span_at(id)?,
        kind: ReferenceKind::Call,
        target: model(QualifiedName::new(target))?,
        receiver: None,
    })
}

/// The newest snapshot: `a.rs` (file 100) calls `b.rs` (file 101), the removed `c.rs` (file 102)
/// and itself.
fn previous_snapshot() -> Result<Previous, CodedError> {
    Ok(Previous {
        names_stored: true,
        // No place of this snapshot matches several symbols, so scans over it read only the
        // references they need (ADR 0086).
        ambiguous: Some(0),
        names: vec![
            old_name("b", 300)?,
            old_name("c", 301)?,
            old_name("a", 302)?,
        ],
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
        skipped: Vec::new(),
    })
}

/// [`previous_snapshot`] with a fourth file `d.rs` (file 103) that no change reaches: its symbol
/// `d` refers to `a` and, in one place that matches two symbols, to `a` or itself.
fn previous_with_an_untouched_file(counted: Option<u64>) -> Result<Arc<World>, CodedError> {
    let mut previous = previous_snapshot()?;
    previous.ambiguous = counted;
    previous.files.push(old_file(103, "d.rs")?);
    previous.index.symbols.push(old_symbol(203, 103, "d")?);
    let in_d = |id: u64, line: u32, to: u64, confidence| -> Result<Reference, CodedError> {
        Ok(Reference {
            id: ReferenceId::new(id),
            from: SymbolId::new(203),
            to: SymbolId::new(to),
            file: FileId::new(103),
            span: span_at(line)?,
            kind: ReferenceKind::Call,
            confidence,
            verified: false,
        })
    };
    previous.index.references.extend([
        in_d(310, 5, 200, Confidence::Certain)?,
        in_d(311, 6, 200, Confidence::Probable)?,
        in_d(312, 6, 203, Confidence::Probable)?,
    ]);
    Ok(Arc::new(World {
        previous,
        found: Some(vec![("a.rs", false), ("b.rs", true), ("d.rs", false)]),
        ..World::default()
    }))
}

impl World {
    /// Which references the scan asked the store for, one text per read.
    fn reads(&self) -> Vec<String> {
        self.reads
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// The pairs the last measurement got for the references that stayed in the store.
    fn last_pairs(&self) -> Vec<SymbolPair> {
        let pairs = self.pairs.lock().unwrap_or_else(PoisonError::into_inner);
        pairs.last().cloned().unwrap_or_default()
    }
}

#[test]
fn an_incremental_scan_leaves_the_references_of_untouched_files_in_the_store()
-> Result<(), CodedError> {
    let world = previous_with_an_untouched_file(Some(1))?;
    let (result, _) = run(&world, ScanMode::Incremental);
    assert!(matches!(result, JobResult::Finished(_)));
    let (a, b, c) = (SymbolId::new(200), SymbolId::new(201), SymbolId::new(202));
    let left = [FileId::new(101), FileId::new(102)];
    let expected = [
        format!("{:?}", StoredReferences::To(&[b, c])),
        format!("{:?}", StoredReferences::From(&[a])),
        format!("{:?}", StoredReferences::In(&left)),
    ];
    assert_eq!(
        world.reads(),
        expected,
        "nothing that starts in the symbol of the untouched file is read"
    );
    let d = SymbolId::new(203);
    assert_eq!(
        world.last_pairs(),
        [(d, SymbolId::new(200)), (d, d)],
        "coupling gets the references of the untouched file as pairs"
    );
    let persisted = world.last_persisted()?;
    assert_eq!(
        persisted.ambiguous_places,
        Some(1),
        "the place in the untouched file is counted without its references"
    );
    let ambiguous = ScanWarning {
        code: ErrorCode::AmbiguousReference,
        count: 1,
    };
    assert_eq!(persisted.warnings, [ambiguous]);
    Ok(())
}

#[test]
fn without_a_stored_count_an_incremental_scan_reads_every_reference() -> Result<(), CodedError> {
    let world = previous_with_an_untouched_file(None)?;
    let (result, _) = run(&world, ScanMode::Incremental);
    assert!(matches!(result, JobResult::Finished(_)));
    let all = format!("{:?}", StoredReferences::All);
    assert!(world.reads().contains(&all), "{:?}", world.reads());
    assert_eq!(world.last_pairs(), []);
    assert_eq!(world.last_persisted()?.ambiguous_places, Some(1));
    Ok(())
}

#[test]
fn a_stored_count_and_a_count_over_every_reference_give_the_same_snapshot() -> Result<(), CodedError>
{
    let narrow = previous_with_an_untouched_file(Some(1))?;
    let wide = previous_with_an_untouched_file(None)?;
    for world in [&narrow, &wide] {
        let (result, _) = run(world, ScanMode::Incremental);
        assert!(matches!(result, JobResult::Finished(_)));
    }
    let (narrow, wide) = (narrow.last_persisted()?, wide.last_persisted()?);
    assert_eq!(narrow.references, wide.references);
    assert_eq!(narrow.dropped_references, wide.dropped_references);
    assert_eq!(narrow.warnings, wide.warnings);
    assert_eq!(narrow.ambiguous_places, wide.ambiguous_places);
    Ok(())
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
fn an_incremental_scan_that_finds_nothing_changed_confirms_the_snapshot_and_writes_nothing()
-> Result<(), CodedError> {
    let world = Arc::new(nothing_changed()?);
    let (result, events) = run(&world, ScanMode::Incremental);
    let JobResult::Finished(job) = result else {
        panic!("scan did not finish");
    };
    assert_eq!(world.persisted_count(), 0, "no snapshot is written");
    assert_eq!(calls_of(&world.confirmed), 1);
    assert_eq!(
        job.snapshot().id,
        SnapshotId::new(9),
        "the newest one stays"
    );
    assert_eq!(
        job.snapshot().created,
        Timestamp(42),
        "with the time of the scan"
    );
    assert_eq!(job.snapshot().mode, ScanMode::Incremental);
    assert_eq!(calls_of(&world.resolved), 0, "no name is resolved again");
    assert_eq!(calls_of(&world.measured), 0, "nothing is measured again");
    assert!(
        events
            .iter()
            .any(|event| matches!(event, Event::SnapshotCreated(created) if created.unchanged)),
        "the listeners still learn that the scan ended"
    );
    Ok(())
}

/// A world in which discovery sees the three files of the previous snapshot as they are stored.
fn nothing_changed() -> Result<World, CodedError> {
    Ok(World {
        previous: previous_snapshot()?,
        found: Some(vec![("a.rs", false), ("b.rs", false), ("c.rs", false)]),
        ..World::default()
    })
}

#[test]
fn a_confirmed_snapshot_gives_the_scan_its_stored_warnings() -> Result<(), CodedError> {
    let world = Arc::new(nothing_changed()?);
    let (result, _) = run(&world, ScanMode::Incremental);
    let JobResult::Finished(job) = result else {
        panic!("scan did not finish");
    };
    let warnings: Vec<_> = job.warnings().iter().map(|w| (w.code, w.count)).collect();
    assert_eq!(warnings, [(ErrorCode::AmbiguousReference, 2)]);
    Ok(())
}

#[test]
fn a_snapshot_written_meanwhile_fails_the_scan_that_found_nothing_changed() -> Result<(), CodedError>
{
    let world = Arc::new(World {
        written_meanwhile: 1,
        ..nothing_changed()?
    });
    let (result, _) = run(&world, ScanMode::Incremental);
    let JobResult::Failed(job) = result else {
        panic!("scan did not fail");
    };
    assert_eq!(job.code(), ErrorCode::TransactionConflict);
    assert_eq!(world.persisted_count(), 0);
    Ok(())
}

#[test]
fn a_file_with_another_hash_is_a_change_whatever_discovery_marked() -> Result<(), CodedError> {
    let mut previous = previous_snapshot()?;
    previous.files[1].content_hash = model(ContentHash::new("older"))?;
    let world = Arc::new(World {
        previous,
        ..nothing_changed()?
    });
    let (result, _) = run(&world, ScanMode::Incremental);
    assert!(matches!(result, JobResult::Finished(_)));
    assert_eq!(calls_of(&world.confirmed), 0);
    assert_eq!(world.persisted_count(), 1, "the scan goes the whole way");
    Ok(())
}

/// What the list of left-out files of a snapshot says of a file its scan left out for good.
fn still_named(file: &SkippedFile) -> LeftOut {
    LeftOut {
        path: file.path.clone(),
        code: file.code,
        kept: false,
    }
}

/// A file as a scan that left it out for good recorded it.
fn skipped_file(path: &str, hash: &str) -> Result<SkippedFile, CodedError> {
    Ok(SkippedFile {
        path: model(RelativePath::new(path))?,
        language: model(LanguageId::new("rust"))?,
        content_hash: model(ContentHash::new(hash))?,
        code: ErrorCode::FileEncodingUnsupported,
    })
}

/// The previous snapshot, whose scan left `d.rs` out for good with the hash `hash`, and a disk
/// that holds the three files of the snapshot unchanged and `found` besides. Discovery marks a
/// file the snapshot does not hold as changed, with the hash `h`.
fn after_a_skipped_file(hash: &str, found: &[&'static str]) -> Result<Arc<World>, CodedError> {
    let mut previous = previous_snapshot()?;
    previous.skipped = vec![skipped_file("d.rs", hash)?];
    let mut on_disk = vec![("a.rs", false), ("b.rs", false), ("c.rs", false)];
    on_disk.extend(found.iter().map(|path| (*path, true)));
    Ok(Arc::new(World {
        previous,
        found: Some(on_disk),
        broken: vec![("d.rs", ErrorCode::FileEncodingUnsupported)],
        ..World::default()
    }))
}

#[test]
fn a_file_left_out_for_good_that_looks_the_same_is_no_change() -> Result<(), CodedError> {
    let world = after_a_skipped_file("h", &["d.rs"])?;
    let (result, _) = run(&world, ScanMode::Incremental);
    assert_eq!(calls_of(&world.confirmed), 1);
    assert_eq!(world.persisted_count(), 0, "no snapshot is written");
    let named = ("d.rs".to_owned(), ErrorCode::FileEncodingUnsupported, false);
    assert_eq!(left_out(&result), [named], "the scan still names the file");
    Ok(())
}

#[test]
fn a_file_left_out_for_good_is_a_change_once_it_differs_or_is_gone() -> Result<(), CodedError> {
    for world in [
        after_a_skipped_file("other", &["d.rs"])?,
        after_a_skipped_file("h", &[])?,
        after_a_skipped_file("h", &["d.rs", "e.rs"])?,
    ] {
        let (result, _) = run(&world, ScanMode::Incremental);
        assert!(matches!(result, JobResult::Finished(_)));
        assert_eq!(calls_of(&world.confirmed), 0);
        assert_eq!(world.persisted_count(), 1, "the scan goes the whole way");
    }
    Ok(())
}

#[test]
fn a_scan_records_the_files_it_left_out_for_good_with_their_hash() -> Result<(), CodedError> {
    let world = b_changed_and_broken(ErrorCode::FileEncodingUnsupported)?;
    run(&world, ScanMode::Incremental);
    assert_eq!(
        world.last_persisted()?.skipped,
        [skipped_file("b.rs", "h")?]
    );
    for code in LeftOut::MAY_PASS {
        let world = b_changed_and_broken(*code)?;
        run(&world, ScanMode::Incremental);
        let skipped = world.last_persisted()?.skipped;
        assert_eq!(skipped, Vec::new(), "the next scan tries the file again");
    }
    Ok(())
}

#[test]
fn a_scan_of_some_paths_keeps_the_left_out_files_it_does_not_look_at() -> Result<(), CodedError> {
    let world = after_a_skipped_file("h", &["d.rs"])?;
    let paths = ["b.rs".to_owned()];
    let job = Job::new(ProjectId::new(7), ScanMode::Incremental).with_paths(&paths);
    let result = scanner(&world, &EventBus::new()).run(&Principal::local_owner(), job);
    assert!(matches!(result, JobResult::Finished(_)));
    assert_eq!(calls_of(&world.confirmed), 1, "b.rs is as it was");

    let mut previous = previous_snapshot()?;
    previous.skipped = vec![skipped_file("d.rs", "h")?];
    let world = Arc::new(World {
        previous,
        found: Some(vec![("a.rs", false), ("b.rs", true), ("c.rs", false)]),
        ..World::default()
    });
    let job = Job::new(ProjectId::new(7), ScanMode::Incremental).with_paths(&paths);
    let result = scanner(&world, &EventBus::new()).run(&Principal::local_owner(), job);
    assert!(matches!(result, JobResult::Finished(_)));
    let stored = world.last_persisted()?;
    assert_eq!(stored.skipped, [skipped_file("d.rs", "h")?]);
    // The snapshot goes on naming and counting the file, or the scan after it would confirm a
    // snapshot that says nothing of it.
    assert_eq!(stored.left_out, [still_named(&stored.skipped[0])]);
    let code = ErrorCode::FileEncodingUnsupported;
    assert!(stored.warnings.contains(&ScanWarning { code, count: 1 }));
    Ok(())
}

/// How many calls the fake recorded in `log`.
fn calls_of<T>(log: &Mutex<Vec<T>>) -> usize {
    log.lock().unwrap_or_else(PoisonError::into_inner).len()
}

#[test]
fn a_file_that_is_gone_is_a_change() -> Result<(), CodedError> {
    let world = Arc::new(World {
        previous: previous_snapshot()?,
        found: Some(vec![("a.rs", false), ("b.rs", false)]),
        ..World::default()
    });
    let (result, _) = run(&world, ScanMode::Incremental);
    assert!(matches!(result, JobResult::Finished(_)));
    assert_eq!(
        world.persisted_count(),
        1,
        "the snapshot without c.rs is written"
    );
    Ok(())
}

/// The world of `a_unchanged_b_changed` over this previous snapshot.
fn world_over(previous: Previous) -> Arc<World> {
    Arc::new(World {
        previous,
        found: Some(vec![("a.rs", false), ("b.rs", true)]),
        ..World::default()
    })
}

#[test]
fn a_snapshot_without_stored_names_is_followed_by_a_full_scan() -> Result<(), CodedError> {
    let mut previous = previous_snapshot()?;
    previous.names_stored = false;
    let world = world_over(previous);
    let (result, _) = run(&world, ScanMode::Incremental);
    let JobResult::Finished(job) = result else {
        panic!("scan did not finish");
    };
    assert_eq!(job.snapshot().mode, ScanMode::Full);
    assert_eq!(world.last_persisted()?.files.len(), 2);
    Ok(())
}

#[test]
fn a_name_that_found_nothing_before_finds_a_new_target_in_a_changed_file() -> Result<(), CodedError>
{
    let mut previous = previous_snapshot()?;
    // `a` wrote the name `b` before, but there was no `b` then, so no reference was stored.
    previous.index.references.retain(|r| r.id.get() != 300);
    previous.index.symbols.retain(|s| s.id.get() != 201);
    let world = world_over(previous);
    let _ = run(&world, ScanMode::Incremental);
    let persisted = world.last_persisted()?;
    let new_b = persisted.symbols[0].id;
    assert!(
        persisted
            .references
            .iter()
            .any(|r| r.from.get() == 200 && r.to == new_b),
        "the unchanged caller reaches the new target"
    );
    Ok(())
}

#[test]
fn a_kept_reference_that_the_full_resolution_no_longer_produces_is_dropped()
-> Result<(), CodedError> {
    let mut previous = previous_snapshot()?;
    // The stored call of `a` to itself is now a name that looks up the removed `c`, so the
    // resolution is due and finds nothing for it.
    previous.names.retain(|name| name.target.as_str() != "a");
    previous.names.push(old_name("c", 302)?);
    let world = world_over(previous);
    let _ = run(&world, ScanMode::Incremental);
    let persisted = world.last_persisted()?;
    let dropped: Vec<u64> = persisted
        .dropped_references
        .iter()
        .map(|id| id.get())
        .collect();
    assert_eq!(dropped, [302]);
    Ok(())
}

#[test]
fn a_kept_reference_that_is_produced_again_is_neither_written_nor_dropped() -> Result<(), CodedError>
{
    let world = a_unchanged_b_changed()?;
    let _ = run(&world, ScanMode::Incremental);
    let persisted = world.last_persisted()?;
    assert_eq!(persisted.dropped_references, Vec::new());
    assert!(
        persisted.references.iter().all(|r| r.to.get() != 200),
        "the call from a to itself stays in the store as it is"
    );
    Ok(())
}

fn verified(reference: Reference) -> Reference {
    Reference {
        verified: true,
        ..reference
    }
}

#[test]
fn level_two_references_stay_and_one_whose_target_is_gone_falls_back_to_level_one()
-> Result<(), CodedError> {
    let mut previous = previous_snapshot()?;
    previous.index.references = vec![verified(old_call(300, 201)?), verified(old_call(302, 200)?)];
    // Level 2 reported nothing for the call of `c`; level 1 must not add what it replaced.
    previous.names.push(old_name("a", 302)?);
    let world = world_over(previous);
    let (result, _) = run(&world, ScanMode::Incremental);
    let JobResult::Finished(job) = result else {
        panic!("scan did not finish");
    };
    let persisted = world.last_persisted()?;
    assert_eq!(
        persisted.dropped_references,
        Vec::new(),
        "the verified call of a stays"
    );
    let new_b = persisted.symbols[0].id;
    let again: Vec<bool> = persisted
        .references
        .iter()
        .filter(|r| r.from.get() == 200 && r.to == new_b)
        .map(|r| r.verified)
        .collect();
    assert_eq!(
        again,
        [false],
        "the discarded verified call is an unchecked level 1 one"
    );
    assert_eq!(persisted.references.len(), 2, "that and the call inside b");
    assert_eq!(job.snapshot().stats.references, 3);
    Ok(())
}

/// How many names the last scan handed to the resolver.
fn resolved_names(world: &World) -> Vec<usize> {
    world
        .resolved
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clone()
}

/// A symbol that level 2 created (ADR 0055).
fn created_symbol(id: u64, file: u64, qualified: &str) -> Result<Symbol, CodedError> {
    Ok(Symbol {
        qualified_name: model(QualifiedName::new(qualified))?,
        origin: SymbolOrigin::Index,
        ..old_symbol(id, file, "Gen")?
    })
}

/// What a scan stored as references, as `(from, to, span)` and without the ids it made up.
fn stored_edges(world: &World) -> Result<Vec<(u64, u64, Span)>, CodedError> {
    let mut edges: Vec<(u64, u64, Span)> = world
        .last_persisted()?
        .references
        .iter()
        .map(|r| (r.from.get(), r.to.get(), r.span))
        .collect();
    edges.sort_by_key(|(from, to, _)| (*from, *to));
    Ok(edges)
}

#[test]
fn a_scan_gives_the_same_level_one_result_whether_level_two_ran_before_or_not()
-> Result<(), CodedError> {
    // Oracle: the same incremental scan over the same snapshot, once plain and once with the
    // symbols and references that level 2 left in it. 210 has the name of the changed `b`.
    let plain = world_over(previous_snapshot()?);
    let mut with_level_two = previous_snapshot()?;
    with_level_two
        .index
        .symbols
        .push(created_symbol(210, 100, "m::b")?);
    with_level_two
        .index
        .symbols
        .push(created_symbol(211, 101, "m::Gen")?);
    with_level_two.index.references.push(old_call(310, 210)?);
    with_level_two.index.references.push(old_call(311, 211)?);
    let after_level_two = world_over(with_level_two);
    let _ = run(&plain, ScanMode::Incremental);
    let _ = run(&after_level_two, ScanMode::Incremental);
    assert_eq!(stored_edges(&plain)?, stored_edges(&after_level_two)?);
    assert_eq!(resolved_names(&plain), resolved_names(&after_level_two));
    Ok(())
}

#[test]
fn a_level_two_reference_from_an_unchanged_file_to_a_symbol_of_a_changed_file_is_dropped()
-> Result<(), CodedError> {
    let mut previous = previous_snapshot()?;
    previous
        .index
        .symbols
        .push(created_symbol(211, 101, "m::Gen")?);
    previous
        .index
        .references
        .push(verified(old_call(311, 211)?));
    let world = world_over(previous);
    let (result, _) = run(&world, ScanMode::Incremental);
    let JobResult::Finished(job) = result else {
        panic!("scan did not finish");
    };
    let persisted = world.last_persisted()?;
    assert!(
        persisted.references.iter().all(|r| r.to.get() != 211),
        "level 1 does not make the reference again"
    );
    assert_eq!(
        job.snapshot().stats.symbols,
        2,
        "a and the new b; 211 left with its file"
    );
    Ok(())
}

/// The module symbol of a file, named after the file as level 1 does (`lib`, `mod`).
fn module_symbol(id: u64, file: u64, name: &str) -> Result<Symbol, CodedError> {
    Ok(Symbol {
        kind: SymbolKind::Module,
        ..old_symbol(id, file, name)?
    })
}

#[test]
fn a_level_two_use_of_the_module_of_a_changed_file_is_dropped_until_level_two_runs_again()
-> Result<(), CodedError> {
    // ADR 0061: `a.rs` (unchanged) names the module of `b.rs` (changed), as `use crate::b` does
    // in a `lib.rs` or `mod.rs`. Level 1 does not make that reference again; the next level 2 pass
    // does, from the new module symbol.
    let mut previous = previous_snapshot()?;
    previous.index.symbols.push(module_symbol(212, 101, "mod")?);
    previous.index.references.push(Reference {
        kind: ReferenceKind::Read,
        ..verified(old_call(312, 212)?)
    });
    let world = world_over(previous);
    let (result, _) = run(&world, ScanMode::Incremental);
    let JobResult::Finished(_) = result else {
        panic!("scan did not finish");
    };
    let persisted = world.last_persisted()?;
    assert!(
        persisted.references.iter().all(|r| r.to.get() != 212),
        "the module of the changed file left with its file"
    );
    assert!(
        persisted.symbols.iter().all(|s| s.id.get() != 212),
        "and its symbol is not carried"
    );
    Ok(())
}

#[test]
fn a_change_that_keeps_the_declarations_resolves_only_the_names_that_pointed_into_it()
-> Result<(), CodedError> {
    // `c.rs` is not gone in this snapshot, so the changed `b.rs` declares what it declared before.
    let mut previous = previous_snapshot()?;
    previous.files.retain(|f| f.path.as_str() != "c.rs");
    previous.index.symbols.retain(|s| s.file.get() != 102);
    previous.index.references.retain(|r| r.to.get() != 202);
    previous.names.retain(|n| n.target.as_str() != "c");
    let world = world_over(previous);
    let _ = run(&world, ScanMode::Incremental);
    // The new call in `b.rs` and the call of `a` that pointed at the old `b`; the call of `a` to
    // itself is not resolved again.
    assert_eq!(resolved_names(&world), [2]);
    let persisted = world.last_persisted()?;
    assert_eq!(persisted.dropped_references, Vec::new());
    assert_eq!(persisted.references.len(), 2);
    Ok(())
}

#[test]
fn a_gone_declaration_resolves_the_names_that_looked_it_up_again() -> Result<(), CodedError> {
    let world = a_unchanged_b_changed()?;
    let _ = run(&world, ScanMode::Incremental);
    // `c.rs` is gone: the new call of `b.rs`, the name of `a` that looked up `c`, and the name that
    // pointed at the replaced symbol of `b`; the call of `a` to itself is not resolved again.
    assert_eq!(resolved_names(&world), [3]);
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
fn every_stage_has_its_own_label_in_execution_order() {
    let expected = [
        "fetch", "discover", "analyze", "resolve", "measure", "persist",
    ];
    let labels: Vec<_> = StageKind::ALL.iter().map(|s| s.label()).collect();
    assert_eq!(labels, expected);
}

#[test]
fn a_reference_that_is_kept_is_not_resolved_a_second_time() -> Result<(), CodedError> {
    let mut previous = previous_snapshot()?;
    // `m::b` is declared in the unchanged a.rs as well; the call at 303 to it stays valid, and the
    // name is resolved again to the same unchanged candidate and the new `b`.
    previous.index.symbols.push(old_symbol(203, 100, "b")?);
    previous.index.references.push(old_call(303, 203)?);
    previous.names.push(old_name("b", 303)?);
    let world = world_over(previous);
    let _ = run(&world, ScanMode::Incremental);
    let calls_to_203 = world
        .last_persisted()?
        .references
        .iter()
        .filter(|r| r.to.get() == 203 && r.span.start().line == 303)
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

/// Counts the runs it is asked for and fails each, so that nothing is imported.
struct CountingRunner(Mutex<u32>);

impl IndexRunner for CountingRunner {
    fn check(&self, _: &IndexerSpec) -> IndexerStatus {
        IndexerStatus::ready()
    }

    fn run(&self, _: &IndexJob<'_>, _: &CancelToken) -> Result<IndexArtifact, CodedError> {
        *self.0.lock().unwrap_or_else(PoisonError::into_inner) += 1;
        Err(CodedError::new(ErrorCode::UnexpectedInternalError))
    }
}

struct NoImport;

impl ReferenceImporter for NoImport {
    fn import(&self, _: &ImportRequest<'_>) -> Result<Imported, CodedError> {
        Err(CodedError::new(ErrorCode::UnexpectedInternalError))
    }
}

fn rust_indexer() -> IndexerSpec {
    let tool = ToolSpec {
        program: "rust-analyzer".to_owned(),
        version_args: Vec::new(),
        min_version: ToolVersion::default(),
    };
    IndexerSpec {
        name: "rust-analyzer".to_owned(),
        indexer: tool,
        toolchain: Vec::new(),
        args: Vec::new(),
        env_allow: Vec::new(),
        home_dirs: Vec::new(),
        env: Vec::new(),
        memory_env: Vec::new(),
        memory_markers: Vec::new(),
        project_files: Vec::new(),
        install_hint: String::new(),
        symbol_kinds: Vec::new(),
        separator: String::new(),
        library_root: String::new(),
    }
}

/// A scanner with a confirmed second pass whose runner counts, and that counter.
fn level2_scanner(world: &Arc<World>) -> Result<(Scanner, Arc<CountingRunner>), CodedError> {
    level2_scanner_over(world, &Arc::new(InMemoryConsent::default()))
}

/// The same over the consent that the caller keeps, so that a test can revoke it.
fn level2_scanner_over(
    world: &Arc<World>,
    store: &Arc<InMemoryConsent>,
) -> Result<(Scanner, Arc<CountingRunner>), CodedError> {
    level2_scanner_on(world, store, &EventBus::new())
}

/// The same publishing on `bus`, for the scanner and for the second pass.
fn level2_scanner_on(
    world: &Arc<World>,
    store: &Arc<InMemoryConsent>,
    bus: &EventBus,
) -> Result<(Scanner, Arc<CountingRunner>), CodedError> {
    let consent: Arc<dyn ConsentStore> = store.clone();
    confirm(
        &Principal::local_owner(),
        consent.as_ref(),
        ProjectId::new(7),
    )?;
    let runner = Arc::new(CountingRunner(Mutex::default()));
    let ids = Arc::new(ProjectIds::default());
    let pass = Level2Pass {
        runner: runner.clone(),
        importer: Arc::new(NoImport),
        consent,
        discovery: world.clone(),
        projects: world.clone(),
        store: world.clone(),
        ids,
        max_file_bytes: u64::from(u32::MAX),
        settings: Config::resolve(&[ConfigLayer::parse(
            LayerSource::User,
            "[level2]\nenabled = true\n",
        )
        .map_err(CodedError::caused_by)?])
        .map_err(CodedError::caused_by)?
        .level2,
        indexers: vec![(model(LanguageId::new("rust"))?, rust_indexer())],
        events: bus.clone(),
    };
    Ok((scanner(world, bus).with_level2(pass), runner))
}

fn runs(runner: &CountingRunner) -> u32 {
    *runner.0.lock().unwrap_or_else(PoisonError::into_inner)
}

#[test]
fn only_a_job_that_wants_level_two_starts_an_indexer() -> Result<(), CodedError> {
    let world = Arc::new(World {
        previous: previous_snapshot()?,
        ..World::default()
    });
    let (scanner, runner) = level2_scanner(&world)?;
    let owner = Principal::local_owner();
    let job = || Job::new(ProjectId::new(7), ScanMode::Incremental);
    let _ = scanner.run(&owner, job().without_level2());
    assert_eq!(runs(&runner), 0, "watch mode scans after the first");
    scanner.rescan(&owner, ProjectId::new(7))?;
    assert_eq!(
        runs(&runner),
        0,
        "a rescan of MCP or the UI is level 1 only"
    );
    let _ = scanner.run(&owner, job());
    assert_eq!(runs(&runner), 0, "a job asks for level 2 explicitly");
    let _ = scanner.run(&owner, job().with_level2());
    assert_eq!(
        runs(&runner),
        1,
        "the scan of the command line runs level 2"
    );
    Ok(())
}

/// The world of `a_unchanged_b_changed` with these naming rules (now, kept by the last snapshot).
fn world_with_rules(
    now: &'static str,
    kept: Option<&'static str>,
) -> Result<Arc<World>, CodedError> {
    Ok(Arc::new(World {
        found: Some(vec![("a.rs", false), ("b.rs", true)]),
        previous: previous_snapshot()?,
        rules: (now, kept),
        ..World::default()
    }))
}

#[test]
fn an_incremental_scan_after_other_naming_rules_is_a_full_scan() -> Result<(), CodedError> {
    for kept in [None, Some("csharp=file")] {
        let world = world_with_rules("csharp=namespace", kept)?;
        let (result, _) = run(&world, ScanMode::Incremental);
        let JobResult::Finished(job) = result else {
            panic!("scan did not finish");
        };
        let persisted = world.last_persisted()?;
        let paths: Vec<_> = persisted.files.iter().map(|f| f.path.as_str()).collect();
        assert_eq!(paths, ["a.rs", "b.rs"], "every file is analysed again");
        assert_eq!(job.snapshot().mode, ScanMode::Full);
        assert_eq!(persisted.naming_rules, "csharp=namespace");
        let timed: Vec<_> = persisted
            .parse_micros
            .iter()
            .map(|(language, _)| language.as_str())
            .collect();
        assert_eq!(timed, ["rust"], "a full scan times each language");
        assert_eq!(persisted.removed, Vec::<RelativePath>::new());
    }
    Ok(())
}

#[test]
fn an_incremental_scan_with_the_same_naming_rules_stays_incremental() -> Result<(), CodedError> {
    let world = world_with_rules("csharp=namespace", Some("csharp=namespace"))?;
    let (result, _) = run(&world, ScanMode::Incremental);
    let JobResult::Finished(job) = result else {
        panic!("scan did not finish");
    };
    let paths: Vec<_> = world
        .last_persisted()?
        .files
        .iter()
        .map(|f| f.path.as_str().to_owned())
        .collect();
    assert_eq!(paths, ["b.rs"]);
    assert_eq!(job.snapshot().mode, ScanMode::Incremental);
    assert_eq!(world.last_persisted()?.parse_micros, Vec::new());
    Ok(())
}

#[test]
fn the_worker_limit_caps_the_threads_and_zero_means_all_cores() {
    use codeskimr_application::pipeline::analyze_workers;
    let cores = analyze_workers(0);
    assert!(cores >= 1);
    assert_eq!(analyze_workers(1), 1);
    assert_eq!(analyze_workers(u32::MAX), cores);
}

/// What an incremental scan restricted to `paths` writes over a world where `found` is what
/// discovery sees; `None` if it found nothing changed where it looked and wrote nothing.
fn restricted_scan_if_written(
    found: Vec<(&'static str, bool)>,
    paths: &[&str],
) -> Result<Option<SnapshotData>, CodedError> {
    let world = Arc::new(World {
        found: Some(found),
        previous: previous_snapshot()?,
        ..World::default()
    });
    let paths: Vec<String> = paths.iter().map(|path| (*path).to_owned()).collect();
    let job = Job::new(ProjectId::new(7), ScanMode::Incremental).with_paths(&paths);
    let result = scanner(&world, &EventBus::new()).run(&Principal::local_owner(), job);
    assert!(
        matches!(result, JobResult::Finished(_)),
        "scan did not finish"
    );
    Ok(world.last_persisted().ok())
}

/// [`restricted_scan_if_written`] for a scan that must write a snapshot.
fn restricted_scan(
    found: Vec<(&'static str, bool)>,
    paths: &[&str],
) -> Result<SnapshotData, CodedError> {
    restricted_scan_if_written(found, paths)?.ok_or_else(|| CodedError::new(ErrorCode::DiskFull))
}

fn analysed(data: &SnapshotData) -> Vec<&str> {
    data.files.iter().map(|file| file.path.as_str()).collect()
}

#[test]
fn a_scan_restricted_to_paths_does_not_look_at_a_change_elsewhere() -> Result<(), CodedError> {
    let found = || vec![("a.rs", false), ("b.rs", true), ("c.rs", false)];
    assert_eq!(
        restricted_scan_if_written(found(), &["a.rs"])?,
        None,
        "the change in b.rs is not looked at, so nothing is written"
    );
    assert_eq!(analysed(&restricted_scan(found(), &["b.rs"])?), ["b.rs"]);
    assert_eq!(analysed(&restricted_scan(found(), &[])?), ["b.rs"]);
    Ok(())
}

#[test]
fn a_scan_restricted_to_paths_keeps_a_file_that_is_gone_elsewhere() -> Result<(), CodedError> {
    let found = || vec![("a.rs", false), ("c.rs", false)];
    let kept = restricted_scan_if_written(found(), &["a.rs"])?;
    assert_eq!(kept, None, "b.rs stays, so nothing is written");
    let all = restricted_scan(found(), &[])?;
    assert_eq!(all.removed, [model(RelativePath::new("b.rs"))?]);
    let named = restricted_scan(found(), &["b.rs"])?;
    assert_eq!(named.removed, [model(RelativePath::new("b.rs"))?]);
    Ok(())
}

#[test]
fn a_scan_restricted_to_paths_that_name_no_file_changes_nothing() -> Result<(), CodedError> {
    let found = vec![("a.rs", false), ("b.rs", true), ("c.rs", false)];
    let in_folder = restricted_scan_if_written(found, &["./src/"])?;
    assert_eq!(in_folder, None, "no file lies below src");
    let found = vec![("a.rs", false), ("b.rs", true), ("c.rs", false)];
    let escaping = restricted_scan_if_written(found, &["../b.rs", "/b.rs"])?;
    assert_eq!(
        escaping, None,
        "a path that leaves the project names nothing"
    );
    Ok(())
}

/// What a scan over the world of `a.rs`, `b.rs` and `c.rs` made of the `paths` it was asked for.
fn paths_report(mode: ScanMode, paths: &[&str]) -> Result<PathsReport, CodedError> {
    let world = Arc::new(World {
        found: Some(vec![("a.rs", false), ("c.rs", false)]),
        previous: previous_snapshot()?,
        ..World::default()
    });
    let paths: Vec<String> = paths.iter().map(|path| (*path).to_owned()).collect();
    let job = Job::new(ProjectId::new(7), mode).with_paths(&paths);
    let JobResult::Finished(job) =
        scanner(&world, &EventBus::new()).run(&Principal::local_owner(), job)
    else {
        return Err(CodedError::new(ErrorCode::UnexpectedInternalError));
    };
    Ok(job.paths_report().clone())
}

#[test]
fn a_path_that_names_no_file_is_reported_and_a_deleted_file_is_not_unmatched()
-> Result<(), CodedError> {
    // `b.rs` is gone from the folder but known to the previous snapshot: a legitimate path.
    let report = paths_report(
        ScanMode::Incremental,
        &["a.rs", "b.rs", "typo.rs", "../a.rs", "/a.rs"],
    )?;
    assert_eq!(report.unmatched, ["typo.rs", "../a.rs", "/a.rs"]);
    assert!(!report.ignored);
    Ok(())
}

#[test]
fn a_scan_without_paths_has_nothing_to_report() -> Result<(), CodedError> {
    assert_eq!(
        paths_report(ScanMode::Incremental, &[])?,
        PathsReport::default()
    );
    Ok(())
}

#[test]
fn paths_of_a_scan_that_had_to_be_a_full_one_are_reported_as_ignored() -> Result<(), CodedError> {
    assert!(paths_report(ScanMode::Full, &["a.rs"])?.ignored);
    let world = world_with_rules("csharp=namespace", Some("csharp=file"))?;
    let job = Job::new(ProjectId::new(7), ScanMode::Incremental).with_paths(&["a.rs".to_owned()]);
    let JobResult::Finished(job) =
        scanner(&world, &EventBus::new()).run(&Principal::local_owner(), job)
    else {
        panic!("scan did not finish");
    };
    assert_eq!(job.snapshot().mode, ScanMode::Full);
    assert!(
        job.paths_report().ignored,
        "other naming rules make a full scan"
    );
    Ok(())
}

/// The scanner with the alert step over the fakes; the critical symbol `Demo.Box.Work` is in the
/// snapshot the states hold.
fn alerting(
    world: &Arc<World>,
    bus: &EventBus,
) -> (Scanner, Arc<MemoryAnnotations>, Arc<MemoryStates>) {
    let annotations = Arc::new(MemoryAnnotations::default());
    let states = Arc::new(MemoryStates::default());
    annotations.critical(SymbolKind::Method, "Demo.Box.Work", &[]);
    states.set(vec![symbol(SymbolKind::Method, "Demo.Box.Work", "one")]);
    let scanner = scanner(world, bus).with_alerts(step(&annotations, &states, bus));
    (scanner, annotations, states)
}

fn scan_once(scanner: &Scanner, job: Job<codeskimr_application::pipeline::Queued>) -> JobResult {
    scanner.run(&Principal::local_owner(), job)
}

#[test]
fn the_alert_of_a_git_project_says_it_came_with_a_fetch_whatever_started_the_scan() {
    let world = Arc::new(World::default());
    let bus = EventBus::new();
    let (scanner, annotations, states) = alerting(&world, &bus);
    let job = || Job::new(ProjectId::new(7), ScanMode::Full).from_watcher();
    assert!(matches!(scan_once(&scanner, job()), JobResult::Finished(_)));
    states.set(vec![symbol(SymbolKind::Method, "Demo.Box.Work", "two")]);
    assert!(matches!(scan_once(&scanner, job()), JobResult::Finished(_)));
    let alerts = annotations.open_alerts();
    assert_eq!(alerts.len(), 1);
    assert_eq!(alerts[0].sighting.trigger, AlertTrigger::Fetch);
}

#[test]
fn the_alert_of_a_folder_project_names_what_started_the_scan() {
    let world = Arc::new(World {
        folder_project: true,
        ..World::default()
    });
    let bus = EventBus::new();
    let (scanner, annotations, states) = alerting(&world, &bus);
    scan_once(&scanner, Job::new(ProjectId::new(7), ScanMode::Full));
    states.set(vec![symbol(SymbolKind::Method, "Demo.Box.Work", "two")]);
    scan_once(&scanner, Job::new(ProjectId::new(7), ScanMode::Full));
    states.set(vec![symbol(SymbolKind::Method, "Demo.Box.Work", "three")]);
    scan_once(
        &scanner,
        Job::new(ProjectId::new(7), ScanMode::Incremental).from_watcher(),
    );
    let alerts = annotations.open_alerts();
    assert_eq!(alerts[0].counter, 2);
    assert_eq!(alerts[0].sighting.trigger, AlertTrigger::Watcher);
}

#[test]
fn a_failure_of_the_alert_step_leaves_the_snapshot_and_its_event_as_they_are() {
    let world = Arc::new(World::default());
    let bus = EventBus::new();
    let events = bus.subscribe();
    let (scanner, annotations, _) = alerting(&world, &bus);
    *annotations.fail.lock().unwrap() = Some(ErrorCode::DiskFull);
    let result = scan_once(&scanner, Job::new(ProjectId::new(7), ScanMode::Full));
    let JobResult::Finished(job) = result else {
        panic!("a failed alert step must not fail the scan");
    };
    assert_eq!(job.snapshot().stats.files, 2);
    assert_eq!(world.persisted_count(), 1);
    let events: Vec<Event> = events.try_iter().collect();
    assert!(matches!(events.last(), Some(Event::SnapshotCreated(_))));
    assert!(
        events
            .iter()
            .all(|event| !matches!(event, Event::ScanStopped(_)))
    );
}

#[test]
fn the_alert_step_runs_after_the_snapshot_is_stored() {
    let world = Arc::new(World::default());
    let bus = EventBus::new();
    let (scanner, annotations, _) = alerting(&world, &bus);
    scan_once(&scanner, Job::new(ProjectId::new(7), ScanMode::Full));
    assert_eq!(world.persisted_count(), 1);
    assert_eq!(annotations.calls_named("refresh_targets"), 1);
}

/// Keeps what the scanner stores, or refuses to.
#[derive(Default)]
struct MemoryReports {
    writes: Mutex<Vec<ReportWrite>>,
    refuse: bool,
}

impl ScanReportStore for MemoryReports {
    fn store_report(&self, write: &ReportWrite) -> Result<(), CodedError> {
        if self.refuse {
            return Err(CodedError::new(ErrorCode::DatabaseFailed));
        }
        self.writes
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(write.clone());
        Ok(())
    }

    fn report(&self, _: ProjectId) -> Result<Option<ScanReport>, CodedError> {
        Ok(None)
    }
}

fn run_with_reports(
    reports: &Arc<MemoryReports>,
    job: Job<codeskimr_application::pipeline::Queued>,
) -> JobResult {
    let world = Arc::new(World::default());
    scanner(&world, &EventBus::new())
        .with_reports(ReportStep::new(reports.clone(), world.clone()))
        .run(&Principal::local_owner(), job)
}

#[test]
fn a_scan_stores_its_channel_and_level_and_a_scan_without_level_2_leaves_the_second_report_alone() {
    let reports = Arc::new(MemoryReports::default());
    let job = Job::new(ProjectId::new(7), ScanMode::Full).started_from(ScanOrigin::Mcp);
    assert!(matches!(
        run_with_reports(&reports, job),
        JobResult::Finished(_)
    ));
    let writes = reports.writes.lock().unwrap();
    assert_eq!(writes.len(), 1);
    assert_eq!(writes[0].last.origin, ScanOrigin::Mcp);
    assert_eq!(writes[0].last.level, ScanLevel::One);
    assert_eq!(writes[0].level2, None);
}

#[test]
fn a_report_that_cannot_be_stored_is_a_warning_and_the_snapshot_stays() {
    let reports = Arc::new(MemoryReports {
        refuse: true,
        ..MemoryReports::default()
    });
    let job = Job::new(ProjectId::new(7), ScanMode::Full);
    let JobResult::Finished(job) = run_with_reports(&reports, job) else {
        panic!("the scan must finish");
    };
    let codes: Vec<_> = job.warnings().iter().map(|warning| warning.code).collect();
    assert_eq!(codes, [ErrorCode::ScanReportNotStored]);
    assert_eq!(job.snapshot().stats.files, 2);
}

#[test]
fn without_a_wired_second_pass_no_level_2_report_is_stored() {
    let reports = Arc::new(MemoryReports::default());
    let world = Arc::new(World::default());
    let scanner = scanner(&world, &EventBus::new())
        .with_reports(ReportStep::new(reports.clone(), world.clone()));
    let job = Job::new(ProjectId::new(7), ScanMode::Full).with_level2();
    // No pass is wired, so nothing runs and nothing is stored for level 2.
    assert!(matches!(
        scanner.run(&Principal::local_owner(), job),
        JobResult::Finished(_)
    ));
    assert_eq!(reports.writes.lock().unwrap()[0].level2, None);
}

/// Level 1 of the browser: stored, no indexer started yet.
fn level_one_of(
    world: &Arc<World>,
    scanner: &Scanner,
) -> Result<
    codeskimr_application::pipeline::Job<codeskimr_application::pipeline::Finished>,
    CodedError,
> {
    let job = Job::new(ProjectId::new(7), ScanMode::Full).started_from(ScanOrigin::Browser);
    match scanner.run(&Principal::local_owner(), job) {
        JobResult::Finished(job) => {
            assert_eq!(world.persisted_count(), 1);
            Ok(job)
        }
        _ => Err(CodedError::new(ErrorCode::UnexpectedInternalError)),
    }
}

fn run_level_two(
    scanner: &Scanner,
    finished: &codeskimr_application::pipeline::Job<codeskimr_application::pipeline::Finished>,
    (principal, cancel): (&Principal, &CancelToken),
) -> Vec<codeskimr_application::level2::LanguageReport> {
    scanner.run_level2(&Level2Run {
        principal,
        snapshot: finished.snapshot(),
        root: finished.root(),
        cancel,
        origin: ScanOrigin::Browser,
    })
}

#[test]
fn level_two_after_level_one_starts_the_indexer_once_and_level_one_is_already_stored()
-> Result<(), CodedError> {
    let world = Arc::new(World {
        previous: previous_snapshot()?,
        ..World::default()
    });
    let (scanner, runner) = level2_scanner(&world)?;
    let finished = level_one_of(&world, &scanner)?;
    assert_eq!(runs(&runner), 0, "the scan itself starts no indexer");
    let reports = run_level_two(
        &scanner,
        &finished,
        (&Principal::local_owner(), &CancelToken::new()),
    );
    assert_eq!(runs(&runner), 1);
    assert_eq!(reports.len(), 1);
    Ok(())
}

#[test]
fn a_cancelled_token_stops_level_two_before_the_indexer_and_keeps_level_one()
-> Result<(), CodedError> {
    let world = Arc::new(World {
        previous: previous_snapshot()?,
        ..World::default()
    });
    let (scanner, runner) = level2_scanner(&world)?;
    let finished = level_one_of(&world, &scanner)?;
    let cancel = CancelToken::new();
    cancel.cancel();
    let reports = run_level_two(&scanner, &finished, (&Principal::local_owner(), &cancel));
    assert_eq!(runs(&runner), 0);
    assert!(matches!(
        reports[0].outcome,
        codeskimr_application::level2::Level2Outcome::Skipped(
            codeskimr_application::level2::Skipped::Cancelled
        )
    ));
    assert_eq!(world.persisted_count(), 1, "level 1 is as it was");
    Ok(())
}

#[test]
fn a_caller_without_the_right_to_run_indexers_gets_no_run_even_after_level_one()
-> Result<(), CodedError> {
    let world = Arc::new(World {
        previous: previous_snapshot()?,
        ..World::default()
    });
    let (scanner, runner) = level2_scanner(&world)?;
    let finished = level_one_of(&world, &scanner)?;
    let remote = Principal::limited(
        PermissionSet::none()
            .with(Permission::Read)
            .with(Permission::Scan),
    );
    let gate = scanner.level2_gate(&remote, ProjectId::new(7), finished.root());
    assert_eq!(
        gate,
        Err(codeskimr_application::level2::Skipped::NotPermitted)
    );
    let reports = run_level_two(&scanner, &finished, (&remote, &CancelToken::new()));
    assert_eq!(runs(&runner), 0);
    assert!(matches!(
        reports[0].outcome,
        codeskimr_application::level2::Level2Outcome::Skipped(
            codeskimr_application::level2::Skipped::NotPermitted
        )
    ));
    Ok(())
}

#[test]
fn a_consent_that_is_gone_when_the_run_starts_stops_it() -> Result<(), CodedError> {
    let world = Arc::new(World {
        previous: previous_snapshot()?,
        ..World::default()
    });
    let consent = Arc::new(InMemoryConsent::default());
    let (scanner, runner) = level2_scanner_over(&world, &consent)?;
    let finished = level_one_of(&world, &scanner)?;
    let owner = Principal::local_owner();
    assert!(
        scanner
            .level2_gate(&owner, ProjectId::new(7), finished.root())
            .is_ok()
    );
    // The owner revokes between the answer of the command and the start of the run.
    codeskimr_application::level2::revoke(&owner, consent.as_ref(), ProjectId::new(7))?;
    let reports = run_level_two(&scanner, &finished, (&owner, &CancelToken::new()));
    assert_eq!(runs(&runner), 0);
    assert!(matches!(
        reports[0].outcome,
        codeskimr_application::level2::Level2Outcome::Skipped(
            codeskimr_application::level2::Skipped::NotConfirmed
        )
    ));
    Ok(())
}

#[test]
fn a_level_two_run_tells_the_language_and_the_end_and_nothing_else() -> Result<(), CodedError> {
    let world = Arc::new(World {
        previous: previous_snapshot()?,
        ..World::default()
    });
    let bus = EventBus::new();
    let (scanner, _runner) =
        level2_scanner_on(&world, &Arc::new(InMemoryConsent::default()), &bus)?;
    let finished = level_one_of(&world, &scanner)?;
    let events = bus.subscribe();
    let _ = run_level_two(
        &scanner,
        &finished,
        (&Principal::local_owner(), &CancelToken::new()),
    );
    let seen: Vec<Event> = events.try_iter().collect();
    assert!(matches!(
        seen.first(),
        Some(Event::Level2Progressed(progress)) if progress.language.as_str() == "rust"
    ));
    assert!(matches!(
        seen.last(),
        Some(Event::Level2Finished(done)) if done.project == ProjectId::new(7)
    ));
    Ok(())
}

fn memory_run(used_mb: u64, max_mb: u64) -> (JobResult, Arc<World>) {
    let bus = EventBus::new();
    let world = Arc::new(World {
        resolve_wait_steps: 500,
        ..World::default()
    });
    let limit = MemoryLimit::new(
        Arc::new(GrowingGauge::new(
            used_mb.saturating_sub(max_mb + 1),
            used_mb,
        )),
        max_mb,
        std::time::Duration::from_millis(1),
    );
    let job = Job::new(ProjectId::new(7), ScanMode::Full);
    let result = scanner_with(&world, &bus, Some(limit)).run(&Principal::local_owner(), job);
    (result, world)
}

#[test]
fn a_scan_that_holds_more_memory_than_allowed_fails_with_its_own_code_and_writes_nothing() {
    let (result, world) = memory_run(900, 800);
    assert!(matches!(result, JobResult::Failed(job) if job.code() == ErrorCode::ScanMemoryLimit));
    assert_eq!(world.persisted_count(), 0);
}

#[test]
fn a_scan_within_the_memory_limit_is_not_stopped() {
    let bus = EventBus::new();
    let world = Arc::new(World::default());
    let limit = MemoryLimit::new(
        Arc::new(FixedGauge(100)),
        800,
        std::time::Duration::from_millis(1),
    );
    let job = Job::new(ProjectId::new(7), ScanMode::Full);
    let result = scanner_with(&world, &bus, Some(limit)).run(&Principal::local_owner(), job);
    assert!(matches!(result, JobResult::Finished(_)));
}
