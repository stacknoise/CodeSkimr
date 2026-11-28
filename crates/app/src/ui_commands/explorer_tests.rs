//! The tree of the explorer: file modules next to the types (a Python package full of modules).

use codeskimr_application::ports::OutlineNode;
use codeskimr_domain::{LanguageId, QualifiedName, SymbolId, SymbolKind, SymbolName};

use super::explorer::build;

fn node(
    id: u64,
    kind: SymbolKind,
    name: &str,
    qualified: &str,
    parent: Option<u64>,
) -> OutlineNode {
    OutlineNode {
        id: SymbolId::new(id),
        name: SymbolName::new(name).unwrap(),
        qualified_name: QualifiedName::new(qualified).unwrap(),
        kind,
        parent: parent.map(SymbolId::new),
        members: 0,
        language: LanguageId::new("python").unwrap(),
        folder: String::new(),
    }
}

/// The node with a number of direct members.
fn counted(node: OutlineNode, members: u32) -> OutlineNode {
    OutlineNode { members, ..node }
}

#[test]
fn a_module_without_types_gets_its_own_namespace_entry() {
    let nodes = [
        node(1, SymbolKind::Module, "__init__", "pkg.__init__", None),
        node(2, SymbolKind::Module, "entry", "pkg.entry", None),
        node(3, SymbolKind::Class, "Entry", "pkg.entry.Entry", Some(2)),
    ];
    let layers = build(&[], &nodes);
    let spaces = &layers[0].namespaces;
    assert_eq!(spaces.len(), 2);
    assert_eq!(spaces[0].name, "pkg.__init__");
    assert_eq!(spaces[0].types.len(), 0);
    assert_eq!(spaces[0].module.as_ref().unwrap().id, 1);
    assert_eq!(spaces[1].types[0].name, "Entry");
    assert_eq!(spaces[1].module.as_ref().unwrap().name, "entry");
}

#[test]
fn the_module_of_a_file_with_real_namespaces_is_not_a_namespace() {
    let nodes = [node(
        1,
        SymbolKind::Module,
        "Calculator",
        "app/Calculator.cs",
        None,
    )];
    assert_eq!(build(&[], &nodes).len(), 0);
}

#[test]
fn a_name_that_only_ends_like_the_module_is_no_namespace_module() {
    let nodes = [node(1, SymbolKind::Module, "entry", "pkg.bentry", None)];
    assert_eq!(build(&[], &nodes).len(), 0);
}

#[test]
fn a_namespace_symbol_with_functions_holds_them_and_one_without_does_not() {
    let nodes = [
        counted(node(1, SymbolKind::Namespace, "util", "app.util", None), 3),
        node(2, SymbolKind::Class, "Tool", "app.util.Tool", Some(1)),
        counted(
            node(3, SymbolKind::Namespace, "empty", "app.empty", None),
            1,
        ),
        node(4, SymbolKind::Class, "Other", "app.empty.Other", Some(3)),
    ];
    let spaces = &build(&[], &nodes)[0].namespaces;
    let util = spaces
        .iter()
        .find(|space| space.name == "app.util")
        .unwrap();
    assert_eq!(util.holders.len(), 1);
    assert_eq!(util.holders[0].members, 2);
    let empty = spaces
        .iter()
        .find(|space| space.name == "app.empty")
        .unwrap();
    assert_eq!(empty.holders.len(), 0);
}

#[test]
fn the_module_of_a_file_without_a_namespace_holds_its_functions() {
    let nodes = [
        counted(node(1, SymbolKind::Module, "Util", "app/Util.kt", None), 2),
        counted(
            node(2, SymbolKind::Module, "Named", "app/Named.kt", None),
            1,
        ),
        node(3, SymbolKind::Namespace, "app", "app", Some(2)),
    ];
    let spaces = &build(&[], &nodes)[0].namespaces;
    let held: Vec<_> = spaces
        .iter()
        .filter(|space| !space.holders.is_empty())
        .collect();
    assert_eq!(held.len(), 1);
    assert_eq!(held[0].name, "app/Util.kt");
    assert_eq!(held[0].holders[0].members, 2);
}

#[test]
fn a_type_in_a_file_without_a_namespace_gets_the_file_as_its_namespace_entry() {
    let nodes = [
        node(1, SymbolKind::Module, "Plain", "app/Plain.cs", None),
        node(2, SymbolKind::Class, "Plain", "Plain", Some(1)),
        node(3, SymbolKind::Module, "entry", "pkg.entry", None),
        node(4, SymbolKind::Class, "Entry", "pkg.entry.Entry", Some(3)),
    ];
    let spaces = &build(&[], &nodes)[0].namespaces;
    let file = spaces
        .iter()
        .find(|space| space.name == "app/Plain.cs")
        .unwrap();
    assert!(file.file);
    let python = spaces
        .iter()
        .find(|space| space.name == "pkg.entry")
        .unwrap();
    assert!(!python.file);
}

