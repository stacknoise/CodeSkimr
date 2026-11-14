//! Names and places the indexers write in their own way: functions that start behind their name,
//! getters and setters, overloads with one symbol, local functions (ADR 0033).

use codeskimr_domain::{ReferenceKind, SourceFile, Symbol, SymbolId, SymbolKind};

use crate::common::{definition, document, file, index, scip, symbol, usage, varint_field};
use crate::mapping::{run, run_text, target};

fn one_file() -> Vec<SourceFile> {
    vec![file(1, "src/a.ts")]
}

fn inside(mut symbol: Symbol, parent: u64) -> Symbol {
    symbol.parent = Some(SymbolId::new(parent));
    symbol
}

/// `const isObj = (v) => ...` on line 2: the function starts at column 15, behind the name.
fn arrow_world() -> Vec<Symbol> {
    vec![
        symbol(1, 1, SymbolKind::Function, "isObj", ((2, 15), (3, 20))),
        symbol(2, 1, SymbolKind::Function, "use", ((6, 0), (9, 1))),
    ]
}

#[test]
fn a_function_that_starts_behind_its_name_is_found_by_its_line() {
    let name = scip("src/a.ts/isObj.");
    let bytes = index(
        "",
        &[document(
            "src/a.ts",
            &[definition(&[2, 7, 12], &name), usage(&[7, 4, 9], &name)],
        )],
    );
    let outcome = run(&bytes, &one_file(), &arrow_world(), &[]);
    assert_eq!(target(&outcome), [(2, 1, ReferenceKind::Call)]);
}

/// Imports an index whose only definition of `isObj` is at `range`; it must be refused, because
/// nothing maps and so the definition has not been guessed.
fn refused(symbols: &[Symbol], range: [u32; 3]) -> bool {
    let name = scip("src/a.ts/isObj.");
    let bytes = index(
        "",
        &[document(
            "src/a.ts",
            &[definition(&range, &name), usage(&[7, 4, 9], &name)],
        )],
    );
    crate::mapping::run_with(&bytes, (&one_file(), symbols, &[]), 0.5).is_err()
}

#[test]
fn two_symbols_of_that_name_behind_the_definition_are_not_told_apart() {
    let mut symbols = arrow_world();
    symbols.push(symbol(
        3,
        1,
        SymbolKind::Function,
        "isObj",
        ((2, 40), (2, 60)),
    ));
    assert!(refused(&symbols, [2, 7, 12]));
}

#[test]
fn a_function_on_another_line_is_not_taken_for_the_definition() {
    assert!(refused(&arrow_world(), [1, 7, 12]));
}

#[test]
fn getter_and_setter_are_the_property_of_the_project() {
    let symbols = vec![
        symbol(1, 1, SymbolKind::Class, "Grid", ((0, 0), (12, 1))),
        inside(
            symbol(2, 1, SymbolKind::Property, "size", ((2, 2), (4, 3))),
            1,
        ),
        inside(symbol(3, 1, SymbolKind::Method, "use", ((6, 2), (9, 3))), 1),
    ];
    let getter = scip("src/a.ts/Grid#`<get>size`().");
    let setter = scip("src/a.ts/Grid#`<set>size`().");
    let bytes = index(
        "",
        &[document(
            "src/a.ts",
            &[
                definition(&[2, 14, 18], &getter),
                definition(&[3, 14, 18], &setter),
                usage(&[7, 9, 13], &getter),
                usage(&[8, 9, 13], &setter),
            ],
        )],
    );
    let outcome = run(&bytes, &one_file(), &symbols, &[]);
    assert_eq!(
        target(&outcome),
        [(3, 2, ReferenceKind::Read), (3, 2, ReferenceKind::Read)]
    );
}

/// Python: `parse` is declared three times (two overload heads and the implementation) and has
/// one SCIP symbol, defined at the first.
fn overload_world() -> Vec<Symbol> {
    vec![
        symbol(1, 1, SymbolKind::Function, "parse", ((4, 0), (4, 33))),
        symbol(2, 1, SymbolKind::Function, "parse", ((8, 0), (8, 33))),
        symbol(3, 1, SymbolKind::Function, "parse", ((11, 0), (14, 16))),
        symbol(4, 1, SymbolKind::Function, "use", ((17, 0), (18, 19))),
    ]
}

