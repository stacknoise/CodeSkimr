//! Mapping of occurrences onto project symbols.

use codeskimr_domain::{Confidence, Reference, ReferenceKind, SourceFile, Symbol, SymbolKind};
use codeskimr_scip::{ImportOutcome, ScipError, SnapshotView, decode_index, import_references};

use crate::common::{
    LIMIT, definition, document, file, index, level_one, run_index, scip, span, symbol, usage,
};

/// Decodes `bytes` and imports them into the snapshot, failing below the share `min_mapped`.
pub fn run_with(
    bytes: &[u8],
    snapshot: (&[SourceFile], &[Symbol], &[Reference]),
    min_mapped: f64,
) -> Result<ImportOutcome, ScipError> {
    let decoded = decode_index(bytes, LIMIT).expect("decodes");
    let (files, symbols, references) = snapshot;
    let view = SnapshotView {
        files,
        symbols,
        references,
    };
    import_references(&decoded, &view, min_mapped)
}

/// Like [`run_with`] with the default share of one half, which must be reached.
pub fn run(
    bytes: &[u8],
    files: &[SourceFile],
    symbols: &[Symbol],
    references: &[Reference],
) -> ImportOutcome {
    run_with(bytes, (files, symbols, references), 0.5).expect("imports")
}

/// Two files: `Svc` with the methods `Run` and `Stop`, and `Caller` with the method `Go`.
pub fn service_world() -> (Vec<SourceFile>, Vec<Symbol>) {
    let files = vec![file(1, "src/a.cs"), file(2, "src/b.cs")];
    let symbols = vec![
        symbol(1, 1, SymbolKind::Class, "Svc", ((1, 0), (12, 1))),
        symbol(2, 1, SymbolKind::Method, "Run", ((2, 4), (5, 5))),
        symbol(3, 1, SymbolKind::Method, "Stop", ((7, 4), (10, 5))),
        symbol(4, 2, SymbolKind::Class, "Caller", ((0, 0), (9, 1))),
        symbol(5, 2, SymbolKind::Method, "Go", ((1, 4), (8, 5))),
    ];
    (files, symbols)
}

fn targets_of(outcome: &ImportOutcome) -> Vec<u64> {
    outcome.references.iter().map(|r| r.to.get()).collect()
}

fn target(outcome: &ImportOutcome) -> Vec<(u64, u64, ReferenceKind)> {
    outcome
        .references
        .iter()
        .map(|r| (r.from.get(), r.to.get(), r.kind))
        .collect()
}

#[test]
fn a_call_becomes_a_certain_verified_reference() {
    let (files, symbols) = service_world();
    let bytes = run_index(&[[3, 12, 15]]);
    let outcome = run(&bytes, &files, &symbols, &[]);
    assert_eq!(target(&outcome), [(5, 2, ReferenceKind::Call)]);
    let reference = outcome.references.first().expect("one reference");
    assert_eq!(reference.confidence, Confidence::Certain);
    assert!(reference.verified);
    assert_eq!(reference.file.get(), 2);
    assert_eq!(reference.span, span((3, 12), (3, 15)));
    assert_eq!(outcome.stats.mapped, 1);
    assert_eq!(outcome.stats.dropped(), 0);
}

#[test]
fn the_definition_occurrence_is_not_a_reference() {
    let (files, symbols) = service_world();
    let bytes = run_index(&[]);
    let outcome = run(&bytes, &files, &symbols, &[]);
    assert_eq!(outcome.references, []);
    assert_eq!(outcome.stats.mapped, 0);
}

#[test]
fn the_kind_of_level_one_at_the_same_place_is_kept() {
    let (files, symbols) = service_world();
    let bytes = run_index(&[[3, 12, 15]]);
    let old = [level_one(7, (5, 3), 2, (3, 12), ReferenceKind::Override)];
    let outcome = run(&bytes, &files, &symbols, &old);
    assert_eq!(target(&outcome), [(5, 2, ReferenceKind::Override)]);
}

