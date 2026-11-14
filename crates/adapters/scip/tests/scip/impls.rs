//! The impl blocks of rust-analyzer: `Self` and the methods of trait implementations (ADR 0033).
//! Only what is certain is mapped: a bare name is the type of the module of the block, a path
//! must fit one project type, a trait must be a trait of the project.

use codeskimr_domain::{
    Confidence, Reference, ReferenceKind, SourceFile, Symbol, SymbolId, SymbolKind,
};

use crate::common::{definition, document, file, index, symbol, usage};
use crate::mapping::{run, run_with, target};

/// A rust-analyzer symbol of the crate `app`.
fn rust(descriptors: &str) -> String {
    format!("rust-analyzer cargo app 0.1.0 {descriptors}")
}

/// `symbol`, declared inside the symbol `parent`.
fn inside(mut symbol: Symbol, parent: u64) -> Symbol {
    symbol.parent = Some(SymbolId::new(parent));
    symbol
}

fn imported(
    files: &[SourceFile],
    symbols: &[Symbol],
    documents: &[Vec<u8>],
) -> (Vec<(u64, u64, ReferenceKind)>, Vec<Reference>) {
    let outcome = run(&index("", documents), files, symbols, &[]);
    (target(&outcome), outcome.references)
}

/// Type uses of `Prints` (symbol 1) from the given symbols.
fn uses(from: &[u64]) -> Vec<(u64, u64, ReferenceKind)> {
    from.iter()
        .map(|f| (*f, 1, ReferenceKind::TypeUse))
        .collect()
}

/// Two files, `src/a.rs` with the struct `Prints` and a method, `src/b.rs` with a method only.
fn world() -> (Vec<SourceFile>, Vec<Symbol>) {
    let files = vec![file(1, "src/a.rs"), file(2, "src/b.rs")];
    let symbols = vec![
        symbol(1, 1, SymbolKind::Struct, "Prints", ((0, 0), (10, 1))),
        symbol(2, 1, SymbolKind::Method, "build", ((3, 4), (6, 5))),
        symbol(3, 2, SymbolKind::Method, "make", ((3, 4), (6, 5))),
    ];
    (files, symbols)
}

/// The definition of `Prints` in module `a` and an ordinary use of it, so that the index maps.
fn prints_in_a() -> Vec<u8> {
    document(
        "src/a.rs",
        &[
            definition(&[0, 11, 17], &rust("a/Prints#")),
            usage(&[8, 2, 8], &rust("a/Prints#")),
        ],
    )
}

#[test]
fn self_in_an_impl_block_is_a_use_of_the_type() {
    let (files, symbols) = world();
    let all = [
        definition(&[0, 11, 17], &rust("a/Prints#")),
        usage(&[4, 20, 24], &rust("a/impl#[Prints]")),
        usage(&[5, 8, 12], &rust("a/impl#[Prints][Display]")),
        usage(&[5, 20, 24], &rust("a/impl#[`Prints<Queued>`]")),
    ];
    let (edges, references) = imported(&files, &symbols, &[document("src/a.rs", &all)]);
    assert_eq!(edges, [(2, 1, ReferenceKind::TypeUse); 3]);
    assert!(references.iter().all(|r| r.verified));
    assert!(
        references
            .iter()
            .all(|r| r.confidence == Confidence::Certain)
    );
}

#[test]
fn a_bare_name_is_only_the_type_of_the_module_of_the_block() {
    let (files, symbols) = world();
    let b = document("src/b.rs", &[usage(&[4, 20, 24], &rust("b/impl#[Prints]"))]);
    let (edges, _) = imported(&files, &symbols, &[prints_in_a(), b]);
    // The type of module `a` is not guessed for a block in module `b`: it may be a library type.
    assert_eq!(edges, uses(&[1]));
}

#[test]
fn the_header_of_the_block_proves_the_type_of_another_module() {
    let (files, symbols) = world();
    let header = usage(&[3, 12, 18], &rust("a/Prints#"));
    let me = usage(&[4, 20, 24], &rust("b/impl#[Prints]"));
    let b = document("src/b.rs", &[header.clone(), me.clone()]);
    let (edges, _) = imported(&files, &symbols, &[prints_in_a(), b]);
    assert_eq!(edges, uses(&[1, 3, 3]));
    // A type of a library with that name in the same document makes it ambiguous.
    let library = usage(&[3, 30, 36], "rust-analyzer cargo std 1.0.0 io/Prints#");
    let b = document("src/b.rs", &[header, library, me]);
    let (edges, _) = imported(&files, &symbols, &[prints_in_a(), b]);
    assert_eq!(edges, uses(&[1, 3]));
}

