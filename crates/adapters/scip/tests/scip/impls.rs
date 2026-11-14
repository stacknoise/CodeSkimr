//! The impl blocks of rust-analyzer: `Self` and the methods of trait implementations (ADR 0033).

use codeskimr_domain::{Confidence, ReferenceKind, SymbolKind};

use crate::common::{definition, document, file, index, symbol, usage};
use crate::mapping::{run, target};

/// A rust-analyzer symbol of the crate `app`.
fn rust(descriptors: &str) -> String {
    format!("rust-analyzer cargo app 0.1.0 {descriptors}")
}

#[test]
fn self_in_an_impl_block_is_a_use_of_the_type() {
    let files = vec![file(1, "src/a.rs")];
    let symbols = vec![
        symbol(1, 1, SymbolKind::Struct, "Prints", ((0, 0), (10, 1))),
        symbol(2, 1, SymbolKind::Method, "build", ((3, 4), (6, 5))),
    ];
    let ty = rust("model/Prints#");
    let bytes = index(
        "",
        &[document(
            "src/a.rs",
            &[
                definition(&[0, 11, 17], &ty),
                usage(&[4, 20, 24], &rust("model/impl#[Prints]")),
                usage(&[5, 8, 12], &rust("model/impl#[Prints][Display]")),
            ],
        )],
    );
    let outcome = run(&bytes, &files, &symbols, &[]);
    assert_eq!(
        target(&outcome),
        [
            (2, 1, ReferenceKind::TypeUse),
            (2, 1, ReferenceKind::TypeUse)
        ]
    );
    assert!(outcome.references.iter().all(|r| r.verified));
    assert!(
        outcome
            .references
            .iter()
            .all(|r| r.confidence == Confidence::Certain)
    );
    assert_eq!(outcome.stats.mapped, 2);
    assert_eq!(outcome.stats.unmapped, 0);
}

#[test]
fn an_impl_in_another_module_finds_the_type_by_its_unique_name() {
    let files = vec![file(1, "src/a.rs"), file(2, "src/b.rs")];
    let symbols = vec![
        symbol(1, 1, SymbolKind::Struct, "Prints", ((0, 0), (10, 1))),
        symbol(2, 2, SymbolKind::Method, "build", ((3, 4), (6, 5))),
    ];
    let bytes = index(
        "",
        &[
            document("src/a.rs", &[definition(&[0, 11, 17], &rust("a/Prints#"))]),
            document("src/b.rs", &[usage(&[4, 20, 24], &rust("b/impl#[Prints]"))]),
        ],
    );
    let outcome = run(&bytes, &files, &symbols, &[]);
    assert_eq!(target(&outcome), [(2, 1, ReferenceKind::TypeUse)]);
}

#[test]
fn a_type_name_that_two_types_share_is_not_guessed() {
    let files = vec![file(1, "src/a.rs"), file(2, "src/b.rs")];
    let symbols = vec![
        symbol(1, 1, SymbolKind::Struct, "Prints", ((0, 0), (10, 1))),
        symbol(2, 2, SymbolKind::Struct, "Prints", ((0, 0), (10, 1))),
        symbol(3, 2, SymbolKind::Method, "build", ((12, 4), (15, 5))),
    ];
    let bytes = index(
        "",
        &[
            document(
                "src/a.rs",
                &[
                    definition(&[0, 11, 17], &rust("a/Prints#")),
                    usage(&[5, 2, 8], &rust("a/Prints#")),
                ],
            ),
            document(
                "src/b.rs",
                &[
                    definition(&[0, 11, 17], &rust("b/Prints#")),
                    usage(&[13, 20, 24], &rust("c/impl#[Prints]")),
                ],
            ),
        ],
    );
    let outcome = run(&bytes, &files, &symbols, &[]);
    assert_eq!(target(&outcome), [(1, 1, ReferenceKind::TypeUse)]);
    assert_eq!(outcome.stats.unmapped, 0);
}

fn trait_world() -> (
    Vec<codeskimr_domain::SourceFile>,
    Vec<codeskimr_domain::Symbol>,
) {
    let files = vec![file(1, "src/a.rs"), file(2, "src/b.rs")];
    let symbols = vec![
        symbol(1, 1, SymbolKind::Trait, "CodeIndex", ((0, 0), (5, 1))),
        symbol(2, 1, SymbolKind::Method, "layer_edges", ((1, 4), (1, 30))),
        symbol(3, 2, SymbolKind::Struct, "Store", ((0, 0), (20, 1))),
        symbol(4, 2, SymbolKind::Method, "layer_edges", ((3, 4), (6, 5))),
    ];
    (files, symbols)
}

#[test]
fn a_method_of_a_trait_implementation_overrides_the_method_of_the_trait() {
    let (files, symbols) = trait_world();
    let bytes = index(
        "",
        &[
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
                    definition(
                        &[3, 7, 18],
                        &rust("store/impl#[Store][CodeIndex]layer_edges()."),
                    ),
                ],
            ),
        ],
    );
    let outcome = run(&bytes, &files, &symbols, &[]);
    assert_eq!(target(&outcome), [(4, 2, ReferenceKind::Override)]);
    let reference = outcome.references.first().expect("one reference");
    assert!(reference.verified);
    assert_eq!(reference.confidence, Confidence::Certain);
    assert_eq!(reference.file.get(), 2);
}

#[test]
fn a_trait_method_that_the_project_does_not_define_gives_no_override() {
    let (files, symbols) = trait_world();
    let bytes = index(
        "",
        &[document(
            "src/b.rs",
            &[
                definition(&[0, 7, 12], &rust("store/Store#")),
                definition(&[3, 7, 18], &rust("store/impl#[Store][Display]fmt().")),
            ],
        )],
    );
    assert_eq!(target(&run(&bytes, &files, &symbols, &[])), []);
}
