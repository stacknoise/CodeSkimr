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
    AnnotationCreation, AnnotationStore, Clock, ContentDigest, SymbolHash,
};
use codeskimr_domain::{
    AlertTrigger, AnnotationContent, AnnotationKind, AnnotationTarget, Author, ContentHash,
    NewAnnotation, ProjectId, QualifiedName, Snapshot, SnapshotId, SnapshotStats, SymbolId,
    SymbolKind, Timestamp,
};
use codeskimr_storage::SqliteStore;
use criterion::Criterion;

mod common;

use common::{SNAPSHOT, SYMBOLS, symbol};

/// Critical annotations of the benchmark.
const CRITICAL: u64 = 100;
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

fn snapshot() -> Snapshot {
    Snapshot {
        id: SnapshotId::new(SNAPSHOT),
        project: ProjectId::new(PROJECT),
        commit: None,
        created: Timestamp(1),
        mode: codeskimr_domain::ScanMode::Full,
        stats: SnapshotStats::default(),
    }
}

fn step() -> Option<AlertStep> {
    let symbols: Vec<_> = (1..=SYMBOLS)
        .filter_map(|id| symbol(id, &name(id)))
        .collect();
    let hashes = (1..=SYMBOLS)
        .filter_map(|id| {
            Some(SymbolHash {
                symbol: SymbolId::new(id),
                hash: ContentHash::new(format!("hash{id}")).ok()?,
            })
        })
        .collect();
    let store = Arc::new(common::store(symbols, Vec::new(), hashes)?);
    (1..=CRITICAL).try_for_each(|id| annotate(&store, id))?;
    Some(AlertStep {
        annotations: store.clone(),
        states: store,
        digest: Arc::new(Plain),
        clock: Arc::new(Plain),
        events: EventBus::new(),
    })
}

fn steps(c: &mut Criterion) -> Option<()> {
    let step = step()?;
    let snapshot = snapshot();
    // The first run takes the baselines; the measured ones find nothing changed.
    step.run(&snapshot, AlertTrigger::Watcher).ok()?;
    c.benchmark_group("alerts")
        .bench_function("step_100_critical", |b| {
            b.iter(|| {
                black_box(step.run(&snapshot, AlertTrigger::Watcher).ok());
            });
        });
    Some(())
}

fn main() -> ExitCode {
    let mut criterion = Criterion::default().configure_from_args();
    if steps(&mut criterion).is_none() {
        return ExitCode::FAILURE;
    }
    criterion.final_summary();
    ExitCode::SUCCESS
}