#[test]
fn a_type_of_the_same_module_wins_over_one_of_another_module() {
    let (mut files, mut symbols) = world();
    files.push(file(3, "src/c.rs"));
    symbols.push(symbol(
        4,
        3,
        SymbolKind::Struct,
        "Prints",
        ((0, 0), (10, 1)),
    ));
    symbols.push(symbol(5, 3, SymbolKind::Method, "go", ((12, 4), (15, 5))));
    let c = document(
        "src/c.rs",
        &[
            definition(&[0, 11, 17], &rust("c/Prints#")),
            usage(&[13, 20, 24], &rust("c/impl#[Prints]")),
        ],
    );
    let (edges, _) = imported(&files, &symbols, &[prints_in_a(), c]);
    assert_eq!(
        edges,
        [
            (1, 1, ReferenceKind::TypeUse),
            (5, 4, ReferenceKind::TypeUse)
        ]
    );
}

#[test]
fn a_path_from_the_crate_root_names_that_module_and_not_the_module_of_the_block() {
    let (files, mut symbols) = world();
    symbols.push(symbol(4, 2, SymbolKind::Struct, "Prints", ((0, 0), (2, 1))));
    let b = document(
        "src/b.rs",
        &[
            definition(&[0, 7, 13], &rust("b/Prints#")),
            usage(&[4, 20, 24], &rust("b/impl#[`crate::a::Prints`]")),
        ],
    );
    let (edges, _) = imported(&files, &symbols, &[prints_in_a(), b]);
    assert_eq!(edges, uses(&[1, 3]));
}

#[test]
fn a_type_of_a_library_is_not_taken_for_a_project_type_of_the_same_name() {
    let (files, mut symbols) = world();
    symbols.push(symbol(
        4,
        1,
        SymbolKind::Struct,
        "Error",
        ((20, 0), (30, 1)),
    ));
    let a = document(
        "src/a.rs",
        &[
            definition(&[20, 11, 16], &rust("a/Error#")),
            usage(&[8, 2, 8], &rust("a/Error#")),
        ],
    );
    let impls = document(
        "src/b.rs",
        &[
            usage(&[4, 20, 24], &rust("b/impl#[`io::Error`]")),
            usage(&[5, 20, 24], &rust("b/impl#[`std::io::Error`][Display]")),
            usage(&[6, 20, 24], &rust("b/impl#[`crate::b::Error`]")),
            usage(&[7, 20, 24], &rust("b/impl#[Error]")),
        ],
    );
    let (edges, _) = imported(&files, &symbols, &[a, impls]);
    assert_eq!(edges, [(1, 4, ReferenceKind::TypeUse)]);
}

/// The trait `CodeIndex` of module `api` with `layer_edges`, and a struct `Store` with the method
/// of the same name, which implements it in module `store`.
fn trait_world() -> (Vec<SourceFile>, Vec<Symbol>) {
    let files = vec![file(1, "src/a.rs"), file(2, "src/b.rs")];
    let symbols = vec![
        symbol(1, 1, SymbolKind::Trait, "CodeIndex", ((0, 0), (5, 1))),
        inside(
            symbol(2, 1, SymbolKind::Method, "layer_edges", ((1, 4), (1, 30))),
            1,
        ),
        symbol(3, 2, SymbolKind::Struct, "Store", ((0, 0), (20, 1))),
        inside(
            symbol(4, 2, SymbolKind::Method, "layer_edges", ((3, 4), (6, 5))),
            3,
        ),
    ];
    (files, symbols)
}

fn trait_documents(implemented: &str) -> Vec<Vec<u8>> {
    vec![
        document(
            "src/a.rs",
            &[
                definition(&[0, 6, 15], &rust("api/CodeIndex#")),
                definition(&[1, 7, 18], &rust("api/CodeIndex#layer_edges().")),
            ],
        ),
        document(
            "src/b.rs",
            &[
                definition(&[0, 7, 12], &rust("store/Store#")),
                definition(&[3, 7, 18], &rust(implemented)),
            ],
        ),
    ]
}

