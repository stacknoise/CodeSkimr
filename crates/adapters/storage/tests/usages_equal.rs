//! The answers read from the database are the answers of the walk over all references in memory.

#[cfg(test)]
mod common;

#[cfg(test)]
mod tests {
    use codeskimr_application::ports::UsageIndex;
    use codeskimr_domain::{
        Confidence, Direction, Reference, ReferenceKind, ReverseIndex, SnapshotId, SymbolId,
        SymbolKind, UsageQuery,
    };
    use codeskimr_storage::ProjectDb;

    use super::common::{reference, symbol, three_snapshots};

    const LIMIT: u32 = 1_000;
    const SYMBOLS: [u64; 10] = [21, 22, 23, 24, 31, 32, 41, 42, 301, 99];

    fn shaped(
        base: Reference,
        kind: ReferenceKind,
        confidence: Confidence,
        verified: bool,
    ) -> Reference {
        Reference {
            kind,
            confidence,
            verified,
            ..base
        }
    }

    fn call(id: u64, file: u64, from: u64, to: u64) -> Reference {
        reference(id, file, from, to)
    }

    fn probable(id: u64, file: u64, from: u64, to: u64) -> Reference {
        shaped(
            call(id, file, from, to),
            ReferenceKind::Call,
            Confidence::Probable,
            false,
        )
    }

    fn checked(id: u64, file: u64, from: u64, to: u64) -> Reference {
        shaped(
            call(id, file, from, to),
            ReferenceKind::Call,
            Confidence::Certain,
            true,
        )
    }

    fn in_test(id: u64, file: u64, from: u64, to: u64) -> Reference {
        shaped(
            call(id, file, from, to),
            ReferenceKind::Test,
            Confidence::Certain,
            false,
        )
    }

    /// The references of the first snapshot: a cycle, a self reference, a test reference, a
    /// probable one and a verified one, in two files.
    fn first_scan() -> Vec<Reference> {
        vec![
            call(101, 11, 21, 22),
            call(102, 11, 22, 23),
            call(103, 11, 23, 21),
            in_test(104, 11, 24, 22),
            probable(105, 12, 31, 22),
            checked(106, 12, 32, 22),
            call(107, 12, 22, 22),
            call(108, 12, 31, 32),
        ]
    }

    /// The references the incremental scan adds in its new file; the first two files are carried.
    fn second_scan() -> Vec<Reference> {
        vec![
            call(109, 13, 41, 21),
            checked(110, 13, 42, 41),
            in_test(111, 13, 41, 24),
            probable(112, 13, 42, 23),
        ]
    }

    fn two_projects() -> ProjectDb {
        let symbols = |names: &[(u64, u64, &str)]| -> Vec<_> {
            names
                .iter()
                .map(|(id, file, name)| symbol(*id, *file, SymbolKind::Method, name, None))
                .collect()
        };
        three_snapshots(
            (
                symbols(&[
                    (21, 11, "App.A.One"),
                    (22, 11, "App.A.Two"),
                    (23, 11, "App.A.Three"),
                    (24, 11, "App.Tests.Check"),
                    (31, 12, "App.B.One"),
                    (32, 12, "App.B.Two"),
                ]),
                first_scan(),
            ),
            (
                symbols(&[(41, 13, "App.C.One"), (42, 13, "App.C.Two")]),
                second_scan(),
            ),
            (
                symbols(&[(301, 300, "Other.One")]),
                vec![call(113, 300, 301, 301)],
            ),
        )
    }

    /// The references each snapshot holds: the incremental one carries the first two files, and
    /// the other project points at a symbol it does not hold.
    fn visible(snapshot: u64) -> Vec<Reference> {
        match snapshot {
            10 => first_scan(),
            20 => [first_scan(), second_scan()].concat(),
            _ => vec![call(113, 300, 301, 301)],
        }
    }

    fn query(symbol: u64, direction: Direction, depth: u32, include_tests: bool) -> UsageQuery {
        UsageQuery {
            symbol: SymbolId::new(symbol),
            direction,
            depth,
            include_tests,
        }
    }

    /// Asks every question in the order of `depths` and compares it with the walk in memory.
    fn usages_match(depths: &[u32]) {
        let store = two_projects();
        for snapshot in [10, 20, 30] {
            let index = ReverseIndex::new(visible(snapshot));
            for symbol in SYMBOLS {
                for direction in [Direction::Incoming, Direction::Outgoing] {
                    for &depth in depths {
                        for include_tests in [true, false] {
                            let asked = query(symbol, direction, depth, include_tests);
                            let page = store
                                .usages(SnapshotId::new(snapshot), &asked, LIMIT)
                                .unwrap();
                            let got: Vec<_> = page
                                .entries
                                .iter()
                                .map(|entry| (entry.depth, entry.reference.id.get()))
                                .collect();
                            let wanted = index.usages(&asked);
                            let expected: Vec<_> = wanted
                                .iter()
                                .map(|usage| (usage.depth, usage.reference.id.get()))
                                .collect();
                            assert_eq!(got, expected, "{snapshot} {asked:?}");
                            assert_eq!(page.total, wanted.len(), "{snapshot} {asked:?}");
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn usages_equal_the_walk_over_all_references_of_the_snapshot() {
        usages_match(&[1, 2, 3]);
    }

    #[test]
    fn usages_served_from_a_deeper_kept_walk_equal_the_walk_too() {
        usages_match(&[3, 2, 1]);
    }

    #[test]
    fn impact_after_a_deeper_walk_equals_the_walk_over_all_references_of_the_snapshot() {
        let store = two_projects();
        for symbol in SYMBOLS {
            let deep = query(symbol, Direction::Incoming, 3, true);
            store.usages(SnapshotId::new(20), &deep, LIMIT).unwrap();
        }
        impact_matches(&store);
    }

    #[test]
    fn impact_equals_the_walk_over_all_references_of_the_snapshot() {
        impact_matches(&two_projects());
    }

    fn impact_matches(store: &ProjectDb) {
        for snapshot in [10, 20, 30] {
            let index = ReverseIndex::new(visible(snapshot));
            for symbol in SYMBOLS {
                for depth in 1..=3 {
                    let asked = query(symbol, Direction::Incoming, depth, true);
                    let page = store
                        .impact(
                            SnapshotId::new(snapshot),
                            SymbolId::new(symbol),
                            depth,
                            LIMIT,
                        )
                        .unwrap();
                    let got: Vec<_> = page
                        .symbols
                        .iter()
                        .map(|found| (found.label.id.get(), found.depth, found.verified))
                        .collect();
                    let expected: Vec<_> = index
                        .reached(&asked)
                        .iter()
                        .map(|found| (found.symbol.get(), found.depth, found.verified))
                        .collect();
                    assert_eq!(got, expected, "{snapshot} {asked:?}");
                }
            }
        }
    }
}
