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
