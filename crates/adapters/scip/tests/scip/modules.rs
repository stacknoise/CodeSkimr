//! Uses of module paths map onto the module symbols of level 1 by position (ADR 0061).

use codeskimr_domain::{Reference, ReferenceKind, SourceFile, Symbol, SymbolKind};
use codeskimr_scip::{
    Creation, ImportOutcome, ScipError, SnapshotView, decode_index, import_with_creation,
};

use crate::common::{LIMIT, definition, document, file, index, level_one, symbol, usage};

/// A symbol of the Rust indexer's shape.
fn rust(package: &str, descriptors: &str) -> String {
    format!("rust-analyzer cargo {package} 0.1.0 {descriptors}")
}

fn core_crate() -> String {
    rust("core", "crate/")
}

fn other_crate() -> String {
    rust("other", "crate/")
}

fn ports() -> String {
    rust("core", "ports/")
}

/// Files: `lib.rs` and `build.rs` of the package `core`, its `ports/mod.rs` and `user.rs`, and
/// `lib.rs` of the package `other`.
fn files() -> Vec<SourceFile> {
    vec![
        file(1, "crates/core/src/lib.rs"),
        file(2, "crates/core/build.rs"),
        file(3, "crates/core/src/ports/mod.rs"),
        file(4, "crates/core/src/user.rs"),
        file(5, "crates/other/src/lib.rs"),
    ]
}

/// A module symbol per file, named after the file as level 1 does (`lib`, `build`, `mod`), and a
/// function in `user.rs`, `lib.rs` of `other` and `build.rs`.
fn symbols() -> Vec<Symbol> {
    let whole = |end: u32| ((0, 0), (end, 0));
    vec![
        symbol(10, 1, SymbolKind::Module, "lib", whole(20)),
        symbol(11, 2, SymbolKind::Module, "build", whole(10)),
        symbol(12, 3, SymbolKind::Module, "mod", whole(5)),
        symbol(13, 4, SymbolKind::Module, "user", whole(30)),
        symbol(14, 5, SymbolKind::Module, "lib", whole(8)),
        symbol(21, 4, SymbolKind::Function, "run", ((2, 0), (10, 1))),
        symbol(22, 5, SymbolKind::Function, "go", ((1, 0), (6, 1))),
        symbol(23, 2, SymbolKind::Function, "setup", ((1, 0), (5, 1))),
    ]
}

/// The index of the package `core` and `other`: every file defines its module over the whole file,
/// `lib.rs` and `build.rs` of `core` the same symbol `crate/`.
fn documents() -> Vec<Vec<u8>> {
    vec![
        document(
            "crates/core/src/lib.rs",
            &[definition(&[0, 0, 20, 0], &core_crate())],
        ),
        document(
            "crates/core/build.rs",
            &[
                definition(&[0, 0, 10, 0], &core_crate()),
                usage(&[2, 4, 9], &core_crate()),
            ],
        ),
        document(
            "crates/core/src/ports/mod.rs",
            &[definition(&[0, 0, 5, 0], &ports())],
        ),
        document(
            "crates/core/src/user.rs",
            &[
                usage(&[3, 4, 9], &core_crate()),
                usage(&[4, 4, 9], &ports()),
            ],
        ),
        document(
            "crates/other/src/lib.rs",
            &[
                definition(&[0, 0, 8, 0], &other_crate()),
                usage(&[2, 4, 9], &core_crate()),
            ],
        ),
    ]
}

fn import(
    documents: &[Vec<u8>],
    snapshot: (&[SourceFile], &[Symbol], &[Reference]),
    library_root: &str,
) -> ImportOutcome {
    try_import(documents, snapshot, library_root).expect("imports")
}

/// The import, which fails if the uses held none that mapped.
fn try_import(
    documents: &[Vec<u8>],
    (files, symbols, references): (&[SourceFile], &[Symbol], &[Reference]),
    library_root: &str,
) -> Result<ImportOutcome, ScipError> {
    let decoded = decode_index(&index("file:///p", documents), LIMIT).expect("decodes");
    let view = SnapshotView {
        files,
        symbols,
        references,
    };
    let creation = Creation {
        library_root,
        ..Creation::off()
    };
    import_with_creation(&decoded, &view, 0.0, &creation)
}

