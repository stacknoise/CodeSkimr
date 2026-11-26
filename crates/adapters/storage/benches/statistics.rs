//! Benchmark of what the statistics overview asks of the store (ADR 0091): the four cards, the
//! counts of the unused symbols and the sums, from the numbers the scan counted. The graph has the
//! size of a 100k-line project; `STATISTICS_BENCH_SCALE=30` gives the size of `microsoft/vscode`
//! (600,000 symbols, 3 million references). The time of the write of the scan, the time of the first
//! question (which counts the numbers, as the thread after a scan does) and the time of the
//! questions after it are printed or measured.

use std::hint::black_box;
use std::process::ExitCode;
use std::time::Instant;

use codeskimr_application::ports::{CodeStatistics, RankingRequest, StatisticsSetup};
use codeskimr_domain::{
    Confidence, FileId, KindGroup, Metric, MetricKind, MetricValue, Ranking, RankingQuery,
    Reference, ReferenceId, ReferenceKind, SnapshotId, SymbolId, SymbolKind, UsageOrder,
};
use codeskimr_storage::ProjectDb;
use criterion::Criterion;

mod common;

use common::{FILES, SNAPSHOT, SYMBOLS, span};

const REFERENCES: u64 = 100_000;
const SPREAD_FROM: u64 = 7_919;
const SPREAD_TO: u64 = 104_729;
const LINES: u32 = 500;
/// One symbol in this many is a class, the others are functions.
const CLASS_EVERY: u64 = 10;
/// Number of rows of a card.
const TOP: usize = 5;
const SCALE_VARIABLE: &str = "STATISTICS_BENCH_SCALE";

fn scale() -> u64 {
    std::env::var(SCALE_VARIABLE)
        .ok()
        .and_then(|text| text.parse().ok())
        .unwrap_or(1)
}

fn reference(id: u64, symbols: u64) -> Option<Reference> {
    Some(Reference {
        id: ReferenceId::new(id),
        from: SymbolId::new((id * SPREAD_FROM) % symbols + 1),
        to: SymbolId::new((id * SPREAD_TO) % ((id * SPREAD_FROM) % symbols + 1) + 1),
        file: FileId::new(id % FILES + 1),
        span: span(u32::try_from(id).ok()? % LINES)?,
        kind: ReferenceKind::Call,
        confidence: if id.is_multiple_of(3) {
            Confidence::Probable
        } else {
            Confidence::Certain
        },
        verified: false,
    })
}

fn metrics(symbols: u64) -> Vec<Metric> {
    let mut all = Vec::new();
    for id in 1..=symbols {
        let value = f64::from(u32::try_from(id % 40).unwrap_or_default());
        for (kind, scaled) in [
            (MetricKind::CyclomaticComplexity, value),
            (MetricKind::Loc, value * 3.0),
            (MetricKind::EfferentCoupling, value),
        ] {
            if let Ok(value) = MetricValue::new(scaled) {
                all.push(Metric {
                    symbol: SymbolId::new(id),
                    kind,
                    value,
                });
            }
        }
    }
    all
}

fn store() -> Option<ProjectDb> {
    let factor = scale();
    let symbols = SYMBOLS * factor;
    let mut all = Vec::new();
    for id in 1..=symbols {
        let mut made = common::symbol(id, &format!("item{id}"))?;
        if id % CLASS_EVERY == 0 {
            made.kind = SymbolKind::Class;
        }
        all.push(made);
    }
    common::store(
        all,
        (1..=REFERENCES * factor)
            .filter_map(|id| reference(id, symbols))
            .collect(),
        Vec::new(),
        metrics(symbols),
    )
}

fn query(ranking: Ranking, kinds: KindGroup) -> RankingQuery {
    RankingQuery {
        ranking,
        kinds,
        language: None,
        include_tests: false,
        order: UsageOrder::Certain,
    }
}

/// The settings of the questions: no test folders and no approximate language.
fn setup_of(names: &[String]) -> StatisticsSetup<'_> {
    StatisticsSetup {
        test_names: names,
        approximate: names,
    }
}

/// The reads of one `stats.overview` call, in the order of the command. A failed read ends the
/// benchmark.
fn overview_reads(store: &ProjectDb) -> Option<()> {
    let snapshot = SnapshotId::new(SNAPSHOT);
    let names: Vec<String> = Vec::new();
    let setup = setup_of(&names);
    let all = query(Ranking::Unused, KindGroup::All);
    black_box(store.unused_counts(snapshot, &all, setup).ok()?);
    black_box(store.summary(snapshot, false, setup).ok()?);
    for (ranking, kinds) in [
        (Ranking::MostUsed, KindGroup::Types),
        (Ranking::Unused, KindGroup::All),
        (Ranking::Complexity, KindGroup::Functions),
        (Ranking::Coupling, KindGroup::Types),
    ] {
        let wanted = query(ranking, kinds);
        black_box(
            store
                .ranked(&RankingRequest {
                    snapshot,
                    query: &wanted,
                    assessment: None,
                    limit: TOP,
                    setup,
                })
                .ok()?,
        );
    }
    Some(())
}

/// The read the exception list of the unused symbols judges from (ADR 0101), against the read of
/// every symbol it replaced. Both are printed once.
fn unused_facts_read(store: &ProjectDb) -> Option<()> {
    let snapshot = SnapshotId::new(SNAPSHOT);
    let names: Vec<String> = Vec::new();
    let setup = setup_of(&names);
    let started = Instant::now();
    let unused = store
        .unused_facts(snapshot, &query(Ranking::Unused, KindGroup::All), setup)
        .ok()?;
    eprintln!(
        "unused facts: {} symbols in {:?}",
        unused.len(),
        started.elapsed()
    );
    let started = Instant::now();
    let all = store.symbol_facts(snapshot).ok()?;
    eprintln!(
        "all facts: {} symbols in {:?}",
        all.len(),
        started.elapsed()
    );
    Some(())
}

fn queries(c: &mut Criterion) -> Option<()> {
    let store = store()?;
    let started = Instant::now();
    overview_reads(&store)?;
    eprintln!(
        "first overview, with counting the numbers: {:?}",
        started.elapsed()
    );
    unused_facts_read(&store)?;
    let mut group = c.benchmark_group("statistics");
    group.bench_function("overview", |b| b.iter(|| overview_reads(&store)));
    group.finish();
    Some(())
}

fn main() -> ExitCode {
    common::run(queries)
}
