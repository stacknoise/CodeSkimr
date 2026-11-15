//! What the text of a Java file adds to the index of scip-java (`docs/level2-import.md`, point
//! 14): import lines are no uses, and the calls of a constructor that javac counts and the index
//! does not write are found by the rules of the language.

use std::sync::Arc;

use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{Excerpt, Imported, ReferenceImporter, SourceText};
use codeskimr_domain::{
    FolderPath, Reference, ReferenceKind, RelativePath, Span, Symbol, SymbolId, SymbolKind,
};
use codeskimr_scip::ScipImporter;

use crate::common::{
    definition, document, file, index, level_one, request, span, symbol, usage, written,
};

/// Answers every request with the same lines.
struct Lines(Vec<String>);

impl SourceText for Lines {
    fn excerpt(
        &self,
        _: &FolderPath,
        _: &RelativePath,
        _: (u32, u32),
        _: (u32, u64),
    ) -> Result<Excerpt, CodedError> {
        Ok(Excerpt {
            lines: self.0.clone(),
            truncated: false,
        })
    }
}

/// A SCIP symbol in the shape of scip-java.
fn java(descriptors: &str) -> String {
    format!("semanticdb maven pkg 1 {descriptors}")
}

fn ctor(of: &str, disambiguator: &str) -> String {
    java(&format!("p/{of}#`<init>`({disambiguator})."))
}

fn class(id: u64, name: &str, ((a, b), (c, d)): ((u32, u32), (u32, u32))) -> Symbol {
    symbol(id, 1, SymbolKind::Class, name, ((a, b), (c, d)))
}

fn constructor(id: u64, parent: u64, name: &str, at: ((u32, u32), (u32, u32))) -> Symbol {
    Symbol {
        parent: Some(SymbolId::new(parent)),
        ..symbol(id, 1, SymbolKind::Constructor, name, at)
    }
}

/// Imports the index over `lines` as the file `src/A.java`; returns what the importer returns.
fn import(
    lines: &[&str],
    symbols: &[Symbol],
    level1: &[Reference],
    documents: &[Vec<u8>],
) -> Imported {
    import_as("src/A.java", lines, symbols, level1, documents)
}

fn import_as(
    name: &str,
    lines: &[&str],
    symbols: &[Symbol],
    level1: &[Reference],
    documents: &[Vec<u8>],
) -> Imported {
    let bytes = index("file:///p", &[document(name, documents)]);
    let (_dir, path) = written(&bytes);
    let files = [file(1, name)];
    let root = FolderPath::new("/work").expect("root");
    let text = Lines(lines.iter().map(|line| (*line).to_owned()).collect());
    ScipImporter::new(Arc::new(text))
        .import(&request(&path, &root, (&files, symbols, level1)))
        .expect("imports")
}

fn calls(imported: &Imported) -> Vec<(u64, u64, Span)> {
    imported
        .references
        .iter()
        .filter(|reference| reference.kind == ReferenceKind::Call)
        .map(|r| (r.from.get(), r.to.get(), r.span))
        .collect()
}

#[test]
fn an_import_line_is_no_use_and_other_lines_are_kept() {
    let lines = [
        "import p.Base;",
        "import static p.Base.make;",
        "class Base {",
        "}",
        "class U { Base b; }",
    ];
    let symbols = [
        class(9, "Top", ((0, 0), (4, 19))),
        class(1, "Base", ((2, 0), (3, 1))),
        class(2, "U", ((4, 0), (4, 19))),
    ];
    let base = java("p/Base#");
    let imported = import(
        &lines,
        &symbols,
        &[],
        &[
            usage(&[0, 9, 13], &base),
            usage(&[1, 21, 25], &base),
            definition(&[2, 6, 10], &base),
            usage(&[4, 10, 14], &base),
        ],
    );
    let spans: Vec<Span> = imported.references.iter().map(|r| r.span).collect();
    assert_eq!(spans, [span((4, 10), (4, 14))]);
}

/// A Java file with its index.
struct World {
    lines: Vec<&'static str>,
    symbols: Vec<Symbol>,
    level1: Vec<Reference>,
    documents: Vec<Vec<u8>>,
}

impl World {
    fn imported(&self) -> Imported {
        self.imported_as("src/A.java")
    }

    fn imported_as(&self, name: &str) -> Imported {
        import_as(
            name,
            &self.lines,
            &self.symbols,
            &self.level1,
            &self.documents,
        )
    }
}

