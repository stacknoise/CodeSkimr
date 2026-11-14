//! Benchmark of the duplicate check over a large project (issue #99): the pairs of similar
//! functions among 5,000 fingerprinted bodies. Every body starts with the same run of tokens
//! (boilerplate like a getter or a constructor), so a few hashes have thousands of owners and
//! would cost the square of that number without the limit of `conventions.duplicate_max_owners`.

use std::hint::black_box;

use codeskimr_bench::random::Lcg;
use codeskimr_domain::{FingerprintParams, Fingerprints, TokenId, similar_pairs, winnow};
use criterion::Criterion;

const BODIES: usize = 5_000;
/// Tokens every body starts with.
const BOILERPLATE: usize = 40;
/// Tokens of a body after the boilerplate.
const OWN_TOKENS: usize = 40;
/// Distinct token kinds the own tokens are drawn from.
const KINDS: usize = 400;
const NGRAM_SIZE: usize = 5;
const WINDOW_SIZE: usize = 4;
const SEED: u64 = 11;
/// The defaults of `config/default.toml`.
const MINIMUM_PERCENT: u32 = 85;
const MAX_OWNERS: usize = 50;

fn token(kind: usize) -> TokenId {
    TokenId::node_kind(&format!("kind{kind}"))
}

fn bodies() -> Option<Vec<Fingerprints>> {
    let params = FingerprintParams::new(NGRAM_SIZE, WINDOW_SIZE).ok()?;
    let mut random = Lcg::new(SEED);
    Some(
        (0..BODIES)
            .map(|_| {
                let tokens: Vec<TokenId> = (0..BOILERPLATE)
                    .map(token)
                    .chain((0..OWN_TOKENS).map(|_| token(BOILERPLATE + random.below(KINDS))))
                    .collect();
                winnow(&tokens, params)
            })
            .collect(),
    )
}

fn pairs(c: &mut Criterion) -> Option<()> {
    let bodies = bodies()?;
    let refs: Vec<&Fingerprints> = bodies.iter().collect();
    let mut group = c.benchmark_group("similar");
    group.sample_size(10);
    group.bench_function("pairs_boilerplate_5k", |b| {
        b.iter(|| black_box(similar_pairs(&refs, MINIMUM_PERCENT, MAX_OWNERS)));
    });
    group.finish();
    Some(())
}

fn main() -> std::process::ExitCode {
    let mut criterion = Criterion::default().configure_from_args();
    if pairs(&mut criterion).is_none() {
        return std::process::ExitCode::FAILURE;
    }
    criterion.final_summary();
    std::process::ExitCode::SUCCESS
}
