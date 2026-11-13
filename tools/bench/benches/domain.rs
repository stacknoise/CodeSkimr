//! Benchmarks of the domain model. They prove the harness; real budgets get their own
//! benchmarks as the pipeline grows (see `budgets.toml`).

use std::hint::black_box;

use codeskimr_domain::QualifiedName;
use criterion::Criterion;

const NAMES: [&str; 4] = [
    "Namespace.Type.member",
    "Namespace.Sub.Type",
    "Root",
    "A.B.C.D.E.F.G.H",
];

fn qualified_names(c: &mut Criterion) {
    let mut group = c.benchmark_group("domain");
    group.bench_function("qualified_name_new", |b| {
        b.iter(|| {
            for name in NAMES {
                let _ = black_box(QualifiedName::new(black_box(name)));
            }
        });
    });
    group.finish();
}

fn main() {
    let mut criterion = Criterion::default().configure_from_args();
    qualified_names(&mut criterion);
    criterion.final_summary();
}
