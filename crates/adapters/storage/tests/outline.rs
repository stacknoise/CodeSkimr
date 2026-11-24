//! The explorer tree and the member table behind the `CodeOutline` port.

#[cfg(test)]
mod common;

use codeskimr_application::ports::CodeOutline;
use codeskimr_domain::{MetricKind, SnapshotId, SymbolId, SymbolKind};
use common::{metric, scanned, symbol};

const SNAPSHOT: SnapshotId = SnapshotId::new(10);

#[cfg(test)]
fn store() -> codeskimr_storage::ProjectDb {
    let namespace = symbol(40, 11, SymbolKind::Namespace, "Shop", None);
    let cart = symbol(41, 11, SymbolKind::Class, "Shop.Cart", Some(40));
    let add = symbol(42, 11, SymbolKind::Method, "Shop.Cart.Add", Some(41));
    let id = symbol(43, 11, SymbolKind::Field, "Shop.Cart.Id", Some(41));
    let item = symbol(44, 11, SymbolKind::Interface, "Shop.IItem", Some(40));
    let line = symbol(45, 11, SymbolKind::Class, "Shop.Cart.Line", Some(41));
    let measured = vec![metric(42, MetricKind::CyclomaticComplexity, 4.0)];
    scanned(
        &[],
        vec![namespace, cart, add, id, item, line],
        vec![],
        measured,
    )
}

#[test]
fn the_outline_holds_containers_and_types_but_no_members() {
    let nodes = store().outline(SNAPSHOT).unwrap();
    let names: Vec<_> = nodes.iter().map(|n| n.qualified_name.as_str()).collect();
    assert_eq!(names, ["Shop", "Shop.Cart", "Shop.Cart.Line", "Shop.IItem"]);
}

/// The symbols of all projects share one table: those of another project are not in the tree, and
/// the tree is read through the files of its snapshot (the plan is checked next to the store).
#[test]
fn the_outline_is_the_same_next_to_another_project() {
    use codeskimr_application::ports::SnapshotStore;
    use codeskimr_domain::{ProjectId, ScanMode};
    use common::{data, file, header, project};

    let store = store();
    store.save_project(&project(2, "/work/other", &[])).unwrap();
    let mut head = header(20, ScanMode::Full);
    head.project = ProjectId::new(2);
    let mut scan = data(head);
    scan.files = vec![file(20, 2001, "src/other.cs", "h")];
    scan.symbols = vec![
        symbol(2100, 2001, SymbolKind::Namespace, "Other", None),
        symbol(2101, 2001, SymbolKind::Class, "Other.Thing", Some(2100)),
    ];
    store.persist(scan).unwrap();
    let nodes = store.outline(SNAPSHOT).unwrap();
    let names: Vec<_> = nodes.iter().map(|n| n.qualified_name.as_str()).collect();
    assert_eq!(names, ["Shop", "Shop.Cart", "Shop.Cart.Line", "Shop.IItem"]);
    let other = store.outline(SnapshotId::new(20)).unwrap();
    assert_eq!(other.len(), 2);
}

#[test]
fn a_node_names_the_language_of_its_file() {
    let nodes = store().outline(SNAPSHOT).unwrap();
    assert!(nodes.iter().all(|node| node.language.as_str() == "csharp"));
}

#[test]
fn a_node_knows_its_parent_and_its_member_count() {
    let nodes = store().outline(SNAPSHOT).unwrap();
    let cart = nodes.iter().find(|n| n.name.as_str() == "Cart").unwrap();
    assert_eq!(cart.parent, Some(SymbolId::new(40)));
    assert_eq!(cart.members, 3);
}

#[test]
fn member_rows_carry_complexity_and_lines() {
    let rows = store().member_rows(SNAPSHOT, SymbolId::new(41)).unwrap();
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[2].name.as_str(), "Line");
    assert_eq!(rows[2].kind, SymbolKind::Class);
    assert_eq!(rows[0].name.as_str(), "Add");
    assert_eq!(rows[0].complexity, Some(4.0));
    assert_eq!(rows[1].complexity, None);
    assert_eq!(rows[0].usages, 0);
}

#[test]
fn an_unknown_symbol_has_no_member_rows() {
    let rows = store().member_rows(SNAPSHOT, SymbolId::new(999)).unwrap();
    assert!(rows.is_empty(), "{rows:?}");
}
