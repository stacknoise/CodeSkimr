//! Replacement per file and the threshold of mapped occurrences.

use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_domain::ReferenceKind;
use codeskimr_scip::ScipError;

use crate::common::{definition, document, index, level_one, run_index, scip, usage};
use crate::mapping::{run, run_with, service_world};

#[test]
fn all_level_one_references_of_a_covered_file_are_replaced() {
    let (files, symbols) = service_world();
    let old = [
        level_one(1, (5, 3), 2, (3, 12), ReferenceKind::Call),
        level_one(2, (5, 3), 2, (6, 0), ReferenceKind::Call),
        level_one(3, (3, 2), 1, (8, 0), ReferenceKind::Read),
    ];
    let outcome = run(&run_index(&[[3, 12, 15]]), &files, &symbols, &old);
    assert!(outcome.references.iter().all(|r| r.verified));
    let places: Vec<_> = outcome
        .references
        .iter()
        .map(|r| (r.file.get(), r.span.start().line))
        .collect();
    assert_eq!(places, [(2, 3)]);
    assert_eq!(outcome.stats.files_replaced, 2);
}

#[test]
fn a_file_without_an_index_keeps_its_level_one_references() {
    let (mut files, mut symbols) = service_world();
    files.push(crate::common::file(3, "src/c.cs"));
    symbols.push(crate::common::symbol(
        6,
        3,
        codeskimr_domain::SymbolKind::Method,
        "Other",
        ((0, 0), (9, 1)),
    ));
    let kept = level_one(9, (6, 2), 3, (4, 4), ReferenceKind::Call);
    let old = [level_one(1, (5, 3), 2, (6, 0), ReferenceKind::Call), kept];
    let outcome = run(&run_index(&[[3, 12, 15]]), &files, &symbols, &old);
    assert!(outcome.references.contains(&kept));
    assert!(!outcome.references.iter().any(|r| r.id.get() == 1));
    assert_eq!(outcome.references.len(), 2);
    assert_eq!(outcome.stats.files_replaced, 2);
}

#[test]
fn documents_that_match_no_file_are_counted() {
    let (files, symbols) = service_world();
    let run_symbol = scip("ns/Svc#Run().");
    let bytes = index(
        "",
        &[
            document("src/a.cs", &[definition(&[2, 16, 19], &run_symbol)]),
            document("obj/generated.cs", &[usage(&[1, 1, 2], &run_symbol)]),
        ],
    );
    let outcome = run(&bytes, &files, &symbols, &[]);
    assert_eq!(outcome.stats.files_unmatched, 1);
    assert_eq!(outcome.stats.files_replaced, 1);
}

fn share_bytes() -> Vec<u8> {
    let run_symbol = scip("ns/Svc#Run().");
    let ghost = scip("ns/Ghost#Haunt().");
    let library = "scip-dotnet nuget Lib 1 Lib/Thing#";
    index(
        "",
        &[
            document(
                "src/a.cs",
                &[
                    definition(&[2, 16, 19], &run_symbol),
                    definition(&[90, 4, 9], &ghost),
                ],
            ),
            document(
                "src/b.cs",
                &[
                    usage(&[3, 12, 15], &run_symbol),
                    usage(&[4, 12, 15], &ghost),
                    usage(&[5, 12, 15], &ghost),
                    usage(&[6, 12, 15], library),
                    usage(&[7, 12, 15], library),
                    usage(&[8, 12, 15], library),
                ],
            ),
        ],
    )
}

#[test]
fn a_low_mapping_share_fails_the_index_and_leaves_level_one() {
    let (files, symbols) = service_world();
    let old = [level_one(1, (5, 3), 2, (6, 0), ReferenceKind::Call)];
    let snapshot = (files.as_slice(), symbols.as_slice(), old.as_slice());
    let error = run_with(&share_bytes(), snapshot, 0.5).expect_err("one of three is mapped");
    assert!(matches!(
        error,
        ScipError::MappingTooLow {
            mapped: 1,
            total: 3
        }
    ));
    assert_eq!(error.code(), ErrorCode::ScipMappingTooLow);
    let outcome = run_with(&share_bytes(), snapshot, 0.3).expect("a third is enough");
    assert_eq!(outcome.stats.mapped, 1);
    assert_eq!(outcome.stats.unmapped, 2);
    assert_eq!(outcome.stats.library, 3);
}

#[test]
fn an_index_that_covers_no_file_fails() {
    let (files, symbols) = service_world();
    let bytes = index("", &[document("elsewhere/x.cs", &[])]);
    let result = run_with(&bytes, (&files, &symbols, &[]), 0.0);
    assert!(matches!(result, Err(ScipError::MappingTooLow { .. })));
}

#[test]
fn an_index_where_no_use_maps_fails_and_leaves_level_one() {
    let (files, symbols) = service_world();
    let library = "scip-dotnet nuget Lib 1 Lib/Thing#";
    let run_symbol = scip("ns/Svc#Run().");
    let only_library = index("", &[document("src/b.cs", &[usage(&[3, 12, 15], library)])]);
    // The indexer never sets the definition bit, so every symbol looks like a library symbol.
    let no_definition_bit = index(
        "",
        &[
            document("src/a.cs", &[usage(&[2, 16, 19], &run_symbol)]),
            document("src/b.cs", &[usage(&[3, 12, 15], &run_symbol)]),
        ],
    );
    let old = [level_one(1, (5, 3), 2, (6, 0), ReferenceKind::Call)];
    for bytes in [only_library, no_definition_bit] {
        let result = run_with(&bytes, (&files, &symbols, &old), 0.0);
        assert!(matches!(
            result,
            Err(ScipError::MappingTooLow { mapped: 0, .. })
        ));
    }
}
