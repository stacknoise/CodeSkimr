//! Benchmark of loading the language packs (issue #426). Since ADR 0059 a pack is loaded when the
//! first file of its language is analyzed, and the scan time leaves that load out. This runs the
//! real binary on a folder with one Rust and one C# file (C# has the longest pack) and takes the
//! load time that the scan reports. Every run is a new process with an empty database, so both
//! packs are loaded every time. The result is checked against `budgets.pack_load_ms` (ADR 0063), and a
//! regression against the base fails like for every benchmark.

use std::fs;
use std::io;

use criterion::Criterion;

mod common;

use common::{SCRATCH, build_binary, measure, pack_load_time, root, wipe};

const SAMPLES: usize = 10;
const MEASUREMENT_SECONDS: u64 = 15;
const CORPUS_FOLDER: &str = "corpus-pack-load";
const DATA_FOLDER: &str = "data-pack-load";
const RUST_FILE: &str = "lib.rs";
const RUST_SOURCE: &str = "pub fn one() -> u32 {\n    1\n}\n";
const CSHARP_FILE: &str = "One.cs";
const CSHARP_SOURCE: &str = "class One\n{\n    int Value() { return 1; }\n}\n";

fn loads(c: &mut Criterion) -> io::Result<()> {
    let scratch = root().join(SCRATCH);
    let corpus = scratch.join(CORPUS_FOLDER);
    wipe(&corpus)?;
    fs::create_dir_all(&corpus)?;
    fs::write(corpus.join(RUST_FILE), RUST_SOURCE)?;
    fs::write(corpus.join(CSHARP_FILE), CSHARP_SOURCE)?;
    let binary = build_binary()?;
    let data = scratch.join(DATA_FOLDER);
    measure(
        c,
        ("pack_load", "rust_and_csharp"),
        (SAMPLES, MEASUREMENT_SECONDS),
        || {
            wipe(&data)?;
            pack_load_time(&binary, &corpus, &data)
        },
    );
    Ok(())
}

fn main() -> io::Result<()> {
    let mut criterion = Criterion::default().configure_from_args();
    loads(&mut criterion)?;
    criterion.final_summary();
    Ok(())
}