/// `class Base { Base() { } }`, then `class Derived extends Base { }`: the default constructor of
/// `Derived` calls the constructor of `Base`.
fn derived_world() -> World {
    let lines = [
        "class Base {",
        "  Base() { }",
        "}",
        "class Derived extends Base {",
        "}",
    ];
    let symbols = vec![
        class(1, "Base", ((0, 0), (2, 1))),
        constructor(2, 1, "Base", ((1, 2), (1, 12))),
        class(3, "Derived", ((3, 0), (4, 1))),
    ];
    let level1 = vec![level_one(1, (3, 1), 1, (3, 22), ReferenceKind::Inherit)];
    let documents = vec![
        definition(&[0, 6, 10], &java("p/Base#")),
        definition(&[1, 2, 6], &ctor("Base", "")),
        definition(&[3, 6, 13], &java("p/Derived#")),
        definition(&[3, 6, 13], &ctor("Derived", "")),
        usage(&[3, 22, 26], &java("p/Base#")),
    ];
    World {
        lines: lines.to_vec(),
        symbols,
        level1,
        documents,
    }
}

#[test]
fn a_default_constructor_calls_the_constructor_without_arguments_of_the_superclass() {
    let imported = derived_world().imported();
    assert_eq!(calls(&imported), [(3, 2, span((3, 6), (3, 13)))]);
}

#[test]
fn nothing_is_called_where_the_superclass_has_no_constructor_without_arguments() {
    let mut world = derived_world();
    world.lines[1] = "  Base(int a) { }";
    world.symbols[1] = constructor(2, 1, "Base", ((1, 2), (1, 17)));
    let imported = world.imported();
    assert_eq!(calls(&imported), []);
}

#[test]
fn braces_and_commas_inside_an_annotation_do_not_make_a_constructor_without_arguments() {
    let mut world = derived_world();
    world.lines[1] = "  Base(@Tags({1, 2}) int a) { }";
    assert_eq!(calls(&world.imported()), []);
}

#[test]
fn an_import_over_two_lines_is_no_use() {
    let lines = [
        "import p.",
        "    Base;",
        "class U { Base b; }",
        "class Base { }",
    ];
    let symbols = [
        class(9, "Top", ((0, 0), (3, 14))),
        class(2, "U", ((2, 0), (2, 19))),
        class(1, "Base", ((3, 0), (3, 14))),
    ];
    let base = java("p/Base#");
    let imported = import(
        &lines,
        &symbols,
        &[],
        &[
            usage(&[1, 4, 8], &base),
            usage(&[2, 10, 14], &base),
            definition(&[3, 6, 10], &base),
        ],
    );
    let spans: Vec<Span> = imported.references.iter().map(|r| r.span).collect();
    assert_eq!(spans, [span((2, 10), (2, 14))]);
}

#[test]
fn the_only_varargs_constructor_answers_a_call_without_arguments() {
    let mut world = derived_world();
    world.lines[1] = "  Base(String... names) { }";
    let imported = world.imported();
    assert_eq!(calls(&imported), [(3, 2, span((3, 6), (3, 13)))]);
}

/// `Sub extends Base` with the constructor `Sub(int a)` over two lines and the constructor
/// `Sub()` that calls `super()` itself.
#[test]
fn a_declared_constructor_calls_at_its_opening_brace_unless_it_calls_another_itself() {
    let lines = [
        "class Base {",
        "  Base() { }",
        "}",
        "class Sub extends Base {",
        "  Sub(int a,",
        "      String b)",
        "      throws Exception",
        "  {",
        "  }",
        "  Sub() { super(); }",
        "}",
    ];
    let symbols = [
        class(1, "Base", ((0, 0), (2, 1))),
        constructor(2, 1, "Base", ((1, 2), (1, 12))),
        class(3, "Sub", ((3, 0), (10, 1))),
        constructor(4, 3, "Sub", ((4, 2), (8, 3))),
        constructor(5, 3, "Sub", ((9, 2), (9, 20))),
    ];
    let level1 = [level_one(1, (3, 1), 1, (3, 18), ReferenceKind::Inherit)];
    let imported = import(
        &lines,
        &symbols,
        &level1,
        &[
            definition(&[0, 6, 10], &java("p/Base#")),
            definition(&[1, 2, 6], &ctor("Base", "")),
            definition(&[3, 6, 9], &java("p/Sub#")),
            usage(&[3, 18, 22], &java("p/Base#")),
            definition(&[4, 2, 5], &ctor("Sub", "")),
            definition(&[9, 2, 5], &ctor("Sub", "+1")),
            usage(&[9, 9, 14], &ctor("Base", "")),
        ],
    );
    let found = calls(&imported);
    assert!(found.contains(&(4, 2, span((7, 2), (7, 3)))), "{found:?}");
    // The constructor that calls `super()` itself implies nothing, and the explicit call of the
    // index is the one use of `Base()` there.
    assert_eq!(found.iter().filter(|call| call.0 == 5).count(), 1);
}

