//! The lookup of one symbol behind the `CodeIndex` port.

#[cfg(test)]
mod common;

use codeskimr_application::ports::{CodeIndex, SymbolKey};
use codeskimr_domain::{MetricKind, QualifiedName, SnapshotId, SymbolId, SymbolKind};
use common::{metric, scanned, symbol};

const SNAPSHOT: SnapshotId = SnapshotId::new(10);

#[cfg(test)]
fn store() -> codeskimr_storage::SqliteStore {
    let symbols = vec![
        symbol(21, 11, SymbolKind::Class, "App.Order", None),
        symbol(22, 11, SymbolKind::Method, "App.Order.Total", Some(21)),
        symbol(23, 11, SymbolKind::Field, "App.Order.Id", Some(21)),
    ];
    let metrics = vec![
        metric(21, MetricKind::Loc, 40.0),
        metric(21, MetricKind::MaintainabilityIndex, 70.5),
    ];
    scanned(&[], symbols, vec![], metrics)
}

#[test]
fn finds_a_symbol_by_id_with_members_and_metrics() {
    let found = store()
        .symbol(SNAPSHOT, &SymbolKey::Id(SymbolId::new(21)))
        .unwrap()
        .unwrap();
    assert_eq!(found.symbol.qualified_name.as_str(), "App.Order");
    assert_eq!(found.path.as_str(), "src/order.cs");
    assert_eq!(found.members.len(), 2);
    assert_eq!(found.members[0].name.as_str(), "Total");
    assert_eq!(found.metrics.len(), 2);
}

#[test]
fn finds_a_symbol_by_qualified_name() {
    let key = SymbolKey::Qualified(QualifiedName::new("App.Order.Total").unwrap());
    let found = store().symbol(SNAPSHOT, &key).unwrap().unwrap();
    assert_eq!(found.symbol.id, SymbolId::new(22));
    assert_eq!(found.symbol.parent, Some(SymbolId::new(21)));
    assert!(found.members.is_empty());
}

#[test]
fn an_unknown_symbol_is_none() {
    let key = SymbolKey::Qualified(QualifiedName::new("App.Missing").unwrap());
    assert!(store().symbol(SNAPSHOT, &key).unwrap().is_none());
    assert!(
        store()
            .symbol(SNAPSHOT, &SymbolKey::Id(SymbolId::new(99)))
            .unwrap()
            .is_none()
    );
}
