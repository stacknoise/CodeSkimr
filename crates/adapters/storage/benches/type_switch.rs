//! Benchmark of what `explorer.symbol` asks of the store (blueprint 16.1: switching the selected
//! type in under 16 ms). The snapshot has the symbols of a 100k-line project, grouped in types
//! with a member table each. One iteration reads one type as the command does: the symbol card and
//! the rows of its member table.

use std::hint::black_box;
use std::process::ExitCode;

use codeskimr_application::ports::{CodeIndex, CodeOutline};
use codeskimr_domain::{SnapshotId, Symbol, SymbolId, SymbolKind};
use codeskimr_storage::SqliteStore;
use criterion::Criterion;

mod common;

use common::{SNAPSHOT, SYMBOLS, symbol};

/// Symbols per type: the type and its members.
const GROUP: u64 = 20;
/// Types the benchmark switches between in turn.
const TYPES_PER_ITERATION: u64 = 10;

fn is_type(id: u64) -> bool {
    id % GROUP == 1
}

fn member_of_group(id: u64) -> Option<Symbol> {
    let mut found = symbol(id, &format!("item{id}"))?;
    if is_type(id) {
        found.kind = SymbolKind::Class;
    } else {
        found.parent = Some(SymbolId::new(id - (id - 1) % GROUP));
    }
    Some(found)
}

/// Reads one type: its card and its member rows.
fn switch_to(store: &SqliteStore, id: u64) -> Option<()> {
    let snapshot = SnapshotId::new(SNAPSHOT);
    let id = SymbolId::new(id);
    black_box(store.symbol_card(snapshot, id).ok()??);
    black_box(store.member_rows(snapshot, id).ok()?);
    Some(())
}

fn switches(c: &mut Criterion) -> Option<()> {
    let store = common::store(
        (1..=SYMBOLS).filter_map(member_of_group).collect(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    )?;
    let types: Vec<u64> = (1..=SYMBOLS).filter(|id| is_type(*id)).collect();
    let step = types.len() / usize::try_from(TYPES_PER_ITERATION).ok()?;
    switch_to(&store, *types.first()?)?;
    let mut group = c.benchmark_group("type_switch");
    group.bench_function("explorer_symbol", |b| {
        let mut at = 0;
        b.iter(|| {
            at = (at + step) % types.len();
            types.get(at).and_then(|id| switch_to(&store, *id))
        });
    });
    group.finish();
    Some(())
}

fn main() -> ExitCode {
    common::run(switches)
}
