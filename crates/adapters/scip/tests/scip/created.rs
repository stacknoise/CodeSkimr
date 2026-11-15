//! Symbols that level 2 creates from what the index defines and level 1 does not know (ADR 0055).

use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_domain::{
    FileId, LanguageId, Reference, SourceFile, Symbol, SymbolId, SymbolKind, SymbolOrigin,
};
use codeskimr_scip::{
    Creation, ImportOutcome, ScipError, SnapshotView, decode_index, import_with_creation,
};

use crate::common::{
    LIMIT, definition_in, document_with, file, index, info, level_one, span, symbol, usage,
};

const STRUCT: u64 = 49;
const ENUM: u64 = 11;
const ENUM_MEMBER: u64 = 12;
const FUNCTION: u64 = 17;

/// A SCIP symbol of the Rust indexer's shape around `descriptors`.
fn rust(descriptors: &str) -> String {
    format!("rust-analyzer cargo app 0.1.0 {descriptors}")
}

fn kinds() -> Vec<(String, SymbolKind)> {
    [
        ("Struct", SymbolKind::Struct),
        ("Enum", SymbolKind::Enum),
        ("EnumMember", SymbolKind::EnumMember),
    ]
    .map(|(name, kind)| (name.to_owned(), kind))
    .to_vec()
}

fn rust_file(id: u64, path: &str) -> SourceFile {
    SourceFile {
        language: LanguageId::new("rust").expect("language"),
        ..file(id, path)
    }
}

/// `src/a.rs` with the module `a` and the function `run`; `src/b.rs` with the module `b` and the
/// function `go`, which uses what the macro calls of `a.rs` generate.
fn world() -> (Vec<SourceFile>, Vec<Symbol>) {
    let files = vec![rust_file(1, "src/a.rs"), rust_file(2, "src/b.rs")];
    let module = |id, file, name: &str| {
        let mut module = symbol(id, file, SymbolKind::Module, name, ((0, 0), (40, 0)));
        module.qualified_name =
            codeskimr_domain::QualifiedName::new(format!("app::{name}")).expect("name");
        module
    };
    let symbols = vec![
        module(1, 1, "a"),
        symbol(2, 1, SymbolKind::Function, "run", ((20, 0), (25, 1))),
        module(3, 2, "b"),
        symbol(4, 2, SymbolKind::Function, "go", ((2, 0), (9, 1))),
    ];
    (files, symbols)
}

/// The index: `id_type!(JobId);` on line 3 and `labeled_enum! { Level { Low, High } }` on lines
/// 5 to 9 of `a.rs`, uses of them in `go` of `b.rs` and in `run` of `a.rs`.
fn generated_index() -> Vec<u8> {
    let job = rust("a/JobId#");
    let level = rust("a/Level#");
    let low = rust("a/Level#Low#");
    let high = rust("a/Level#High#");
    let a = document_with(
        "src/a.rs",
        &[
            definition_in(&[3, 9, 14], &[3, 0, 3, 16], &job),
            definition_in(&[6, 4, 9], &[5, 0, 9, 1], &level),
            definition_in(&[7, 8, 11], &[5, 0, 9, 1], &low),
            definition_in(&[8, 8, 12], &[5, 0, 9, 1], &high),
            usage(&[21, 4, 9], &level),
        ],
        &[
            info(
                &job,
                STRUCT,
                "JobId",
                "pub struct JobId(u64)",
                "Identifies a job.",
            ),
            info(&level, ENUM, "Level", "pub enum Level", "How loud."),
            info(&low, ENUM_MEMBER, "Low", "Low", "Quiet."),
            info(&high, ENUM_MEMBER, "High", "High", ""),
        ],
    );
    let b = document_with(
        "src/b.rs",
        &[
            usage(&[3, 10, 15], &job),
            usage(&[4, 10, 13], &low),
            usage(&[5, 10, 14], &high),
        ],
        &[],
    );
    index("file:///p", &[a, b])
}

fn run_created(
    bytes: &[u8],
    symbols: &[Symbol],
    references: &[Reference],
    creation: &Creation<'_>,
) -> Result<ImportOutcome, ScipError> {
    let (files, _) = world();
    let decoded = decode_index(bytes, LIMIT).expect("decodes");
    let view = SnapshotView {
        files: &files,
        symbols,
        references,
    };
    import_with_creation(&decoded, &view, 0.0, creation)
}

fn creation(kinds: &[(String, SymbolKind)]) -> Creation<'_> {
    Creation {
        kinds,
        separator: "::",
        max_symbols: 100,
    }
}

