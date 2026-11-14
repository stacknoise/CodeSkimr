//! Benchmark of the quick search (blueprint 16.1: under 10 ms per keystroke). It runs the query
//! behind `search.query` on a snapshot with the symbols of a 100k-line project. The queries are
//! what a user types one character after the other, from one letter to a whole name.

use std::hint::black_box;
use std::process::ExitCode;

use codeskimr_domain::SnapshotId;
use codeskimr_storage::SqliteStore;
use criterion::Criterion;

mod common;

use common::{SNAPSHOT, SYMBOLS, symbol};

/// Number of hits the quick search shows.
const LIMIT: u32 = 20;
/// Number of name parts.
const PART_COUNT: usize = 8;
/// Name parts the symbol names are made of, so that short queries match many symbols.
const PARTS: [&str; PART_COUNT] = [
    "parse", "scan", "build", "index", "render", "load", "find", "check",
];
/// The longest query the benchmark types: a part of a name that exists many times.
const TYPED: &str = "scanindex";

fn name(id: u64) -> String {
    let count = PART_COUNT as u64;
    let part = |digit: u64| {
        let at =
            usize::try_from((id / count.pow(u32::try_from(digit).unwrap_or_default())) % count)
                .unwrap_or_default();
        PARTS.get(at).copied().unwrap_or_default()
    };
    format!("{}{}_{id}", part(0), part(1))
}

fn store() -> Option<SqliteStore> {
    common::store(
        (1..=SYMBOLS)
            .filter_map(|id| symbol(id, &name(id)))
            .collect(),
        Vec::new(),
    )
}

/// One keystroke: the search for the first `typed` characters of the query.
fn keystroke(store: &SqliteStore, typed: usize) -> Option<()> {
    let query: String = TYPED.chars().take(typed).collect();
    black_box(
        store
            .search(SnapshotId::new(SNAPSHOT), &query, LIMIT)
            .ok()?,
    );
    Some(())
}

fn searches(c: &mut Criterion) -> Option<()> {
    let store = store()?;
    keystroke(&store, 1)?;
    let mut group = c.benchmark_group("search");
    for (name, typed) in [("typed_1", 1), ("typed_3", 3), ("typed_9", TYPED.len())] {
        group.bench_function(name, |b| b.iter(|| keystroke(&store, typed)));
    }
    group.finish();
    Some(())
}

fn main() -> ExitCode {
    let mut criterion = Criterion::default().configure_from_args();
    if searches(&mut criterion).is_none() {
        return ExitCode::FAILURE;
    }
    criterion.final_summary();
    ExitCode::SUCCESS
}
