//! The import must grow about linearly with the size of one file (issue #220): a minified or
//! generated file has thousands of symbols and occurrences on a few lines.

use std::time::{Duration, Instant};

use codeskimr_domain::{Reference, ReferenceKind, SourceFile, Symbol, SymbolKind};
use codeskimr_scip::{SnapshotView, decode_index, import_references};

use crate::common::{definition, document, file, index, level_one, scip, span, symbol, usage};

/// Columns one method takes on the single line of the file.
const WIDTH: u32 = 20;

/// The method `i` of the file: its symbol, and what the index and level 1 say about it. Method `i`
/// is defined at its name and uses method `i + 1` (the last one uses the first); level 1 has a
/// reference that starts one column before the usage, so it is found only by the line.
fn method(i: u32, count: u32) -> (Symbol, Vec<Vec<u8>>, Reference) {
    let left = WIDTH * i;
    let name = format!("M{i}");
    let id = u64::from(i) + 2;
    let next = (i + 1) % count;
    let own = scip(&format!("ns/Svc#{name}()."));
    let used = scip(&format!("ns/Svc#M{next}()."));
    let name_end = left + 2 + u32::try_from(name.len()).expect("short name");
    let method = symbol(
        id,
        1,
        SymbolKind::Method,
        &name,
        ((0, left), (0, left + WIDTH - 2)),
    );
    let occurrences = vec![
        definition(&[0, left + 2, name_end], &own),
        usage(&[0, left + 8, left + 10], &used),
    ];
    let reference = Reference {
        span: span((0, left + 7), (0, left + 11)),
        ..level_one(
            u64::from(i) + 1,
            (id, u64::from(next) + 2),
            1,
            (0, left + 7),
            ReferenceKind::Call,
        )
    };
    (method, occurrences, reference)
}

/// One file, one class around everything and `count` methods side by side on its first line.
fn world(count: u32) -> (Vec<u8>, Vec<SourceFile>, Vec<Symbol>, Vec<Reference>) {
    let class = symbol(
        1,
        1,
        SymbolKind::Class,
        "Svc",
        ((0, 0), (0, WIDTH * count + 10)),
    );
    let mut symbols = vec![class];
    let mut references = Vec::new();
    let mut occurrences = Vec::new();
    for i in 0..count {
        let (method, found, reference) = method(i, count);
        symbols.push(method);
        occurrences.extend(found);
        references.push(reference);
    }
    let bytes = index("file:///p", &[document("src/min.cs", &occurrences)]);
    (bytes, vec![file(1, "src/min.cs")], symbols, references)
}

/// The time the import of a file with `count` methods takes, and the number of references.
fn time_of(count: u32) -> (Duration, usize) {
    let (bytes, files, symbols, references) = world(count);
    let decoded = decode_index(&bytes, 1 << 30).expect("decodes");
    let view = SnapshotView {
        files: &files,
        symbols: &symbols,
        references: &references,
    };
    let started = Instant::now();
    let outcome = import_references(&decoded, &view, 0.5).expect("imports");
    (started.elapsed(), outcome.references.len())
}

/// Prints the times with `cargo test -p codeskimr_scip --test scip scale -- --nocapture`.
#[test]
fn a_file_with_thousands_of_symbols_on_one_line_imports_in_time() {
    for count in [2_000_u32, 4_000, 8_000] {
        let (time, references) = time_of(count);
        eprintln!("scale: {count} methods: {time:?} ({references} references)");
        assert_eq!(references, count as usize);
    }
    // Twenty thousand methods: far beyond what a quadratic import finishes in this time.
    let (time, references) = time_of(20_000);
    eprintln!("scale: 20000 methods: {time:?}");
    assert_eq!(references, 20_000);
    assert!(time < Duration::from_secs(10), "{time:?}");
}