fn named<'a>(outcome: &'a ImportOutcome, name: &str) -> &'a Symbol {
    outcome
        .symbols
        .iter()
        .find(|symbol| symbol.name.as_str() == name)
        .expect("created")
}

#[test]
fn a_generated_struct_enum_and_variants_become_symbols_of_origin_index() {
    let (_, symbols) = world();
    let kinds = kinds();
    let outcome = run_created(&generated_index(), &symbols, &[], &creation(&kinds)).expect("ok");
    assert_eq!(outcome.symbols.len(), 4);
    let job = named(&outcome, "JobId");
    assert_eq!(job.kind, SymbolKind::Struct);
    assert_eq!(job.origin, SymbolOrigin::Index);
    assert_eq!(job.qualified_name.as_str(), "app::a::JobId");
    assert_eq!(job.parent, Some(SymbolId::new(1)), "the module of the file");
    assert_eq!(job.file, FileId::new(1));
    assert_eq!(job.span, span((3, 0), (3, 16)), "the whole macro call");
    assert_eq!(job.signature.as_deref(), Some("pub struct JobId(u64)"));
    assert_eq!(job.doc.as_deref(), Some("Identifies a job."));
    assert_eq!((job.visibility, job.is_static), (None, false));
}

#[test]
fn the_variants_of_a_generated_enum_belong_to_the_created_enum() {
    let (_, symbols) = world();
    let kinds = kinds();
    let outcome = run_created(&generated_index(), &symbols, &[], &creation(&kinds)).expect("ok");
    let level = named(&outcome, "Level");
    assert_eq!(level.kind, SymbolKind::Enum);
    let low = named(&outcome, "Low");
    assert_eq!(low.kind, SymbolKind::EnumMember);
    assert_eq!(
        low.parent,
        Some(level.id),
        "variants belong to the created enum"
    );
    assert_eq!(low.qualified_name.as_str(), "app::a::Level::Low");
    assert_eq!(named(&outcome, "High").doc, None);
}

#[test]
fn ids_are_consecutive_after_the_highest_id_of_the_snapshot() {
    let (_, symbols) = world();
    let kinds = kinds();
    let outcome = run_created(&generated_index(), &symbols, &[], &creation(&kinds)).expect("ok");
    let ids: Vec<u64> = outcome.symbols.iter().map(|s| s.id.get()).collect();
    assert_eq!(ids, [5, 6, 7, 8]);
}

#[test]
fn uses_of_created_symbols_become_verified_references() {
    let (_, symbols) = world();
    let kinds = kinds();
    let outcome = run_created(&generated_index(), &symbols, &[], &creation(&kinds)).expect("ok");
    let job = named(&outcome, "JobId").id;
    let low = named(&outcome, "Low").id;
    let level = named(&outcome, "Level").id;
    let mut found: Vec<(u64, u64)> = outcome
        .references
        .iter()
        .map(|r| (r.from.get(), r.to.get()))
        .collect();
    found.sort_unstable();
    let high = named(&outcome, "High").id;
    let mut expected = vec![
        (4, job.get()),
        (4, low.get()),
        (4, high.get()),
        (2, level.get()),
    ];
    expected.sort_unstable();
    assert_eq!(found, expected);
    assert!(outcome.references.iter().all(|r| r.verified));
    assert_eq!(outcome.stats.unmapped, 0, "nothing is dropped any more");
}

#[test]
fn without_a_kind_table_nothing_is_created_and_the_uses_stay_unmapped() {
    let (_, symbols) = world();
    // The uses stay unmapped, as before ADR 0055; here not one of them maps, so the index is refused.
    let error =
        run_created(&generated_index(), &symbols, &[], &Creation::off()).expect_err("fails");
    assert!(matches!(
        error,
        ScipError::MappingTooLow {
            mapped: 0,
            total: 4
        }
    ));
}

#[test]
fn a_kind_the_pack_does_not_name_creates_nothing() {
    let (_, symbols) = world();
    let kinds = vec![("Struct".to_owned(), SymbolKind::Struct)];
    let outcome = run_created(&generated_index(), &symbols, &[], &creation(&kinds)).expect("ok");
    let names: Vec<&str> = outcome.symbols.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, ["JobId"]);
}

/// `src/a.rs` with the definition of `symbol` and this `information` about it.
fn defining(symbol: &str, information: Vec<u8>) -> Vec<u8> {
    document_with(
        "src/a.rs",
        &[definition_in(&[3, 9, 14], &[3, 0, 3, 16], symbol)],
        &[information],
    )
}