fn member(
    id: u64,
    kind: SymbolKind,
    signature: Option<&str>,
) -> codeskimr_application::ports::MemberRow {
    codeskimr_application::ports::MemberRow {
        id: SymbolId::new(id),
        name: SymbolName::new("Work").unwrap(),
        qualified_name: QualifiedName::new("Demo.Box.Work").unwrap(),
        kind,
        visibility: None,
        signature: signature.map(str::to_owned),
        complexity: None,
        usages: 0,
        first_line: 0,
        last_line: 0,
    }
}

#[test]
fn only_an_overloaded_member_names_the_parameters_of_its_signature() {
    use super::symbol::overload_parameters;
    let rows = [
        member(1, SymbolKind::Method, Some("void Work(int x)")),
        member(2, SymbolKind::Method, Some("void Work(int x, string s)")),
        member(3, SymbolKind::Field, Some("int Work")),
    ];
    assert_eq!(overload_parameters(&rows[0], &rows), ["int x"]);
    assert_eq!(overload_parameters(&rows[1], &rows), ["int x", "string s"]);
    // Same name but another kind: no overload.
    assert_eq!(overload_parameters(&rows[2], &rows), Vec::<String>::new());
    // Alone, it names none: its annotations keep the key without parameters.
    assert_eq!(
        overload_parameters(&rows[0], &rows[..1]),
        Vec::<String>::new()
    );
}

#[test]
fn a_type_leaves_out_what_the_reader_can_put_together_again() {
    let nodes = [
        node(1, SymbolKind::Class, "A", "pkg.A", None),
        node(2, SymbolKind::Class, "B", "pkg.B", None),
        OutlineNode {
            language: LanguageId::new("java").unwrap(),
            ..node(3, SymbolKind::Class, "C", "pkg.C", None)
        },
        node(4, SymbolKind::Class, "D", "pkg.sub.D", Some(1)),
    ];
    let space = &build(&[], &nodes)[0].namespaces[0];
    assert_eq!(space.language.as_deref(), Some("python"));
    let kept: Vec<_> = space
        .types
        .iter()
        .map(|node| {
            (
                node.name.as_str(),
                node.qualified_name.is_some(),
                node.language.as_deref(),
            )
        })
        .collect();
    assert_eq!(
        kept,
        [
            ("A", false, None),
            ("B", false, None),
            ("C", false, Some("java"))
        ]
    );
}

#[test]
fn the_answer_leaves_out_empty_lists_and_what_the_module_repeats() {
    let nodes = [node(1, SymbolKind::Module, "util", "pkg.util", None)];
    let space = &build(&[], &nodes)[0].namespaces[0];
    let wire = serde_json::to_value(space).unwrap();
    assert_eq!(wire["name"], "pkg.util");
    assert_eq!(wire["language"], "python");
    for left_out in ["types", "holders", "file"] {
        assert!(wire.get(left_out).is_none(), "{left_out} is sent");
    }
    let module = &wire["module"];
    assert!(module.get("qualified_name").is_none());
    assert!(module.get("language").is_none());
    assert_eq!(module["name"], "util");
}

/// A node of an ABAP file in a folder: a package of an abapGit repository.
fn abap(id: u64, kind: SymbolKind, name: &str, qualified: &str, folder: &str) -> OutlineNode {
    OutlineNode {
        language: LanguageId::new("abap").unwrap(),
        folder: folder.to_owned(),
        ..node(id, kind, name, qualified, None)
    }
}

#[test]
fn the_namespace_of_an_abap_type_is_the_folder_of_its_file() {
    let nodes = [
        abap(1, SymbolKind::Module, "zcl_a.clas", "zcl_a.clas", "src"),
        abap(2, SymbolKind::Class, "zcl_a", "zcl_a", "src"),
        abap(3, SymbolKind::Class, "zcl_b", "zcl_b", "src/sub"),
        abap(4, SymbolKind::Interface, "zif_a", "zif_a", "src"),
    ];
    let spaces = &build(&[], &nodes)[0].namespaces;
    let names: Vec<_> = spaces.iter().map(|space| space.name.as_str()).collect();
    assert_eq!(names, ["src", "src/sub"]);
    let types: Vec<_> = spaces[0]
        .types
        .iter()
        .map(|node| node.name.as_str())
        .collect();
    assert_eq!(types, ["zcl_a", "zif_a"]);
    assert!(
        spaces
            .iter()
            .all(|space| space.module.is_none() && !space.file)
    );
}