fn creation_world(second_constructor: bool) -> (Vec<&'static str>, Vec<Symbol>, Vec<Vec<u8>>) {
    let mut lines = vec!["abstract class Base {", "  Base(int a) { }"];
    let mut symbols = vec![
        class(1, "Base", ((0, 0), (3, 1))),
        constructor(2, 1, "Base", ((1, 2), (1, 17))),
    ];
    let mut documents = vec![
        definition(&[0, 15, 19], &java("p/Base#")),
        definition(&[1, 2, 6], &ctor("Base", "")),
    ];
    if second_constructor {
        lines.push("  Base() { }");
        symbols.push(constructor(5, 1, "Base", ((2, 2), (2, 13))));
        documents.push(definition(&[2, 2, 6], &ctor("Base", "+1")));
    }
    lines.extend([
        "}",
        "class Maker {",
        "  Object make() { return new Base<String>(1,",
        "      2) { }; }",
        "}",
    ]);
    let maker = u32::try_from(lines.len() - 4).expect("line");
    let creation = lines[lines.len() - 3];
    let column = u32::try_from(creation.find("Base").expect("type")).expect("column");
    symbols.push(class(3, "Maker", ((maker, 0), (maker + 3, 1))));
    symbols.push(Symbol {
        parent: Some(SymbolId::new(3)),
        ..symbol(
            4,
            1,
            SymbolKind::Method,
            "make",
            ((maker + 1, 2), (maker + 2, 14)),
        )
    });
    documents.push(usage(&[maker + 1, column, column + 4], &java("p/Base#")));
    (lines, symbols, documents)
}

#[test]
fn an_anonymous_subclass_calls_the_only_constructor_of_its_type() {
    let (lines, symbols, documents) = creation_world(false);
    let imported = import(&lines, &symbols, &[], &documents);
    // The call stands on the opening brace, where javac puts it, on the second line here.
    assert_eq!(calls(&imported), [(4, 2, span((5, 9), (5, 10)))]);
}

#[test]
fn an_anonymous_subclass_of_a_type_with_two_constructors_calls_none() {
    let (lines, symbols, documents) = creation_world(true);
    let imported = import(&lines, &symbols, &[], &documents);
    assert_eq!(calls(&imported), []);
}

#[test]
fn new_of_a_class_without_a_constructor_is_a_use_of_the_class() {
    let lines = ["class Plain { }", "class User { Object o = new Plain(); }"];
    let symbols = [
        class(1, "Plain", ((0, 0), (0, 15))),
        class(2, "User", ((1, 0), (1, 38))),
    ];
    let imported = import(
        &lines,
        &symbols,
        &[],
        &[
            definition(&[0, 6, 11], &java("p/Plain#")),
            definition(&[0, 6, 11], &ctor("Plain", "")),
            definition(&[1, 6, 10], &java("p/User#")),
            usage(&[1, 31, 36], &ctor("Plain", "")),
        ],
    );
    let uses: Vec<(u64, u64, ReferenceKind)> = imported
        .references
        .iter()
        .map(|r| (r.from.get(), r.to.get(), r.kind))
        .collect();
    assert_eq!(uses, [(2, 1, ReferenceKind::TypeUse)]);
}

#[test]
fn an_enum_constant_with_a_body_is_a_use_of_its_enum() {
    let lines = ["enum Color {", "  RED() { },", "  BLUE", "}"];
    let enum_symbol = symbol(1, 1, SymbolKind::Enum, "Color", ((0, 0), (3, 1)));
    let member = |id: u64, name: &str, at| Symbol {
        parent: Some(SymbolId::new(1)),
        ..symbol(id, 1, SymbolKind::EnumMember, name, at)
    };
    let symbols = [
        enum_symbol,
        member(2, "RED", ((1, 2), (1, 12))),
        member(3, "BLUE", ((2, 2), (2, 6))),
    ];
    let imported = import(
        &lines,
        &symbols,
        &[],
        &[
            definition(&[0, 5, 10], &java("p/Color#")),
            definition(&[1, 2, 5], &java("p/Color#RED.")),
            definition(&[2, 2, 6], &java("p/Color#BLUE.")),
        ],
    );
    let uses: Vec<(u64, u64, Span)> = imported
        .references
        .iter()
        .map(|r| (r.from.get(), r.to.get(), r.span))
        .collect();
    assert_eq!(uses, [(2, 1, span((1, 2), (1, 5)))]);
}

