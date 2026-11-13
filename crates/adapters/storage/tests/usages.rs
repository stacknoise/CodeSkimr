//! Usages and impact through the `UsageIndex` port.

#[cfg(test)]
mod common;

#[cfg(test)]
mod tests {
    use codeskimr_application::ports::{DependencyQuery, SnapshotStore, UsageIndex};
    use codeskimr_domain::{
        Direction, ReferenceKind, ScanMode, SnapshotId, SymbolId, SymbolKind, UsageQuery,
        Visibility,
    };

    use super::common::{data, file, header, reference, scanned, symbol};

    const SNAPSHOT: SnapshotId = SnapshotId::new(10);

    fn query(symbol: u64, direction: Direction, depth: u32) -> UsageQuery {
        UsageQuery {
            symbol: SymbolId::new(symbol),
            direction,
            depth,
            include_tests: true,
        }
    }

    /// `Main` calls `Service.Run`, `Service.Run` calls `Repo.Save`, a test also calls `Repo.Save`.
    fn calls() -> codeskimr_storage::SqliteStore {
        let symbols = vec![
            symbol(21, 11, SymbolKind::Method, "App.Main", None),
            symbol(22, 11, SymbolKind::Method, "App.Service.Run", None),
            symbol(23, 11, SymbolKind::Method, "App.Repo.Save", None),
            symbol(24, 11, SymbolKind::Method, "App.Tests.SaveWorks", None),
        ];
        let mut in_test = reference(33, 11, 24, 23);
        in_test.kind = ReferenceKind::Test;
        let references = vec![
            reference(31, 11, 21, 22),
            reference(32, 11, 22, 23),
            in_test,
        ];
        scanned(&[], symbols, references, vec![])
    }

    #[test]
    fn usages_name_both_ends_the_file_and_the_step() {
        let page = calls()
            .usages(SNAPSHOT, &query(23, Direction::Incoming, 2), 10)
            .unwrap();
        assert_eq!(page.total, 3);
        let seen: Vec<_> = page
            .entries
            .iter()
            .map(|entry| {
                (
                    entry.depth,
                    entry.from.qualified_name.as_str(),
                    entry.to.qualified_name.as_str(),
                    entry.path.as_str(),
                )
            })
            .collect();
        assert_eq!(
            seen,
            [
                (1, "App.Service.Run", "App.Repo.Save", "src/order.cs"),
                (1, "App.Tests.SaveWorks", "App.Repo.Save", "src/order.cs"),
                (2, "App.Main", "App.Service.Run", "src/order.cs"),
            ]
        );
    }

    #[test]
    fn the_limit_cuts_the_entries_but_not_the_total() {
        let page = calls()
            .usages(SNAPSHOT, &query(23, Direction::Incoming, 2), 1)
            .unwrap();
        assert_eq!((page.total, page.entries.len()), (3, 1));
    }

    #[test]
    fn tests_are_left_out_when_not_asked_for() {
        let mut question = query(23, Direction::Incoming, 1);
        question.include_tests = false;
        let page = calls().usages(SNAPSHOT, &question, 10).unwrap();
        assert_eq!(page.total, 1);
    }

    #[test]
    fn outgoing_walks_follow_what_a_symbol_uses() {
        let page = calls()
            .usages(SNAPSHOT, &query(21, Direction::Outgoing, 3), 10)
            .unwrap();
        let targets: Vec<_> = page
            .entries
            .iter()
            .map(|entry| entry.to.qualified_name.as_str())
            .collect();
        assert_eq!(targets, ["App.Service.Run", "App.Repo.Save"]);
    }

    #[test]
    fn impact_lists_what_a_change_reaches_nearest_first() {
        let page = calls().impact(SNAPSHOT, SymbolId::new(23), 5, 10).unwrap();
        let reached: Vec<_> = page
            .symbols
            .iter()
            .map(|found| (found.depth, found.label.qualified_name.as_str()))
            .collect();
        assert_eq!(
            reached,
            [
                (1, "App.Service.Run"),
                (1, "App.Tests.SaveWorks"),
                (2, "App.Main")
            ]
        );
        assert_eq!(page.symbols[0].visibility, Some(Visibility::Public));
        assert_eq!(page.total, 3);
    }

    #[test]
    fn an_unknown_symbol_has_no_usages() {
        let store = calls();
        let page = store
            .usages(SNAPSHOT, &query(99, Direction::Incoming, 3), 10)
            .unwrap();
        assert_eq!((page.total, page.entries.len()), (0, 0));
        assert_eq!(
            store
                .impact(SNAPSHOT, SymbolId::new(99), 3, 10)
                .unwrap()
                .total,
            0
        );
    }