#[test]
fn without_level_one_the_kind_follows_the_target() {
    let files = vec![file(1, "src/a.cs")];
    let symbols = vec![
        symbol(1, 1, SymbolKind::Class, "Svc", ((0, 0), (30, 1))),
        symbol(2, 1, SymbolKind::Method, "Run", ((2, 4), (5, 5))),
        symbol(3, 1, SymbolKind::Class, "Config", ((6, 4), (9, 5))),
        symbol(4, 1, SymbolKind::Property, "Name", ((10, 4), (11, 20))),
    ];
    let names = [
        (scip("ns/Svc#Run()."), [2, 8, 11]),
        (scip("ns/Svc#Config#"), [6, 10, 16]),
        (scip("ns/Svc#Name."), [10, 11, 15]),
    ];
    let defs: Vec<_> = names.iter().map(|(s, r)| definition(r, s)).collect();
    let uses: Vec<_> = names
        .iter()
        .zip([[20_u32, 1, 4], [21, 1, 4], [22, 1, 4]])
        .map(|((s, _), r)| usage(&r, s))
        .collect();
    let all: Vec<_> = defs.into_iter().chain(uses).collect();
    let outcome = run(
        &index("", &[document("src/a.cs", &all)]),
        &files,
        &symbols,
        &[],
    );
    assert_eq!(
        target(&outcome),
        [
            (1, 2, ReferenceKind::Call),
            (1, 3, ReferenceKind::TypeUse),
            (1, 4, ReferenceKind::Read),
        ]
    );
}

#[test]
fn overloads_map_by_position() {
    let files = vec![file(1, "src/a.cs"), file(2, "src/b.cs")];
    let symbols = vec![
        symbol(1, 1, SymbolKind::Class, "Guard", ((0, 0), (20, 1))),
        symbol(2, 1, SymbolKind::Method, "Zero", ((2, 4), (5, 5))),
        symbol(3, 1, SymbolKind::Method, "Zero", ((7, 4), (10, 5))),
        symbol(4, 2, SymbolKind::Method, "Use", ((0, 0), (9, 1))),
    ];
    let first = scip("ns/Guard#Zero().");
    let second = scip("ns/Guard#Zero(+1).");
    let bytes = index(
        "",
        &[
            document(
                "src/a.cs",
                &[
                    definition(&[2, 8, 12], &first),
                    definition(&[7, 8, 12], &second),
                ],
            ),
            document(
                "src/b.cs",
                &[usage(&[1, 2, 6], &second), usage(&[2, 2, 6], &first)],
            ),
        ],
    );
    let outcome = run(&bytes, &files, &symbols, &[]);
    assert_eq!(
        target(&outcome),
        [(4, 3, ReferenceKind::Call), (4, 2, ReferenceKind::Call)]
    );
}

#[test]
fn nested_chains_constructors_and_qualified_names_map_by_the_last_descriptor() {
    let files = vec![file(1, "src/a.cs"), file(2, "src/b.cs")];
    let symbols = vec![
        symbol(1, 1, SymbolKind::Class, "A", ((0, 0), (30, 1))),
        symbol(2, 1, SymbolKind::Class, "B", ((1, 4), (20, 5))),
        symbol(3, 1, SymbolKind::Method, "C", ((2, 8), (4, 9))),
        symbol(4, 1, SymbolKind::Constructor, "A", ((6, 4), (8, 5))),
        symbol(5, 1, SymbolKind::Method, "Dispose", ((22, 4), (24, 5))),
        symbol(6, 2, SymbolKind::Method, "Use", ((0, 0), (9, 1))),
    ];
    let chain = scip("ns/A#B#C().");
    let ctor = scip("ns/A#`.ctor`().");
    let dispose = scip("ns/A#`System.IDisposable.Dispose`().");
    let bytes = index(
        "",
        &[
            document(
                "src/a.cs",
                &[
                    definition(&[2, 12, 13], &chain),
                    definition(&[6, 4, 5], &ctor),
                    definition(&[22, 8, 15], &dispose),
                ],
            ),
            document(
                "src/b.cs",
                &[
                    usage(&[1, 0, 1], &chain),
                    usage(&[2, 0, 1], &ctor),
                    usage(&[3, 0, 1], &dispose),
                ],
            ),
        ],
    );
    let outcome = run(&bytes, &files, &symbols, &[]);
    assert_eq!(targets_of(&outcome), [3, 4, 5]);
}

#[test]
fn a_partial_class_maps_to_the_symbol_of_each_file() {
    let files = vec![
        file(1, "src/a.cs"),
        file(2, "src/b.cs"),
        file(3, "src/c.cs"),
    ];
    let symbols = vec![
        symbol(1, 1, SymbolKind::Class, "Foo", ((0, 0), (9, 1))),
        symbol(2, 2, SymbolKind::Class, "Foo", ((0, 0), (9, 1))),
        symbol(3, 3, SymbolKind::Method, "Use", ((0, 0), (9, 1))),
    ];
    let foo = scip("ns/Foo#");
    let bytes = index(
        "",
        &[
            document("src/a.cs", &[definition(&[0, 14, 17], &foo)]),
            document("src/b.cs", &[definition(&[0, 14, 17], &foo)]),
            document("src/c.cs", &[usage(&[4, 8, 11], &foo)]),
        ],
    );
    let outcome = run(&bytes, &files, &symbols, &[]);
    assert_eq!(targets_of(&outcome), [1, 2]);
    assert_eq!(outcome.stats.mapped, 1);
}