fn overload_bytes(symbol_text: &str, extra: &[Vec<u8>]) -> Vec<u8> {
    let mut all = vec![
        definition(&[4, 4, 9], symbol_text),
        usage(&[8, 4, 9], symbol_text),
        usage(&[11, 4, 9], symbol_text),
        usage(&[18, 11, 16], symbol_text),
    ];
    all.extend_from_slice(extra);
    index("", &[document("src/a.py", &all)])
}

#[test]
fn the_heads_of_overloads_are_declarations_and_the_uses_go_to_every_overload() {
    let files = vec![file(1, "src/a.py")];
    let bytes = overload_bytes(&scip("pkg.m/parse()."), &[]);
    let outcome = run(&bytes, &files, &overload_world(), &[]);
    assert_eq!(
        target(&outcome),
        [
            (4, 1, ReferenceKind::Call),
            (4, 2, ReferenceKind::Call),
            (4, 3, ReferenceKind::Call)
        ]
    );
}

#[test]
fn a_call_inside_the_function_is_a_use_and_not_a_head() {
    let files = vec![file(1, "src/a.py")];
    let text = scip("pkg.m/parse().");
    let bytes = overload_bytes(&text, &[usage(&[12, 15, 20], &text)]);
    let outcome = run(&bytes, &files, &overload_world(), &[]);
    // The recursive call in line 12 comes from the implementation, to all three.
    assert_eq!(
        target(&outcome)
            .iter()
            .filter(|(from, _, _)| *from == 3)
            .count(),
        3
    );
}

#[test]
fn a_function_of_the_same_name_in_another_parent_is_not_an_overload() {
    let files = vec![file(1, "src/a.py")];
    let mut symbols = overload_world();
    symbols[1] = inside(symbols[1].clone(), 4);
    let bytes = overload_bytes(&scip("pkg.m/parse()."), &[]);
    let outcome = run(&bytes, &files, &symbols, &[]);
    // The occurrence in line 8 stays what it is, a use of the first function.
    assert!(target(&outcome).contains(&(2, 1, ReferenceKind::Call)));
    assert!(!target(&outcome).iter().any(|(_, to, _)| *to == 2));
}

/// A local function: `has = (id) => ...` on line 3 of the method `m`, named `name`.
fn local_world(name: &str, column: u32) -> Vec<Symbol> {
    vec![
        symbol(1, 1, SymbolKind::Function, "m", ((1, 0), (8, 1))),
        inside(
            symbol(2, 1, SymbolKind::Function, name, ((3, column), (3, 40))),
            1,
        ),
    ]
}

/// Position encoding of the documents in `local_bytes`: UTF-8 bytes or UTF-16 units.
const BYTES: u64 = 1;
const UNITS: u64 = 2;

/// The lines of `src/a.ts` as the import reads them: `line3` is the line of the definition.
fn text<'a>(definition_line: &'a str, use_line: &'a str) -> Vec<&'a str> {
    let mut rows = vec![""; 9];
    rows[3] = definition_line;
    rows[5] = use_line;
    rows
}

/// The references to the local function (symbol 2) when `defs` are defined and the local
/// symbol 4 is used on line 5 at `at`.
fn local_uses(
    (name, encoding, column): (&str, u64, u32),
    layout: (&str, &str),
    defs: &[Vec<u8>],
    at: [u32; 3],
) -> usize {
    let mut all = defs.to_vec();
    all.push(usage(&at, "local 4"));
    let mut doc = document("src/a.ts", &all);
    doc.extend(varint_field(6, encoding));
    let bytes = index("", &[doc]);
    let rows = text(layout.0, layout.1);
    let symbols = local_world(name, column);
    let outcome = run_text(&bytes, (&one_file(), &symbols, &[]), &rows);
    outcome.map_or(0, |o| {
        o.references.iter().filter(|r| r.to.get() == 2).count()
    })
}

#[test]
fn a_local_function_is_found_by_the_name_in_front_of_it() {
    let defs = [definition(&[3, 8, 11], "local 4")];
    let found = local_uses(
        ("has", BYTES, 14),
        ("        has = (id) => id;", "          has(1)"),
        &defs,
        [5, 10, 13],
    );
    assert_eq!(found, 1);
}

#[test]
fn a_local_name_that_reads_differently_is_not_the_function_even_at_the_same_length() {
    // `{ const key = 1; function has() {} }` on one line: `key` is the last name in front of the
    // function `has`, and has as many characters. Only the text tells them apart.
    let line = "{ const key = 1; function has() {} }";
    let defs = [definition(&[3, 8, 11], "local 4")];
    let layout = (line, "          key + 1");
    let found = local_uses(("has", BYTES, 14), layout, &defs, [5, 10, 13]);
    assert_eq!(found, 0);
}