/// The references as `(file, from, to)`, in the order of the files.
fn edges(outcome: &ImportOutcome) -> Vec<(u64, u64, u64)> {
    let mut found: Vec<(u64, u64, u64)> = outcome
        .references
        .iter()
        .map(|r| (r.file.get(), r.from.get(), r.to.get()))
        .collect();
    found.sort_unstable();
    found
}

#[test]
fn a_crate_root_maps_onto_the_module_of_the_file_that_the_index_names() {
    let outcome = import(&documents(), (&files(), &symbols(), &[]), "lib.rs");
    // `core::crate` is used in `user.rs` and `other`; both reach `lib.rs` of `core`.
    assert!(edges(&outcome).contains(&(4, 21, 10)));
    assert!(edges(&outcome).contains(&(5, 22, 10)));
}

#[test]
fn a_module_in_a_mod_rs_file_maps_onto_the_module_of_that_file() {
    let outcome = import(&documents(), (&files(), &symbols(), &[]), "lib.rs");
    assert!(edges(&outcome).contains(&(4, 21, 12)));
    let reference = outcome
        .references
        .iter()
        .find(|r| r.to.get() == 12)
        .expect("a use of ports");
    assert_eq!(reference.kind, ReferenceKind::Read);
    assert!(reference.verified);
}

#[test]
fn a_symbol_with_several_roots_gives_each_use_to_one_of_them() {
    let outcome = import(&documents(), (&files(), &symbols(), &[]), "lib.rs");
    // (a) the use in `build.rs` (inside `setup`) belongs to the module of `build.rs`, which stands
    // in the same file.
    assert!(edges(&outcome).contains(&(2, 23, 11)));
    // (b) the use in `src/user.rs` belongs to the root in `crates/core/src`, the deepest folder
    // above it; not to `build.rs` in `crates/core`.
    assert!(edges(&outcome).contains(&(4, 21, 10)));
    assert!(!edges(&outcome).contains(&(4, 21, 11)));
    // (c) the use from another package belongs to the root that bears the name of the library root.
    assert!(edges(&outcome).contains(&(5, 22, 10)));
}

#[test]
fn without_a_library_root_a_use_that_no_folder_places_stays_unmapped() {
    let with = import(&documents(), (&files(), &symbols(), &[]), "lib.rs");
    let without = import(&documents(), (&files(), &symbols(), &[]), "");
    assert!(edges(&with).contains(&(5, 22, 10)));
    assert!(!edges(&without).contains(&(5, 22, 10)));
    assert_eq!(without.stats.unmapped, with.stats.unmapped + 1);
    // The uses that the folders place do not depend on the key.
    assert!(edges(&without).contains(&(4, 21, 10)));
}

#[test]
fn a_symbol_with_one_root_needs_no_rule() {
    let outcome = import(&documents(), (&files(), &symbols(), &[]), "");
    assert!(edges(&outcome).contains(&(4, 21, 12)));
}

#[test]
fn a_definition_that_is_not_the_whole_file_keeps_the_name_rule() {
    let inline = rust("core", "user/inner/");
    let other = rust("core", "elsewhere/");
    let mut symbols = symbols();
    symbols.push(symbol(
        30,
        4,
        SymbolKind::Namespace,
        "inner",
        ((12, 0), (20, 1)),
    ));
    let docs = vec![document(
        "crates/core/src/user.rs",
        &[
            // An inline `mod inner { ... }`: the range is the name, the name rule matches.
            definition(&[12, 4, 9], &inline),
            usage(&[3, 4, 9], &inline),
            // The range starts at the first position but ends inside the file: no root.
            definition(&[0, 0, 0, 3], &other),
            usage(&[4, 4, 9], &other),
        ],
    )];
    let outcome = import(&docs, (&files(), &symbols, &[]), "lib.rs");
    assert_eq!(edges(&outcome), [(4, 21, 30)]);
    assert_eq!(outcome.stats.unmapped, 1);
}