#[test]
fn local_and_library_symbols_are_not_references() {
    let (files, symbols) = service_world();
    let library = "scip-dotnet nuget System.Runtime 8.0.0 System/Console#WriteLine().";
    let bytes = index(
        "",
        &[
            document(
                "src/a.cs",
                &[definition(&[2, 16, 19], &scip("ns/Svc#Run()."))],
            ),
            document(
                "src/b.cs",
                &[
                    usage(&[3, 12, 15], "local 4"),
                    usage(&[4, 12, 15], library),
                    usage(&[5, 12, 15], &scip("ns/Svc#Run().(count)")),
                    usage(&[6, 12, 15], &scip("ns/Svc#Run().")),
                ],
            ),
        ],
    );
    let outcome = run_with(&bytes, (&files, &symbols, &[]), 0.0).expect("imports");
    assert_eq!(targets_of(&outcome), [2]);
    assert_eq!(outcome.stats.ignored, 2);
    assert_eq!(outcome.stats.library, 1);
    assert_eq!(outcome.stats.unmapped, 0);
    assert_eq!(outcome.stats.mapped, 1);
}

#[test]
fn columns_are_stored_as_the_index_gives_them() {
    let (files, symbols) = service_world();
    // The index counts UTF-16 units; the line has two characters of two bytes before the
    // call, so level 1 (bytes) starts two columns later. Nothing is converted: the span is the
    // one of the index and the kind of level 1 is not found.
    let bytes = run_index(&[[3, 12, 15]]);
    let old = [level_one(7, (5, 3), 2, (3, 14), ReferenceKind::Override)];
    let outcome = run(&bytes, &files, &symbols, &old);
    assert_eq!(target(&outcome), [(5, 2, ReferenceKind::Call)]);
    let reference = outcome.references.first().expect("one reference");
    assert_eq!(reference.span, span((3, 12), (3, 15)));
}

#[test]
fn an_occurrence_outside_every_symbol_is_unmapped() {
    let (files, symbols) = service_world();
    let bytes = run_index(&[[3, 12, 15], [90, 0, 3]]);
    let outcome = run(&bytes, &files, &symbols, &[]);
    assert_eq!(targets_of(&outcome), [2]);
    assert_eq!(outcome.stats.mapped, 1);
    assert_eq!(outcome.stats.unmapped, 1);
}

#[test]
fn the_smallest_enclosing_symbol_is_the_origin() {
    let (files, symbols) = service_world();
    let stop = scip("ns/Svc#Stop().");
    let bytes = index(
        "",
        &[document(
            "src/a.cs",
            &[definition(&[7, 8, 12], &stop), usage(&[3, 10, 14], &stop)],
        )],
    );
    let outcome = run(&bytes, &files, &symbols, &[]);
    assert_eq!(target(&outcome), [(2, 3, ReferenceKind::Call)]);
}

#[test]
fn among_equal_spans_the_kind_that_fits_the_descriptor_wins() {
    let files = vec![file(1, "src/a.cs")];
    let symbols = vec![
        symbol(1, 1, SymbolKind::Class, "Box", ((0, 0), (30, 1))),
        symbol(2, 1, SymbolKind::Method, "Value", ((2, 4), (3, 20))),
        symbol(3, 1, SymbolKind::Property, "Value", ((2, 4), (3, 20))),
    ];
    let value = scip("ns/Box#Value.");
    let bytes = index(
        "",
        &[document(
            "src/a.cs",
            &[definition(&[2, 10, 15], &value), usage(&[20, 1, 6], &value)],
        )],
    );
    let outcome = run(&bytes, &files, &symbols, &[]);
    assert_eq!(target(&outcome), [(1, 3, ReferenceKind::Read)]);
}

#[test]
fn repeated_occurrences_give_one_reference_and_new_ids_follow_the_old() {
    let (files, symbols) = service_world();
    let bytes = run_index(&[[3, 12, 15], [3, 12, 15]]);
    let other = level_one(40, (1, 1), 1, (0, 0), ReferenceKind::Read);
    let outcome = run(&bytes, &files, &symbols, &[other]);
    assert_eq!(outcome.stats.mapped, 2);
    let ids: Vec<_> = outcome.references.iter().map(|r| r.id.get()).collect();
    assert_eq!(ids, [41]);
}