#[test]
fn files_of_other_languages_keep_their_import_lines_and_get_no_implied_call() {
    // The same index and text, as a TypeScript file: nothing of the Java rules applies.
    let imported = derived_world().imported_as("src/A.ts");
    assert_eq!(calls(&imported), []);
    let lines = ["import p.Base;", "class U { Base b; }", "class Base { }"];
    let symbols = [
        class(9, "Top", ((0, 0), (2, 14))),
        class(2, "U", ((1, 0), (1, 19))),
        class(1, "Base", ((2, 0), (2, 14))),
    ];
    let base = java("p/Base#");
    let documents = [
        usage(&[0, 9, 13], &base),
        usage(&[1, 10, 14], &base),
        definition(&[2, 6, 10], &base),
    ];
    let imported = import_as("src/A.ts", &lines, &symbols, &[], &documents);
    let at: Vec<u32> = imported
        .references
        .iter()
        .map(|r| r.span.start().line)
        .collect();
    assert_eq!(at, [0, 1]);
}

/// `Base(int a)` and a `Maker` whose `make` holds `creation`, a line with one `new Base(...) { }`.
fn maker_world(creation: &str, usages: &[u32]) -> World {
    let lines = vec![
        "class Base {",
        "  Base(int a) { }",
        "}",
        "class Maker {",
        "",
        "}",
    ];
    let mut lines = lines;
    let creation = creation.to_owned();
    let leaked: &'static str = Box::leak(creation.into_boxed_str());
    lines[4] = leaked;
    let width = u32::try_from(leaked.len()).expect("width");
    let mut documents = vec![
        definition(&[0, 6, 10], &java("p/Base#")),
        definition(&[1, 2, 6], &ctor("Base", "")),
    ];
    documents.extend(
        usages
            .iter()
            .map(|at| usage(&[4, *at, at + 4], &java("p/Base#"))),
    );
    World {
        lines,
        symbols: vec![
            class(1, "Base", ((0, 0), (2, 1))),
            constructor(2, 1, "Base", ((1, 2), (1, 17))),
            class(3, "Maker", ((3, 0), (5, 1))),
            Symbol {
                parent: Some(SymbolId::new(3)),
                ..symbol(4, 1, SymbolKind::Method, "make", ((4, 2), (4, width)))
            },
        ],
        level1: Vec::new(),
        documents,
    }
}

#[test]
fn a_text_block_with_brackets_inside_does_not_end_the_arguments() {
    let line = "  Object make() { return new Base(\"\"\"x\"y) {\"\"\") { }; }";
    let at = u32::try_from(line.find("Base(").expect("type")).expect("column");
    let brace = u32::try_from(line.rfind("{ }").expect("body")).expect("column");
    let imported = maker_world(line, &[at]).imported();
    assert_eq!(calls(&imported), [(4, 2, span((4, brace), (4, brace + 1)))]);
}

#[test]
fn a_qualified_super_call_is_an_explicit_call() {
    let mut world = derived_world();
    world.lines = vec![
        "class Base {",
        "  Base() { }",
        "}",
        "class Derived extends Base {",
        "  Derived(Outer o) { o.super(); }",
        "}",
    ];
    world.symbols[2] = class(3, "Derived", ((3, 0), (5, 1)));
    world
        .symbols
        .push(constructor(4, 3, "Derived", ((4, 2), (4, 33))));
    world.documents[3] = definition(&[4, 2, 9], &ctor("Derived", ""));
    assert_eq!(calls(&world.imported()), []);
    // The same constructor with an ordinary first statement does call `super()`.
    world.lines[4] = "  Derived(Outer o) { run(); }";
    assert_eq!(calls(&world.imported()).len(), 1);
}

#[test]
fn a_file_that_needs_too_much_reading_gets_nothing_marked() {
    let depth = 400_usize;
    let creation = format!(
        "  Object make() {{ return {}{} {{ }}; }}",
        "new Base(".repeat(depth),
        ")".repeat(depth)
    );
    let columns: Vec<u32> = creation
        .match_indices("Base(")
        .map(|(at, _)| u32::try_from(at).expect("column"))
        .collect();
    assert_eq!(calls(&maker_world(&creation, &columns).imported()), []);
    // One creation in the same place is marked: it is the size, not the shape, that stops it.
    let one = "  Object make() { return new Base(1) { }; }";
    let at = u32::try_from(one.find("Base(").expect("type")).expect("column");
    assert_eq!(calls(&maker_world(one, &[at]).imported()).len(), 1);
}
