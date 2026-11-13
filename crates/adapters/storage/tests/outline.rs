//! The explorer tree and the member table behind the `CodeOutline` port.

#[cfg(test)]
mod common;

use codeskimr_application::ports::CodeOutline;
use codeskimr_domain::{MetricKind, SnapshotId, SymbolId, SymbolKind};
use common::{metric, scanned, symbol};

const SNAPSHOT: SnapshotId = SnapshotId::new(10);

#[cfg(test)]
fn store() -> codeskimr_storage::SqliteStore {
    let namespace = symbol(40, 11, SymbolKind::Namespace, "Shop", None);
    let cart = symbol(41, 11, SymbolKind::Class, "Shop.Cart", Some(40));
    let add = symbol(42, 11, SymbolKind::Method, "Shop.Cart.Add", Some(41));
    let id = symbol(43, 11, SymbolKind::Field, "Shop.Cart.Id", Some(41));
    let item = symbol(44, 11, SymbolKind::Interface, "Shop.IItem", Some(40));
    let measured = vec![metric(42, MetricKind::CyclomaticComplexity, 4.0)];
    scanned(&[], vec![namespace, cart, add, id, item], vec![], measured)
}

#[test]
fn the_outline_holds_containers_and_types_but_no_members() {
    let nodes = store().outline(SNAPSHOT).unwrap();
    let names: Vec<_> = nodes.iter().map(|n| n.qualified_name.as_str()).collect();
    assert_eq!(names, ["Shop", "Shop.Cart", "Shop.IItem"]);
}

#[test]
fn a_node_knows_its_parent_and_its_member_count() {
    let nodes = store().outline(SNAPSHOT).unwrap();
    let cart = nodes.iter().find(|n| n.name.as_str() == "Cart").unwrap();
    assert_eq!(cart.parent, Some(SymbolId::new(40)));
    assert_eq!(cart.members, 2);
}

#[test]
fn member_rows_carry_complexity_and_lines() {
    let rows = store().member_rows(SNAPSHOT, SymbolId::new(41)).unwrap();
    assert_eq!(rows.len(), 2);
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
