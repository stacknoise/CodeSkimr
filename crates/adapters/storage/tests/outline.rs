//! The explorer tree and the member table behind the `CodeOutline` port.

#[cfg(test)]
mod common;

use codeskimr_application::ports::CodeOutline;
use codeskimr_domain::{MetricKind, SnapshotId, SymbolId, SymbolKind};
use common::{metric, scanned, symbol};

const SNAPSHOT: SnapshotId = SnapshotId::new(10);

#[cfg(test)]
fn store() -> codeskimr_storage::SqliteStore {
    let symbols = vec![
        symbol(20, 11, SymbolKind::Namespace, "App", None),
        symbol(21, 11, SymbolKind::Class, "App.Order", Some(20)),
        symbol(22, 11, SymbolKind::Method, "App.Order.Total", Some(21)),
        symbol(23, 11, SymbolKind::Field, "App.Order.Id", Some(21)),
        symbol(24, 11, SymbolKind::Interface, "App.IOrder", Some(20)),
    ];
    let metrics = vec![metric(22, MetricKind::CyclomaticComplexity, 4.0)];
    scanned(&[], symbols, vec![], metrics)
}

#[test]
fn the_outline_holds_containers_and_types_but_no_members() {
    let nodes = store().outline(SNAPSHOT).unwrap();
    let names: Vec<_> = nodes.iter().map(|n| n.qualified_name.as_str()).collect();
    assert_eq!(names, ["App", "App.IOrder", "App.Order"]);
}

#[test]
fn a_node_knows_its_parent_and_its_member_count() {
    let nodes = store().outline(SNAPSHOT).unwrap();
    let order = nodes.iter().find(|n| n.name.as_str() == "Order").unwrap();
    assert_eq!(order.parent, Some(SymbolId::new(20)));
    assert_eq!(order.members, 2);
}

#[test]
fn member_rows_carry_complexity_and_lines() {
    let rows = store().member_rows(SNAPSHOT, SymbolId::new(21)).unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].name.as_str(), "Total");
    assert_eq!(rows[0].complexity, Some(4.0));
    assert_eq!(rows[1].complexity, None);
    assert_eq!(rows[0].usages, 0);
}

#[test]
fn an_unknown_symbol_has_no_member_rows() {
    let rows = store().member_rows(SNAPSHOT, SymbolId::new(999)).unwrap();
    assert!(rows.is_empty(), "{rows:?}");
}