#[test]
fn non_ascii_names_are_compared_in_bytes_and_by_their_text() {
    // UTF-16 units: `größe` is five units at column 8, seven bytes; the function starts at the
    // parenthesis, byte 18.
    let line3 = "        größe = (id) => id;";
    let line5 = "          größe(1)";
    let defs = [definition(&[3, 8, 13], "local 4")];
    let same = local_uses(("größe", UNITS, 18), (line3, line5), &defs, [5, 10, 15]);
    assert_eq!(same, 1);
    // `gräße` has as many characters, and is another name.
    let other = local_uses(("gräße", UNITS, 18), (line3, line5), &defs, [5, 10, 15]);
    assert_eq!(other, 0);
}

#[test]
fn without_the_text_a_local_symbol_stays_unnamed() {
    let name = "has";
    let defs = [definition(&[3, 8, 11], "local 4")];
    let mut all = defs.to_vec();
    all.push(usage(&[5, 10, 13], "local 4"));
    all.push(definition(&[1, 4, 5], &scip("src/a.ts/m().")));
    all.push(usage(&[7, 1, 2], &scip("src/a.ts/m().")));
    let bytes = index("", &[document("src/a.ts", &all)]);
    let outcome = run(&bytes, &one_file(), &local_world(name, 14), &[]);
    assert!(!target(&outcome).iter().any(|(_, to, _)| *to == 2));
}

#[test]
fn a_name_further_away_or_with_another_definition_in_between_is_not_the_function() {
    let name = ("has", BYTES, 14);
    let layout = ("        has = (id) => id;", "          has(1)");
    // On another line.
    let elsewhere = [definition(&[2, 8, 11], "local 4")];
    assert_eq!(local_uses(name, layout, &elsewhere, [5, 10, 13]), 0);
    // Another one-character definition directly in front of the function.
    let between = [
        definition(&[3, 8, 11], "local 4"),
        definition(&[3, 11, 12], "local 5"),
    ];
    assert_eq!(local_uses(name, layout, &between, [5, 10, 13]), 0);
}

#[test]
fn a_use_at_the_position_of_a_head_of_another_symbol_is_kept() {
    // `use` is used at the very place where `parse` has its second head (line 8, column 4).
    let files = vec![file(1, "src/a.py")];
    let parse = scip("pkg.m/parse().");
    let other = scip("pkg.m/use().");
    let bytes = overload_bytes(
        &parse,
        &[definition(&[17, 4, 7], &other), usage(&[8, 4, 9], &other)],
    );
    let outcome = run(&bytes, &files, &overload_world(), &[]);
    assert!(target(&outcome).contains(&(2, 4, ReferenceKind::Call)));
    assert!(target(&outcome).contains(&(4, 2, ReferenceKind::Call)));
}

#[test]
fn many_local_functions_on_one_line_are_matched_in_time_and_exactly() {
    const COUNT: u32 = 30_000;
    const USED: u32 = 10;
    let width = |n: u32| n * 8;
    let mut symbols = vec![symbol(
        u64::from(COUNT) + 1,
        1,
        SymbolKind::Function,
        "m",
        ((0, 0), (9, 1)),
    )];
    let mut occurrences = Vec::new();
    for i in 0..COUNT {
        let at = width(i);
        symbols.push(symbol(
            u64::from(i) + 1,
            1,
            SymbolKind::Function,
            "a",
            ((3, at + 2), (3, at + 7)),
        ));
        occurrences.push(definition(&[3, at, at + 1], &format!("local {i}")));
    }
    for i in 0..USED {
        occurrences.push(usage(&[5, 2 * i, 2 * i + 1], &format!("local {i}")));
    }
    let mut doc = document("src/a.ts", &occurrences);
    doc.extend(varint_field(6, BYTES));
    let bytes = index("", &[doc]);
    let definitions = "a=()=>1;".repeat(COUNT as usize);
    let uses = "a;".repeat(USED as usize);
    let rows = text(&definitions, &uses);
    let started = std::time::Instant::now();
    let outcome = run_text(&bytes, (&one_file(), &symbols, &[]), &rows).expect("imports");
    assert!(started.elapsed().as_secs() < 20);
    let mut targets: Vec<u64> = outcome.references.iter().map(|r| r.to.get()).collect();
    targets.sort_unstable();
    assert_eq!(targets, (1..=u64::from(USED)).collect::<Vec<_>>());
}