#[test]
fn a_display_name_that_differs_from_the_descriptor_creates_nothing() {
    let (_, symbols) = world();
    let job = rust("a/JobId#");
    let a = defining(
        &job,
        info(&job, STRUCT, "Other", "pub struct JobId(u64)", ""),
    );
    let b = document_with("src/b.rs", &[usage(&[3, 10, 15], &job)], &[]);
    let bytes = index("file:///p", &[a, b]);
    let kinds = kinds();
    let error = run_created(&bytes, &symbols, &[], &creation(&kinds)).expect_err("nothing maps");
    assert!(matches!(
        error,
        ScipError::MappingTooLow {
            mapped: 0,
            total: 1
        }
    ));
}

#[test]
fn a_kind_without_information_creates_nothing() {
    let (_, symbols) = world();
    let job = rust("a/JobId#");
    let a = defining(&job, info(&job, FUNCTION, "JobId", "", ""));
    let bytes = index("file:///p", &[a]);
    let kinds = kinds();
    let outcome = run_created(&bytes, &symbols, &[], &creation(&kinds)).expect("ok");
    assert_eq!(outcome.symbols, []);
}

#[test]
fn a_definition_that_level_one_knows_creates_nothing() {
    let (_, mut symbols) = world();
    symbols.push(symbol(5, 1, SymbolKind::Struct, "JobId", ((3, 0), (3, 16))));
    let kinds = kinds();
    let outcome = run_created(&generated_index(), &symbols, &[], &creation(&kinds)).expect("ok");
    let names: Vec<&str> = outcome.symbols.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, ["Level", "Low", "High"]);
    assert!(
        outcome.references.iter().any(|r| r.to.get() == 5),
        "the uses of the known struct map to it"
    );
}

#[test]
fn an_index_that_wants_too_many_symbols_fails_as_a_whole() {
    let (_, symbols) = world();
    let kinds = kinds();
    let limited = Creation {
        max_symbols: 3,
        ..creation(&kinds)
    };
    let error = run_created(&generated_index(), &symbols, &[], &limited).expect_err("too many");
    assert!(matches!(
        error,
        ScipError::TooManyCreatedSymbols {
            wanted: 4,
            limit: 3
        }
    ));
    assert_eq!(error.code(), ErrorCode::ScipTooManyCreatedSymbols);
    let exact = Creation {
        max_symbols: 4,
        ..creation(&kinds)
    };
    assert!(run_created(&generated_index(), &symbols, &[], &exact).is_ok());
}

#[test]
fn symbols_of_an_earlier_import_are_not_taken_for_level_one() {
    let (_, symbols) = world();
    let kinds = kinds();
    let first = run_created(&generated_index(), &symbols, &[], &creation(&kinds)).expect("ok");
    let mut again = symbols.clone();
    again.extend(first.symbols.clone());
    let second = run_created(
        &generated_index(),
        &again,
        &first.references,
        &creation(&kinds),
    )
    .expect("ok");
    let shape = |outcome: &ImportOutcome| -> Vec<(String, String, Option<String>)> {
        outcome
            .symbols
            .iter()
            .map(|s| {
                (
                    s.name.to_string(),
                    s.qualified_name.to_string(),
                    s.signature.clone(),
                )
            })
            .collect()
    };
    assert_eq!(shape(&first), shape(&second));
    assert_eq!(second.references.len(), first.references.len());
    let ids: Vec<u64> = second.symbols.iter().map(|s| s.id.get()).collect();
    assert_eq!(
        ids,
        [9, 10, 11, 12],
        "after the highest id of all stored symbols"
    );
}

#[test]
fn level_one_symbols_and_references_are_untouched() {
    let (_, symbols) = world();
    let kinds = kinds();
    let old = [level_one(
        50,
        (4, 2),
        2,
        (4, 10),
        codeskimr_domain::ReferenceKind::Read,
    )];
    let outcome = run_created(&generated_index(), &symbols, &old, &creation(&kinds)).expect("ok");
    assert!(
        outcome
            .symbols
            .iter()
            .all(|s| s.origin == SymbolOrigin::Index),
        "only symbols of origin index are returned"
    );
    let low = named(&outcome, "Low").id;
    let kept = outcome
        .references
        .iter()
        .find(|r| r.to == low)
        .expect("use of Low");
    assert_eq!(
        kept.kind,
        codeskimr_domain::ReferenceKind::Read,
        "the kind of level 1 at the same place is kept"
    );
}
