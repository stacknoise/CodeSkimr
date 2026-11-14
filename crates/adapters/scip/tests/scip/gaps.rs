//! Names and places the indexers write in their own way: functions that start behind their name,
//! getters and setters, overloads with one symbol, local functions (ADR 0033).

use codeskimr_domain::{ReferenceKind, SourceFile, Symbol, SymbolId, SymbolKind};

use crate::common::{definition, document, file, index, scip, symbol, usage};
use crate::mapping::{run, target};

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

/// A local function: `const has = (id) => ...` on line 3 of the method `m`.
fn local_world() -> Vec<Symbol> {
    vec![
        symbol(1, 1, SymbolKind::Function, "m", ((1, 0), (8, 1))),
        inside(
            symbol(2, 1, SymbolKind::Function, "has", ((3, 14), (3, 40))),
            1,
        ),
    ]
}

#[test]
fn a_local_function_is_found_by_the_name_in_front_of_it() {
    let bytes = index(
        "",
        &[document(
            "src/a.ts",
            &[
                definition(&[3, 8, 11], "local 4"),
                usage(&[5, 10, 13], "local 4"),
                usage(&[6, 10, 13], "local 4"),
                usage(&[6, 20, 22], "local 7"),
            ],
        )],
    );
    let outcome = run(&bytes, &one_file(), &local_world(), &[]);
    assert_eq!(
        target(&outcome),
        [(1, 2, ReferenceKind::Call), (1, 2, ReferenceKind::Call)]
    );
}

#[test]
fn a_local_name_of_another_length_or_further_away_is_not_the_function() {
    // `ab` in front of the function is not `has`, and a name on another line is not either.
    for defs in [
        vec![definition(&[3, 8, 10], "local 4")],
        vec![definition(&[2, 8, 11], "local 4")],
        vec![
            definition(&[3, 8, 11], "local 4"),
            definition(&[3, 11, 12], "local 5"),
        ],
    ] {
        let mut all = defs;
        all.push(usage(&[5, 10, 13], "local 4"));
        all.push(definition(&[1, 4, 5], &scip("src/a.ts/m().")));
        all.push(usage(&[7, 1, 2], &scip("src/a.ts/m().")));
        let bytes = index("", &[document("src/a.ts", &all)]);
        let outcome = run(&bytes, &one_file(), &local_world(), &[]);
        assert!(
            !target(&outcome).iter().any(|(_, to, _)| *to == 2),
            "{defs_len}",
            defs_len = all.len()
        );
    }
}