    #[test]
    fn every_snapshot_has_its_own_index() {
        let store = calls();
        let before = store
            .usages(SNAPSHOT, &query(23, Direction::Incoming, 1), 10)
            .unwrap();
        assert_eq!(before.total, 2);
        let mut scan = data(header(40, ScanMode::Full));
        scan.files = vec![file(40, 41, "src/order.cs", "h2")];
        scan.symbols = vec![
            symbol(51, 41, SymbolKind::Method, "App.Main", None),
            symbol(52, 41, SymbolKind::Method, "App.Repo.Save", None),
        ];
        scan.references = vec![reference(61, 41, 51, 52)];
        store.persist(scan).unwrap();
        let newest = SnapshotId::new(40);
        let after = store
            .usages(newest, &query(52, Direction::Incoming, 1), 10)
            .unwrap();
        assert_eq!(after.total, 1);
        let again = store
            .usages(SNAPSHOT, &query(23, Direction::Incoming, 1), 10)
            .unwrap();
        assert_eq!(again, before);
    }

    #[test]
    fn a_snapshot_that_is_not_stored_is_empty_and_keeps_the_cache() {
        let store = calls();
        let question = query(23, Direction::Incoming, 1);
        assert_eq!(store.usages(SNAPSHOT, &question, 10).unwrap().total, 2);
        let missing = store.usages(SnapshotId::new(99), &question, 10).unwrap();
        assert_eq!((missing.total, missing.entries.len()), (0, 0));
        assert_eq!(store.usages(SNAPSHOT, &question, 10).unwrap().total, 2);
    }

    #[test]
    fn outgoing_walks_skip_tests_when_not_asked_for() {
        let mut question = query(24, Direction::Outgoing, 2);
        assert_eq!(calls().usages(SNAPSHOT, &question, 10).unwrap().total, 1);
        question.include_tests = false;
        assert_eq!(calls().usages(SNAPSHOT, &question, 10).unwrap().total, 0);
    }

    #[test]
    fn impact_respects_depth_and_limit() {
        let store = calls();
        let shallow = store.impact(SNAPSHOT, SymbolId::new(23), 1, 10).unwrap();
        assert_eq!(shallow.total, 2);
        let cut = store.impact(SNAPSHOT, SymbolId::new(23), 5, 1).unwrap();
        assert_eq!((cut.total, cut.symbols.len()), (3, 1));
    }

    /// `App.Service` contains `Run` and `Stop`; `Main` uses `Run`, `Run` calls `Repo.Save` twice
    /// and `Stop` once, a test calls `Run`, and `Run` and `Stop` call each other.
    fn service() -> codeskimr_storage::SqliteStore {
        let symbols = vec![
            symbol(20, 11, SymbolKind::Class, "App.Service", None),
            symbol(22, 11, SymbolKind::Method, "App.Service.Run", Some(20)),
            symbol(25, 11, SymbolKind::Method, "App.Service.Stop", Some(20)),
            symbol(21, 11, SymbolKind::Method, "App.Main", None),
            symbol(23, 11, SymbolKind::Method, "App.Repo.Save", None),
            symbol(24, 11, SymbolKind::Method, "App.Tests.RunWorks", None),
        ];
        let mut in_test = reference(36, 11, 24, 22);
        in_test.kind = ReferenceKind::Test;
        let references = vec![
            reference(31, 11, 21, 22),
            reference(32, 11, 22, 23),
            reference(33, 11, 22, 23),
            reference(34, 11, 25, 23),
            reference(35, 11, 22, 25),
            in_test,
        ];
        scanned(&[], symbols, references, vec![])
    }

    fn scope(direction: Direction, include_tests: bool) -> DependencyQuery {
        DependencyQuery {
            symbol: SymbolId::new(20),
            direction,
            include_tests,
        }
    }

    #[test]
    fn dependencies_count_references_of_the_whole_scope_and_skip_inner_ones() {
        let page = service()
            .dependencies(SNAPSHOT, &scope(Direction::Outgoing, false), 10)
            .unwrap();
        let seen: Vec<_> = page
            .edges
            .iter()
            .map(|edge| (edge.other.qualified_name.as_str(), edge.count))
            .collect();
        assert_eq!(seen, vec![("App.Repo.Save", 3)]);
        assert_eq!(page.total, 1);
    }

    #[test]
    fn incoming_dependencies_and_tests_follow_the_question() {
        let store = service();
        let without = store
            .dependencies(SNAPSHOT, &scope(Direction::Incoming, false), 10)
            .unwrap();
        assert_eq!(without.total, 1);
        let with = store
            .dependencies(SNAPSHOT, &scope(Direction::Incoming, true), 10)
            .unwrap();
        assert_eq!(with.total, 2);
        assert_eq!(with.edges[0].path.as_str(), "src/order.cs");
    }

    #[test]
    fn dependencies_are_cut_to_the_limit_and_missing_scopes_are_empty() {
        let store = service();
        let cut = store
            .dependencies(SNAPSHOT, &scope(Direction::Incoming, true), 1)
            .unwrap();
        assert_eq!((cut.total, cut.edges.len()), (2, 1));
        let mut missing = scope(Direction::Outgoing, true);
        missing.symbol = SymbolId::new(99);
        assert_eq!(store.dependencies(SNAPSHOT, &missing, 10).unwrap().total, 0);
    }
}
