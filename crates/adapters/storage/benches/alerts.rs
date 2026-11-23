//! Benchmark of the alert step (ADR 0068) on a snapshot with the symbols of a 100k-line project and
//! a hundred critical annotations whose symbols did not change. This is what every save of a watched
//! project pays after the snapshot is written; it has to fit well into the budget of a rescan.

use std::hint::black_box;
use std::process::ExitCode;
use std::sync::Arc;

use codeskimr_application::alerts::AlertStep;
use codeskimr_application::error::CodedError;
use codeskimr_application::events::EventBus;
use codeskimr_application::ports::{
    AnnotationCreation, AnnotationStore, Clock, ContentDigest, PendingReference, SnapshotStore,
    SymbolHash,
};
use codeskimr_domain::{
    AlertTrigger, AnnotationContent, AnnotationKind, AnnotationTarget, Author, ContentHash, FileId,
    NewAnnotation, ProjectId, QualifiedName, ReferenceKind, Snapshot, SnapshotId, SnapshotStats,
    Symbol, SymbolId, SymbolKind, Timestamp,
};
use codeskimr_storage::SqliteStore;
use criterion::Criterion;

mod common;

use common::{Content, FILES, SNAPSHOT, SYMBOLS, span, symbol};

/// Critical annotations of the benchmark.
const CRITICAL: u64 = 100;
/// Annotations on symbols that the newest snapshot no longer has.
const VANISHED: u64 = 50;
/// Names written in the files of the newest snapshot that point at nothing.
const PENDING: u64 = 50_000;
const PROJECT: u64 = 1;
const AUTHOR: &str = "bench";

/// A digest and a clock without cost of their own: the real digest hashes its parts, the cost
/// measured here is the store's.
struct Plain;

impl ContentDigest for Plain {
    fn digest(&self, parts: &[&str]) -> Result<ContentHash, CodedError> {
        ContentHash::new(format!("d{}", parts.concat())).map_err(CodedError::caused_by)
    }
}

impl Clock for Plain {
    fn now(&self) -> Timestamp {
        Timestamp(1)
    }
}

fn name(id: u64) -> String {
    format!("function{id}")
}

fn annotate(store: &SqliteStore, id: u64) -> Option<()> {
    let target = symbol(id, &name(id))?;
    store
        .create_annotation(&AnnotationCreation {
            annotation: NewAnnotation {
                project: ProjectId::new(PROJECT),
                target: AnnotationTarget {
                    kind: SymbolKind::Function,
                    name: QualifiedName::new(target.qualified_name.as_str()).ok()?,
                    parameters: Vec::new(),
                },
                author: Author::new(AUTHOR).ok()?,
                content: AnnotationContent {
                    kind: AnnotationKind::Critical,
                    title: None,
                    text: None,
                    status: None,
                    due: None,
                    assignee: None,
                },
            },
            now: Timestamp(1),
        })
        .ok()?;
    Some(())
}

/// Writes the second snapshot: the same files under new ids with `symbols` and `hashes` (their ids
/// and files are moved to the new ones) and `pending` names that point at nothing.
fn next_snapshot(
    store: &SqliteStore,
    symbols: Vec<Symbol>,
    hashes: Vec<SymbolHash>,
    pending: u64,
) -> Option<()> {
    let moved = |symbol: SymbolId| SymbolId::new(symbol.get() + SYMBOLS);
    let first = FILES + 1;
    let names = (1..=pending).filter_map(|n| {
        Some(PendingReference {
            from: SymbolId::new(SYMBOLS + 1),
            file: FileId::new(first),
            span: span(1)?,
            kind: ReferenceKind::Call,
            target: QualifiedName::new(format!("name{n}")).ok()?,
            receiver: None,
        })
    });
    let content = Content {
        symbols: symbols
            .into_iter()
            .map(|symbol| Symbol {
                id: moved(symbol.id),
                file: FileId::new(symbol.file.get() + FILES),
                ..symbol
            })
            .collect(),
        references: Vec::new(),
        hashes: hashes
            .into_iter()
            .map(|hash| SymbolHash {
                symbol: moved(hash.symbol),
                ..hash
            })
            .collect(),
        pending: names.collect(),
    };
    store
        .persist(common::data(SNAPSHOT + 1, first, content))
        .ok()?;
    Some(())
}

fn snapshot() -> Snapshot {
    Snapshot {
        id: SnapshotId::new(SNAPSHOT + 1),
        project: ProjectId::new(PROJECT),
        commit: None,
        created: Timestamp(1),
        mode: codeskimr_domain::ScanMode::Full,
        stats: SnapshotStats::default(),
    }
}

/// The step over a store with `CRITICAL` annotations on symbols that stay and `vanished` on symbols
/// that the newest snapshot no longer has, and `PENDING` names written in its files.
fn step(vanished: u64) -> Option<AlertStep> {
    let symbols: Vec<_> = (1..=SYMBOLS)
        .filter_map(|id| symbol(id, &name(id)))
        .collect();
    let hashes: Vec<SymbolHash> = (1..=SYMBOLS)
        .filter_map(|id| {
            Some(SymbolHash {
                symbol: SymbolId::new(id),
                hash: ContentHash::new(format!("hash{id}")).ok()?,
            })
        })
        .collect();
    let store = common::store(symbols.clone(), Vec::new(), hashes.clone(), Vec::new())?;
    (1..=CRITICAL + vanished).try_for_each(|id| annotate(&store, id))?;
    let gone = |id: u64| id > CRITICAL && id <= CRITICAL + vanished;
    next_snapshot(
        &store,
        symbols
            .into_iter()
            .filter(|symbol| !gone(symbol.id.get()))
            .collect(),
        hashes
            .into_iter()
            .filter(|hash| !gone(hash.symbol.get()))
            .collect(),
        PENDING,
    )?;
    let store = Arc::new(store);
    Some(AlertStep {
        annotations: store.clone(),
        states: store,
        digest: Arc::new(Plain),
        clock: Arc::new(Plain),
        events: EventBus::new(),
    })
}

fn measure(c: &mut Criterion, name: &str, vanished: u64) -> Option<()> {
    let step = step(vanished)?;
    let snapshot = snapshot();
    // The first run takes the baselines and raises the alerts; the measured ones find nothing new.
    step.run(&snapshot, AlertTrigger::Watcher).ok()?;
    c.benchmark_group("alerts").bench_function(name, |b| {
        b.iter(|| {
            black_box(step.run(&snapshot, AlertTrigger::Watcher).ok());
        });
    });
    Some(())
}

fn main() -> ExitCode {
    let mut criterion = Criterion::default().configure_from_args();
    if measure(&mut criterion, "step_100_critical", 0).is_none()
        || measure(&mut criterion, "step_50_vanished", VANISHED).is_none()
    {
        return ExitCode::FAILURE;
    }
    criterion.final_summary();
    ExitCode::SUCCESS
}
