//! Benchmarks of the domain model. They prove the harness; real budgets get their own
//! benchmarks as the pipeline grows (see `budgets.toml`).

use std::hint::black_box;

use codeskimr_domain::{
    FingerprintParams, Fingerprints, QualifiedName, TokenId, similar_pairs, winnow,
};
use criterion::Criterion;

const NAMES: [&str; 4] = [
    "Namespace.Type.member",
    "Namespace.Sub.Type",
    "Root",
    "A.B.C.D.E.F.G.H",
];

/// Repetitions per iteration, so that one measurement is long enough to be stable.
const REPEATS: usize = 1_000;

fn qualified_names(c: &mut Criterion) {
    let mut group = c.benchmark_group("domain");
    group.bench_function("qualified_name_new", |b| {
        b.iter(|| {
            for _ in 0..REPEATS {
                for name in NAMES {
                    let _ = black_box(QualifiedName::new(black_box(name)));
                }
            }
        });
    });
    group.finish();
}

/// Functions in the similar-pairs benchmark: a large project's worth of small functions.
const FUNCTIONS: u32 = 3_000;
/// Tokens every function starts with: the boilerplate (getter, constructor) all of them share.
const BOILERPLATE_TOKENS: u32 = 40;
/// Tokens that are the function's own.
const OWN_TOKENS: u32 = 40;
/// The limit of `conventions.duplicate_max_hash_sharing` in `config/default.toml`.
const MAX_SHARING: usize = 100;
/// Smallest similarity that counts, as `conventions.duplicate_similarity_percent`.
const MINIMUM_PERCENT: u32 = 85;

fn boilerplate_bodies() -> Vec<Fingerprints> {
    let Ok(params) = FingerprintParams::new(5, 4) else {
        return Vec::new();
    };
    (0..FUNCTIONS)
        .map(|function| {
            let tokens: Vec<TokenId> = (0..BOILERPLATE_TOKENS)
                .map(|n| TokenId::node_kind(&format!("boiler{n}")))
                .chain((0..OWN_TOKENS).map(|n| TokenId::node_kind(&format!("own{function}_{n}"))))
                .collect();
            winnow(&tokens, params)
        })
        .collect()
}

/// Every function shares the boilerplate hashes: without a limit that is a count for each of the
/// 4.5 million pairs.
fn similar_boilerplate(c: &mut Criterion) {
    let bodies = boilerplate_bodies();
    let refs: Vec<&Fingerprints> = bodies.iter().collect();
    let mut group = c.benchmark_group("domain");
    group.bench_function("similar_pairs_boilerplate", |b| {
        b.iter(|| {
            black_box(similar_pairs(
                black_box(&refs),
                MINIMUM_PERCENT,
                MAX_SHARING,
            ))
        });
    });
    group.finish();
}

fn main() {
    let mut criterion = Criterion::default().configure_from_args();
    qualified_names(&mut criterion);
    similar_boilerplate(&mut criterion);
    criterion.final_summary();
}