#[test]
fn only_a_namespace_is_mapped_by_position() {
    // A type defined over the whole file under another name is not guessed.
    let thing = rust("core", "Thing#");
    let docs = vec![document(
        "crates/core/src/user.rs",
        &[
            definition(&[0, 0, 30, 0], &thing),
            usage(&[3, 4, 9], &thing),
        ],
    )];
    let outcome = try_import(&docs, (&files(), &symbols(), &[]), "lib.rs");
    assert!(matches!(
        outcome,
        Err(ScipError::MappingTooLow {
            mapped: 0,
            total: 1
        })
    ));
}

#[test]
fn a_file_without_a_module_symbol_maps_nothing() {
    let symbols: Vec<Symbol> = symbols()
        .into_iter()
        .filter(|s| s.kind != SymbolKind::Module || s.file.get() != 3)
        .collect();
    let outcome = import(&documents(), (&files(), &symbols, &[]), "lib.rs");
    assert!(!edges(&outcome).iter().any(|(_, _, to)| *to == 12));
    assert_eq!(outcome.stats.unmapped, 1);
}

#[test]
fn the_name_rule_still_decides_where_the_name_fits() {
    let user = rust("core", "user/");
    let docs = vec![
        document(
            "crates/core/src/user.rs",
            &[definition(&[0, 0, 30, 0], &user)],
        ),
        document("crates/other/src/lib.rs", &[usage(&[2, 4, 9], &user)]),
    ];
    let outcome = import(&docs, (&files(), &symbols(), &[]), "lib.rs");
    assert_eq!(edges(&outcome), [(5, 22, 13)]);
}

#[test]
fn uses_of_symbols_defined_only_outside_the_scan_are_counted_apart() {
    let skipped_module = rust("core", "config/");
    let skipped_type = rust("core", "config/Settings#");
    let mut docs = documents();
    docs.push(document(
        "crates/core/src/config/mod.rs",
        &[
            definition(&[0, 0, 9, 0], &skipped_module),
            definition(&[2, 11, 19], &skipped_type),
        ],
    ));
    docs.push(document(
        "crates/core/src/user.rs",
        &[
            usage(&[5, 4, 10], &skipped_module),
            usage(&[6, 4, 12], &skipped_type),
            usage(&[7, 4, 12], &skipped_type),
        ],
    ));
    let outcome = import(&docs, (&files(), &symbols(), &[]), "lib.rs");
    assert_eq!(outcome.stats.outside_scan, 3);
    assert_eq!(outcome.stats.files_unmatched, 1);
    // They are no use that failed to map: the share of `min_mapped` does not count them.
    assert_eq!(outcome.stats.unmapped, 0);
    assert_eq!(outcome.stats.dropped(), outcome.stats.library + 3);
}

#[test]
fn a_symbol_that_is_defined_inside_and_outside_the_scan_stays_unmapped_when_it_finds_no_target() {
    let both = rust("core", "dual#");
    let docs = vec![
        document("crates/core/src/user.rs", &[definition(&[2, 0, 4], &both)]),
        document("crates/core/src/gone.rs", &[definition(&[2, 0, 4], &both)]),
        document("crates/other/src/lib.rs", &[usage(&[2, 4, 9], &both)]),
    ];
    let outcome = try_import(&docs, (&files(), &symbols(), &[]), "");
    // Nothing mapped, and the use is held against the index, not set aside as outside the scan.
    assert!(matches!(
        outcome,
        Err(ScipError::MappingTooLow {
            mapped: 0,
            total: 1
        })
    ));
}

