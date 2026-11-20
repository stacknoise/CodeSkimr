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
    with_members(id, kind, name, qualified, parent, 0)
}

fn with_members(
    id: u64,
    kind: SymbolKind,
    name: &str,
    qualified: &str,
    parent: Option<u64>,
    members: u32,
) -> OutlineNode {
    OutlineNode {
        id: SymbolId::new(id),
        name: SymbolName::new(name).unwrap(),
        qualified_name: QualifiedName::new(qualified).unwrap(),
        kind,
        parent: parent.map(SymbolId::new),
        members,
        language: LanguageId::new("python").unwrap(),
    }
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
        with_members(1, SymbolKind::Namespace, "util", "app.util", None, 3),
        node(2, SymbolKind::Class, "Tool", "app.util.Tool", Some(1)),
        with_members(3, SymbolKind::Namespace, "empty", "app.empty", None, 1),
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
        with_members(1, SymbolKind::Module, "Util", "app/Util.kt", None, 2),
        with_members(2, SymbolKind::Module, "Named", "app/Named.kt", None, 1),
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
