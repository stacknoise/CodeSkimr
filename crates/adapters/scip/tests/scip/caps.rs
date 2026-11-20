//! Caps of the import (issues #246, #261): one symbol keeps a bounded target list, and one import
//! produces a bounded number of references.

use std::time::{Duration, Instant};

use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_domain::SymbolKind;
use codeskimr_scip::{Creation, ScipError, SnapshotView, decode_index, import_with_creation};

use crate::common::{definition, document, file, index, scip, symbol, usage};
use crate::mapping::{run, service_world};

/// `MAX_TARGETS_PER_SYMBOL` of the crate; the test names the number on purpose.
const TARGET_CAP: u64 = 1024;
/// Definitions of the one partial class, more than the cap.
const MANY_PARTS: u64 = TARGET_CAP + 200;

/// A class defined in `parts` files and used in one more file.
fn partial_class_world(
    parts: u64,
) -> (
    Vec<u8>,
    Vec<codeskimr_domain::SourceFile>,
    Vec<codeskimr_domain::Symbol>,
) {
    let class = scip("ns/Foo#");
    let mut files = Vec::new();
    let mut symbols = Vec::new();
    let mut documents = Vec::new();
    for part in 1..=parts {
        let path = format!("src/p{part}.cs");
        files.push(file(part, &path));
        symbols.push(symbol(
            part,
            part,
            SymbolKind::Class,
            "Foo",
            ((0, 0), (9, 1)),
        ));
        documents.push(document(&path, &[definition(&[0, 14, 17], &class)]));
    }
    let user = parts + 1;
    files.push(file(user, "src/use.cs"));
    symbols.push(symbol(
        user,
        user,
        SymbolKind::Method,
        "Use",
        ((0, 0), (9, 1)),
    ));
    documents.push(document("src/use.cs", &[usage(&[4, 8, 11], &class)]));
    (index("", &documents), files, symbols)
}

#[test]
fn a_symbol_with_thousands_of_definitions_keeps_a_bounded_target_list() {
    let (bytes, files, symbols) = partial_class_world(MANY_PARTS);
    let started = Instant::now();
    let outcome = run(&bytes, &files, &symbols, &[]);
    let time = started.elapsed();
    assert_eq!(outcome.references.len() as u64, TARGET_CAP);
    assert!(time < Duration::from_secs(20), "{time:?}");
}

#[test]
fn a_symbol_with_exactly_the_cap_of_definitions_keeps_all_targets() {
    let (bytes, files, symbols) = partial_class_world(TARGET_CAP);
    let outcome = run(&bytes, &files, &symbols, &[]);
    assert_eq!(outcome.references.len() as u64, TARGET_CAP);
}

fn limited(max_references: usize) -> Creation<'static> {
    Creation {
        max_references,
        ..Creation::off()
    }
}

/// Two calls of `Run` from `Go`, and the import with the given limit.
fn import_two_calls(max_references: usize) -> Result<usize, ScipError> {
    let (files, symbols) = service_world();
    let run_symbol = scip("ns/Svc#Run().");
    let bytes = index(
        "",
        &[
            document("src/a.cs", &[definition(&[2, 16, 19], &run_symbol)]),
            document(
                "src/b.cs",
                &[
                    usage(&[3, 12, 15], &run_symbol),
                    usage(&[4, 12, 15], &run_symbol),
                ],
            ),
        ],
    );
    let decoded = decode_index(&bytes, 1 << 30).expect("decodes");
    let view = SnapshotView {
        files: &files,
        symbols: &symbols,
        references: &[],
    };
    import_with_creation(&decoded, &view, 0.5, &limited(max_references))
        .map(|outcome| outcome.references.len())
}

#[test]
fn an_import_within_the_reference_limit_succeeds() {
    assert_eq!(import_two_calls(2).expect("within the limit"), 2);
}

#[test]
fn an_import_past_the_reference_limit_fails_with_the_size_code() {
    let error = import_two_calls(1).expect_err("one reference too many");
    assert!(matches!(error, ScipError::TooManyReferences { limit: 1 }));
    assert_eq!(error.code(), ErrorCode::ScipIndexTooLarge);
}