#[test]
fn outside_the_scan_uses_stay_out_of_the_share_that_must_map() {
    let skipped = rust("core", "config/");
    let mut docs = documents();
    docs.push(document(
        "crates/core/src/config/mod.rs",
        &[definition(&[0, 0, 9, 0], &skipped)],
    ));
    let mut uses = vec![usage(&[3, 4, 9], &core_crate())];
    uses.extend((0..20).map(|n| usage(&[5 + n, 4, 10], &skipped)));
    docs.push(document("crates/core/src/user.rs", &uses));
    let decoded = decode_index(&index("file:///p", &docs), LIMIT).expect("decodes");
    let (files, symbols) = (files(), symbols());
    let view = SnapshotView {
        files: &files,
        symbols: &symbols,
        references: &[],
    };
    let creation = Creation {
        library_root: "lib.rs",
        ..Creation::off()
    };
    // 1 of 20 uses is mapped from `user.rs`, but 20 of them lie outside the scan: the share
    // that counts is mapped against mapped and unmapped.
    let outcome = import_with_creation(&decoded, &view, 0.9, &creation);
    assert!(outcome.is_ok(), "{outcome:?}");
}

#[test]
fn level_one_is_not_changed_and_other_files_keep_their_references() {
    let kept = level_one(900, (21, 22), 4, (8, 0), ReferenceKind::Call);
    let docs = vec![document(
        "crates/other/src/lib.rs",
        &[
            definition(&[0, 0, 8, 0], &other_crate()),
            usage(&[2, 4, 9], &other_crate()),
        ],
    )];
    let outcome = import(&docs, (&files(), &symbols(), &[kept]), "lib.rs");
    assert_eq!(outcome.symbols, Vec::new());
    assert_eq!(outcome.covered.len(), 1);
    assert!(outcome.references.contains(&kept));
}

#[test]
fn a_new_scan_gives_the_module_new_ids_and_the_next_import_follows_them() {
    // Rescan: the module symbol of `lib.rs` has another id after a scan of its changed file; the
    // next import points the same use at the new id.
    let first = import(&documents(), (&files(), &symbols(), &[]), "lib.rs");
    let renumbered: Vec<Symbol> = symbols()
        .into_iter()
        .map(|s| {
            if s.id.get() == 10 {
                symbol(110, 1, SymbolKind::Module, "lib", ((0, 0), (20, 0)))
            } else {
                s
            }
        })
        .collect();
    let second = import(&documents(), (&files(), &renumbered, &[]), "lib.rs");
    assert!(edges(&first).contains(&(4, 21, 10)));
    assert!(edges(&second).contains(&(4, 21, 110)));
    assert!(!edges(&second).contains(&(4, 21, 10)));
}

#[test]
fn two_roots_in_one_folder_are_told_apart_by_the_name_of_the_library_root() {
    // A package with `lib.rs` and `main.rs`: a use in `x.rs` has two roots in the deepest folder
    // above it, so the folders do not decide and the library root does.
    let files = vec![
        file(1, "crates/pkg/src/lib.rs"),
        file(2, "crates/pkg/src/main.rs"),
        file(3, "crates/pkg/src/x.rs"),
    ];
    let whole = |end: u32| ((0, 0), (end, 0));
    let symbols = vec![
        symbol(10, 1, SymbolKind::Module, "lib", whole(9)),
        symbol(11, 2, SymbolKind::Module, "main", whole(9)),
        symbol(12, 3, SymbolKind::Module, "x", whole(9)),
        symbol(20, 3, SymbolKind::Function, "f", ((1, 0), (5, 1))),
    ];
    let krate = rust("pkg", "crate/");
    let docs = vec![
        document(
            "crates/pkg/src/lib.rs",
            &[definition(&[0, 0, 9, 0], &krate)],
        ),
        document(
            "crates/pkg/src/main.rs",
            &[definition(&[0, 0, 9, 0], &krate)],
        ),
        document("crates/pkg/src/x.rs", &[usage(&[2, 4, 9], &krate)]),
    ];
    let outcome = import(&docs, (&files, &symbols, &[]), "lib.rs");
    assert_eq!(edges(&outcome), [(3, 20, 10)]);
    let none = try_import(&docs, (&files, &symbols, &[]), "");
    assert!(matches!(none, Err(ScipError::MappingTooLow { .. })));
}
