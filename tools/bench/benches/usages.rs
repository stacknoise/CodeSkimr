//! Benchmark of the usages query (blueprint 16.1: depth 2 in under 20 ms). The graph has the
//! size of a 100k-line project and a few symbols that nearly everything uses, like real code.

use std::hint::black_box;

use codeskimr_bench::random::Lcg;
use codeskimr_domain::{
    Confidence, Direction, FileId, Position, Reference, ReferenceId, ReferenceKind, ReverseIndex,
    Span, SymbolId, UsageQuery,
};
use criterion::Criterion;

const SYMBOLS: usize = 20_000;
const REFERENCES: usize = 100_000;
const FILES: usize = 400;
const LINES: usize = 500;
/// One reference in this many points at symbol 0, a symbol that nearly everything uses.
const HUB_SHARE: usize = 20;

fn reference(id: usize, random: &mut Lcg) -> Option<Reference> {
    let from = random.below(SYMBOLS);
    // The target is drawn below a random bound, so low ids are used much more often.
    let bound = random.below(SYMBOLS) + 1;
    let target = if id.is_multiple_of(HUB_SHARE) {
        0
    } else {
        random.below(bound)
    };
    let position = Position {
        line: u32::try_from(random.below(LINES)).ok()?,
        column: 0,
    };
    Some(Reference {
        id: ReferenceId::new(u64::try_from(id).ok()?),
        from: SymbolId::new(u64::try_from(from).ok()?),
        to: SymbolId::new(u64::try_from(target).ok()?),
        file: FileId::new(u64::try_from(random.below(FILES)).ok()?),
        span: Span::new(position, position).ok()?,
        kind: ReferenceKind::Call,
        confidence: Confidence::Certain,
    })
}

fn project_index() -> ReverseIndex {
    let mut random = Lcg::new(42);
    ReverseIndex::new((0..REFERENCES).filter_map(|id| reference(id, &mut random)))
}

fn queries(c: &mut Criterion) {
    let index = project_index();
    let mut group = c.benchmark_group("usages");
    for (name, depth) in [("depth_1", 1), ("depth_2", 2)] {
        let query = UsageQuery {
            symbol: SymbolId::new(0),
            direction: Direction::Incoming,
            depth,
            include_tests: true,
        };
        group.bench_function(name, |b| b.iter(|| black_box(index.usages(&query))));
    }
    group.finish();
}

fn main() {
    let mut criterion = Criterion::default().configure_from_args();
    queries(&mut criterion);
    criterion.final_summary();
}