#[test]
fn a_method_of_a_trait_implementation_overrides_the_method_of_the_trait() {
    let (files, symbols) = trait_world();
    for implemented in [
        "store/impl#[Store][CodeIndex]layer_edges().",
        "store/impl#[Store][`crate::api::CodeIndex`]layer_edges().",
        "store/impl#[Store][`api::CodeIndex<Edge>`]layer_edges().",
    ] {
        let (edges, references) = imported(&files, &symbols, &trait_documents(implemented));
        assert_eq!(edges, [(4, 2, ReferenceKind::Override)], "{implemented}");
        let reference = references.first().expect("one reference");
        assert!(reference.verified);
        assert_eq!(reference.confidence, Confidence::Certain);
        assert_eq!(reference.file.get(), 2);
    }
}

#[test]
fn a_trait_of_a_library_does_not_override_a_project_method_of_the_same_name() {
    let (files, mut symbols) = trait_world();
    // A struct `Hash` of the project with a method `hash`, and a trait method that is no trait's.
    symbols.push(symbol(5, 1, SymbolKind::Struct, "Hash", ((10, 0), (20, 1))));
    symbols.push(inside(
        symbol(6, 1, SymbolKind::Method, "hash", ((11, 4), (12, 5))),
        5,
    ));
    let mut documents = trait_documents("store/impl#[Store][`std::hash::Hash`]hash().");
    documents.push(document(
        "src/a.rs",
        &[
            definition(&[10, 7, 11], &rust("api/Hash#")),
            definition(&[11, 7, 11], &rust("api/Hash#hash().")),
        ],
    ));
    let (edges, _) = imported(&files, &symbols, &documents);
    assert_eq!(edges, []);
    // Nor does a struct that is no trait.
    let (edges, _) = imported(
        &files,
        &symbols,
        &trait_documents("store/impl#[Store][Hash]hash()."),
    );
    assert_eq!(edges, []);
}

#[test]
fn two_traits_of_one_name_are_told_apart_by_the_path_and_never_guessed() {
    let (mut files, mut symbols) = trait_world();
    files.push(file(3, "src/c.rs"));
    symbols.push(symbol(
        5,
        3,
        SymbolKind::Trait,
        "CodeIndex",
        ((0, 0), (5, 1)),
    ));
    symbols.push(inside(
        symbol(6, 3, SymbolKind::Method, "layer_edges", ((1, 4), (1, 30))),
        5,
    ));
    let mut documents = trait_documents("store/impl#[Store][CodeIndex]layer_edges().");
    documents.push(document(
        "src/c.rs",
        &[
            definition(&[0, 6, 15], &rust("other/CodeIndex#")),
            definition(&[1, 7, 18], &rust("other/CodeIndex#layer_edges().")),
        ],
    ));
    let (edges, _) = imported(&files, &symbols, &documents);
    assert_eq!(edges, [], "two traits of that name, no path");
    let mut by_path =
        trait_documents("store/impl#[Store][`crate::other::CodeIndex`]layer_edges().");
    by_path.push(documents.pop().expect("document of the second trait"));
    let (edges, _) = imported(&files, &symbols, &by_path);
    assert_eq!(edges, [(4, 6, ReferenceKind::Override)]);
}

#[test]
fn a_method_of_a_library_trait_gives_no_override() {
    let (files, symbols) = trait_world();
    let bytes = document(
        "src/b.rs",
        &[
            definition(&[0, 7, 12], &rust("store/Store#")),
            definition(&[3, 7, 18], &rust("store/impl#[Store][Display]fmt().")),
        ],
    );
    assert_eq!(imported(&files, &symbols, &[bytes]).0, []);
}

#[test]
fn a_local_symbol_that_looks_like_an_impl_block_is_ignored() {
    let (files, symbols) = world();
    let a = document(
        "src/a.rs",
        &[
            definition(&[0, 11, 17], &rust("a/Prints#")),
            usage(&[4, 20, 24], &rust("a/Prints#")),
            usage(&[5, 8, 12], "local impl#[Prints]"),
        ],
    );
    let outcome = run(&index("", &[a]), &files, &symbols, &[]);
    assert_eq!(target(&outcome), [(2, 1, ReferenceKind::TypeUse)]);
    assert_eq!(outcome.stats.ignored, 1);
}

