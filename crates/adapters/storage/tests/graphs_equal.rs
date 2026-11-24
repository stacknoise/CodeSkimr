//! The namespace graph and the type focus read from the database are the ones computed over all
//! symbols and references of the snapshot in memory.

#[cfg(test)]
mod common;

#[cfg(test)]
mod tests {
    use codeskimr_application::ports::TypeNeighbour;
    use codeskimr_domain::{
        Confidence, NamespaceGraph, Reference, ReferenceKind, SnapshotId, Symbol, SymbolId,
        SymbolKind, TypeLink, namespace_graph, type_neighbours,
    };
    use codeskimr_storage::ProjectDb;

    use super::common::{reference, symbol, three_snapshots};

    const KINDS: [SymbolKind; 3] = [SymbolKind::Namespace, SymbolKind::Class, SymbolKind::Method];

    /// A symbol of the kind at `position` in `KINDS`: namespace, class, method.
    fn sym(id: u64, file: u64, kind: usize, name: &str, parent: Option<u64>) -> Symbol {
        symbol(id, file, KINDS[kind], name, parent)
    }

    fn shaped(base: Reference, kind: ReferenceKind, confidence: Confidence) -> Reference {
        Reference {
            kind,
            confidence,
            ..base
        }
    }

    fn first_symbols() -> Vec<Symbol> {
        vec![
            sym(21, 11, 0, "App.Shop", None),
            sym(22, 11, 1, "App.Shop.Cart", Some(21)),
            sym(23, 11, 2, "App.Shop.Cart.Total", Some(22)),
            sym(24, 11, 0, "App.Data", None),
            sym(25, 11, 1, "App.Data.Price", Some(24)),
            sym(26, 11, 2, "App.Data.Price.Of", Some(25)),
            sym(27, 12, 1, "App.Shop.View", Some(21)),
            sym(28, 12, 2, "App.Shop.View.Draw", Some(27)),
            sym(29, 12, 1, "App.Data.Store", Some(24)),
        ]
    }

    fn second_symbols() -> Vec<Symbol> {
        vec![
            sym(41, 13, 0, "App.Api", None),
            sym(42, 13, 1, "App.Api.Handler", Some(41)),
            sym(43, 13, 2, "App.Api.Handler.Run", Some(42)),
        ]
    }

    fn first_references() -> Vec<Reference> {
        let call = |id, file, from, to| reference(id, file, from, to);
        let probable = |id, file, from, to| {
            shaped(
                call(id, file, from, to),
                ReferenceKind::Call,
                Confidence::Probable,
            )
        };
        let in_test = |id, file, from, to| {
            shaped(
                call(id, file, from, to),
                ReferenceKind::Test,
                Confidence::Certain,
            )
        };
        vec![
            call(101, 11, 23, 26),
            call(102, 11, 23, 26),
            in_test(103, 11, 23, 26),
            call(104, 12, 28, 23),
            probable(105, 12, 28, 26),
            call(106, 12, 26, 28),
            call(107, 12, 28, 29),
        ]
    }

    fn second_references() -> Vec<Reference> {
        vec![
            reference(108, 13, 43, 23),
            reference(109, 13, 43, 26),
            reference(110, 13, 43, 43),
        ]
    }

    fn all_symbols(snapshot: u64) -> Vec<Symbol> {
        match snapshot {
            10 => first_symbols(),
            20 => [first_symbols(), second_symbols()].concat(),
            _ => vec![
                sym(301, 300, 0, "Other", None),
                sym(302, 300, 1, "Other.Thing", Some(301)),
            ],
        }
    }

    fn all_references(snapshot: u64) -> Vec<Reference> {
        match snapshot {
            10 => first_references(),
            20 => [first_references(), second_references()].concat(),
            _ => vec![reference(111, 300, 302, 302)],
        }
    }

    fn two_projects() -> ProjectDb {
        three_snapshots(
            (first_symbols(), first_references()),
            (second_symbols(), second_references()),
            (all_symbols(30), all_references(30)),
        )
    }

    #[test]
    fn the_namespace_graph_equals_the_one_over_all_symbols_and_references() {
        let store = two_projects();
        for snapshot in [10, 20, 30] {
            let symbols = all_symbols(snapshot);
            let wanted = namespace_graph(
                symbols
                    .iter()
                    .map(|found| (found.id, found.kind, &found.qualified_name)),
                all_references(snapshot)
                    .iter()
                    .filter(|r| {
                        r.kind != ReferenceKind::Test && r.confidence == Confidence::Certain
                    })
                    .map(|r| (r.from, r.to)),
            );
            assert_ne!(wanted, NamespaceGraph::default(), "snapshot {snapshot}");
            let got = store.namespace_graph(SnapshotId::new(snapshot)).unwrap();
            assert_eq!(got, wanted, "snapshot {snapshot}");
        }
    }

    #[test]
    fn the_type_focus_equals_the_one_over_all_symbols_and_references() {
        let store = two_projects();
        for snapshot in [10, 20, 30] {
            let symbols = all_symbols(snapshot);
            let references = all_references(snapshot);
            for id in (21..=43).chain([301, 302, 999]) {
                let wanted = type_neighbours(
                    SymbolId::new(id),
                    symbols
                        .iter()
                        .map(|s| (s.id, s.kind, &s.qualified_name, s.parent)),
                    references.iter().map(|r| (r.from, r.to)),
                );
                let got = store
                    .type_focus(SnapshotId::new(snapshot), SymbolId::new(id))
                    .unwrap();
                let got = got.map(|focus| {
                    let links = |list: &[TypeNeighbour]| {
                        list.iter()
                            .map(|n| (n.label.id, n.count))
                            .collect::<Vec<_>>()
                    };
                    (links(&focus.users), links(&focus.used))
                });
                let wanted = wanted.map(|found| {
                    let links = |list: &[TypeLink]| {
                        list.iter().map(|l| (l.symbol, l.count)).collect::<Vec<_>>()
                    };
                    (links(&found.users), links(&found.used))
                });
                assert_eq!(got, wanted, "snapshot {snapshot} symbol {id}");
            }
        }
    }
}
