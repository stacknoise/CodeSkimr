//! Benchmark of what `usages.query` asks of the store (blueprint 16.1: depth 2 in under 20 ms).
//! The graph has the size of a 100k-line project and one symbol that a share of all references
//! point at, so the walk returns thousands of entries that are cut to the limit of the screen.
//! Unlike the benchmark of the reverse index alone, this one reads the names and files of the
//! entries from the stored snapshot, as the command does.

use std::hint::black_box;
use std::process::ExitCode;

use codeskimr_application::ports::UsageIndex;
use codeskimr_domain::{
    Confidence, Direction, FileId, Reference, ReferenceId, ReferenceKind, SnapshotId, SymbolId,
    UsageQuery,
};
use codeskimr_storage::SqliteStore;
use criterion::Criterion;

mod common;

use common::{FILES, SNAPSHOT, SYMBOLS, span};

const REFERENCES: u64 = 100_000;
/// One reference in this many points at the hub, the symbol that nearly everything uses.
const HUB_SHARE: u64 = 20;
const HUB: u64 = 1;
/// Number of entries the screen asks for.
const LIMIT: u32 = 500;
/// Multipliers of the deterministic spread of the references.
const SPREAD_FROM: u64 = 7_919;
const SPREAD_TO: u64 = 104_729;
const LINES: u32 = 500;
const MAX_DEPTH: u32 = 2;

fn reference(id: u64) -> Option<Reference> {
    let to = if id.is_multiple_of(HUB_SHARE) {
        HUB
    } else {
        // Low ids are used much more often than high ones.
        (id * SPREAD_TO) % ((id * SPREAD_FROM) % SYMBOLS + 1) + 1
    };
    Some(Reference {
        id: ReferenceId::new(id),
        from: SymbolId::new((id * SPREAD_FROM) % SYMBOLS + 1),
        to: SymbolId::new(to),
        file: FileId::new(id % FILES + 1),
        span: span(u32::try_from(id).ok()? % LINES)?,
        kind: ReferenceKind::Call,
        confidence: Confidence::Certain,
        verified: false,
    })
}

fn store() -> Option<SqliteStore> {
    common::store(
        (1..=SYMBOLS)
            .filter_map(|id| common::symbol(id, &format!("item{id}")))
            .collect(),
        (1..=REFERENCES).filter_map(reference).collect(),
    )
}

fn query(direction: Direction, depth: u32) -> UsageQuery {
    UsageQuery {
        symbol: SymbolId::new(HUB),
        direction,
        depth,
        include_tests: true,
    }
}

/// The reads of one `usages.query` call at depth 2 for the hub, in the order the command makes
/// them: the walk in both directions, the direct usages for the counts, and the impact. A failed
/// read ends the benchmark.
fn command_reads(store: &SqliteStore) -> Option<()> {
    let snapshot = SnapshotId::new(SNAPSHOT);
    for walk in [
        query(Direction::Incoming, MAX_DEPTH),
        query(Direction::Outgoing, MAX_DEPTH),
        query(Direction::Incoming, 1),
    ] {
        black_box(store.usages(snapshot, &walk, LIMIT).ok()?);
    }
    black_box(
        store
            .impact(snapshot, SymbolId::new(HUB), MAX_DEPTH, LIMIT)
            .ok()?,
    );
    Some(())
}

fn queries(c: &mut Criterion) -> Option<()> {
    let store = store()?;
    command_reads(&store)?;
    let mut group = c.benchmark_group("usages_page");
    group.bench_function("depth_2_hub", |b| b.iter(|| command_reads(&store)));
    group.finish();
    Some(())
}

fn main() -> ExitCode {
    let mut criterion = Criterion::default().configure_from_args();
    if queries(&mut criterion).is_none() {
        return ExitCode::FAILURE;
    }
    criterion.final_summary();
    ExitCode::SUCCESS
}