#[test]
fn hostile_or_broken_impl_symbols_map_to_nothing_and_do_not_fail() {
    let (files, symbols) = world();
    let deep = rust(&format!("a/impl#[Nope]{}", "[X]".repeat(500)));
    let hostile = [
        deep,
        rust("a/impl#[ä"),
        rust("a/impl#[`"),
        rust("a/impl#[]"),
        rust("a/impl#"),
        rust("a/impl#[Prints"),
        rust("a/impl#[Prints]fmt("),
        rust("a/impl#[`a::::Prints`]"),
        rust("a/impl#[`super::Prints`]"),
        rust("a/impl#[`Vec<Prints>`][`Iterator`]next()."),
        "rust-analyzer cargo impl#[Prints] 0.1.0 a/Prints#".to_owned(),
    ];
    let mut occurrences = vec![
        definition(&[0, 11, 17], &rust("a/Prints#")),
        usage(&[8, 2, 8], &rust("a/Prints#")),
    ];
    occurrences.extend(hostile.iter().map(|symbol| usage(&[4, 20, 24], symbol)));
    let bytes = index("", &[document("src/a.rs", &occurrences)]);
    // Broken symbols count as unmapped, so the share is not the question here.
    let outcome = run_with(&bytes, (&files, &symbols, &[]), 0.0).expect("imports");
    assert_eq!(target(&outcome), [(1, 1, ReferenceKind::TypeUse)]);
}

/// How long the large indexes below may take; the quadratic versions needed minutes.
const LARGE_INDEX_SECONDS: u64 = 20;
/// The size limit for the large indexes.
const LARGE_INDEX_BYTES: u64 = 64 << 20;

/// Imports a large index and returns how long it took.
fn timed(
    bytes: &[u8],
    files: &[SourceFile],
    symbols: &[Symbol],
) -> (codeskimr_scip::ImportOutcome, std::time::Duration) {
    let decoded = codeskimr_scip::decode_index(bytes, LARGE_INDEX_BYTES).expect("decodes");
    let view = codeskimr_scip::SnapshotView {
        files,
        symbols,
        references: &[],
    };
    let started = std::time::Instant::now();
    let outcome = codeskimr_scip::import_references(&decoded, &view, 0.0).expect("imports");
    (outcome, started.elapsed())
}

#[test]
fn many_types_of_one_name_in_a_document_do_not_make_the_import_slow() {
    let (files, symbols) = world();
    let mut occurrences = vec![
        definition(&[0, 11, 17], &rust("a/Prints#")),
        usage(&[8, 2, 8], &rust("a/Prints#")),
        usage(&[4, 20, 24], &rust("b/impl#[Prints]")),
    ];
    // Two hundred thousand different library types that share the name of the one in question.
    occurrences.extend((0..200_000).map(|n| usage(&[9, 1, 7], &rust(&format!("lib{n}/Prints#")))));
    let bytes = index("", &[document("src/a.rs", &occurrences)]);
    let (outcome, took) = timed(&bytes, &files, &symbols);
    assert!(took.as_secs() < LARGE_INDEX_SECONDS, "{took:?}");
    // The impl of `b` is ambiguous (the library types occur too) and maps to nothing.
    assert_eq!(target(&outcome), [(1, 1, ReferenceKind::TypeUse)]);
}

#[test]
fn many_impl_symbols_against_many_candidates_do_not_make_the_import_slow() {
    let files = vec![file(1, "src/a.rs")];
    let types: Vec<Symbol> = (0..3000_u32)
        .map(|n| {
            symbol(
                u64::from(n) + 1,
                1,
                SymbolKind::Struct,
                "Foo",
                ((n, 0), (n, 20)),
            )
        })
        .collect();
    let mut occurrences: Vec<Vec<u8>> = (0..3000_u32)
        .map(|n| definition(&[n, 7, 10], &rust(&format!("m{n}/Foo#"))))
        .collect();
    occurrences.push(usage(&[1, 2, 5], &rust("m0/Foo#")));
    // Fifty thousand blocks whose path fits none of them.
    occurrences
        .extend((0..50_000).map(|n| usage(&[1, 2, 5], &rust(&format!("x/impl#[`zz{n}::Foo`]")))));
    let bytes = index("", &[document("src/a.rs", &occurrences)]);
    let (outcome, took) = timed(&bytes, &files, &types);
    assert!(took.as_secs() < LARGE_INDEX_SECONDS, "{took:?}");
    assert_eq!(target(&outcome), [(2, 1, ReferenceKind::TypeUse)]);
}
