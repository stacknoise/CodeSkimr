//! The reads of the comparison of two scans (ADR 0097): the symbols and the naming rules of a
//! retained snapshot that is not the newest one, and the usages in it.

#[cfg(test)]
mod common;

#[cfg(test)]
mod tests {
    use codeskimr_application::ports::{ComparisonSource, SnapshotStore, SymbolHash, UsageIndex};
    use codeskimr_domain::{
        ContentHash, Direction, MetricKind, MetricLevel, ScanMode, SnapshotId, SymbolId,
        SymbolKind, SymbolOrigin, UsageQuery, Visibility, compare_symbols,
    };
    use codeskimr_storage::ProjectDb;

    use super::common::{
        data, file, header, metric, project, reference, registered_store, store, symbol,
    };

    const FIRST: SnapshotId = SnapshotId::new(10);
    const SECOND: SnapshotId = SnapshotId::new(20);

    fn hashed(symbol: u64, hash: &str) -> SymbolHash {
        SymbolHash {
            symbol: SymbolId::new(symbol),
            hash: ContentHash::new(hash).unwrap(),
        }
    }

    /// Scan 10: `App` (namespace), `App.A` with `Run` and `Gone`, a parameter; `Run` calls `Gone`.
    fn first_scan() -> codeskimr_application::ports::SnapshotData {
        let mut scan = data(header(10, ScanMode::Full));
        scan.files = vec![file(10, 11, "src/a.cs", "ha")];
        let mut run = symbol(23, 11, SymbolKind::Method, "App.A.Run", Some(22));
        run.signature = Some("int Run(int count, string name)".to_owned());
        run.visibility = Some(Visibility::Private);
        let mut from_index = symbol(26, 11, SymbolKind::Method, "App.A.FromIndex", Some(22));
        from_index.origin = SymbolOrigin::Index;
        scan.symbols = vec![
            from_index,
            symbol(21, 11, SymbolKind::Namespace, "App", None),
            symbol(22, 11, SymbolKind::Class, "App.A", Some(21)),
            run,
            symbol(24, 11, SymbolKind::Method, "App.A.Gone", Some(22)),
            symbol(25, 11, SymbolKind::Parameter, "App.A.Run.count", Some(23)),
        ];
        scan.references = vec![reference(31, 11, 23, 24)];
        scan.hashes = vec![hashed(22, "h22"), hashed(23, "h23")];
        scan.metrics = vec![
            metric(23, MetricKind::CyclomaticComplexity, 6.0),
            metric(23, MetricKind::MaintainabilityIndex, 71.0),
            metric(21, MetricKind::EfferentCoupling, 4.0),
            metric(25, MetricKind::Loc, 3.0),
        ];
        scan.naming_rules = "csharp=file".to_owned();
        scan
    }

    /// Scan 20, incremental: file `a.cs` stays, `b.cs` is new and holds `App.B`.
    fn second_scan() -> codeskimr_application::ports::SnapshotData {
        let mut scan = data(header(20, ScanMode::Incremental));
        scan.files = vec![file(20, 12, "src/b.cs", "hb")];
        scan.symbols = vec![symbol(41, 12, SymbolKind::Class, "App.B", None)];
        scan.hashes = vec![hashed(41, "h41")];
        scan.naming_rules = "csharp=namespace".to_owned();
        scan
    }

    fn two_scans() -> ProjectDb {
        let store = registered_store();
        store.persist(first_scan()).unwrap();
        store.persist(second_scan()).unwrap();
        store
    }

    #[test]
    fn the_facts_of_an_older_snapshot_name_the_file_the_hash_and_the_visibility() {
        let facts = two_scans().comparison_facts(FIRST).unwrap();
        let seen: Vec<(u64, &str, &str)> = facts
            .iter()
            .map(|fact| {
                (
                    fact.id.get(),
                    fact.qualified_name.as_str(),
                    fact.path.as_str(),
                )
            })
            .collect();
        assert_eq!(
            seen,
            [
                (22, "App.A", "src/a.cs"),
                (23, "App.A.Run", "src/a.cs"),
                (24, "App.A.Gone", "src/a.cs")
            ],
            "a namespace, a parameter and a symbol of level 2 take no part"
        );
        assert_eq!(facts[1].visibility, Some(Visibility::Private));
        assert_eq!(
            facts[1].signature.as_deref(),
            Some("int Run(int count, string name)")
        );
        assert_eq!(
            facts[1].content_hash.as_ref().map(ContentHash::as_str),
            Some("h23")
        );
        assert_eq!(facts[2].content_hash, None, "no hash was stored for it");
    }

    #[test]
    fn a_snapshot_holds_only_the_symbols_of_its_own_files() {
        let store = two_scans();
        let later = store.comparison_facts(SECOND).unwrap();
        let ids: Vec<u64> = later.iter().map(|fact| fact.id.get()).collect();
        assert_eq!(ids, [22, 23, 24, 41], "the carried file and the new one");
        let earlier = store.comparison_facts(FIRST).unwrap();
        assert!(earlier.iter().all(|fact| fact.id.get() != 41));
    }

    #[test]
    fn two_snapshots_compare_through_the_facts() {
        let store = two_scans();
        let found = compare_symbols(
            &store.comparison_facts(FIRST).unwrap(),
            &store.comparison_facts(SECOND).unwrap(),
        );
        assert_eq!(found.added.len(), 1);
        assert_eq!(found.added[0].qualified_name.as_str(), "App.B");
        assert!(found.removed.is_empty() && found.changed.is_empty());
    }

    #[test]
    fn the_metric_facts_hold_the_values_of_the_comparable_objects() {
        let facts = two_scans().metric_facts(FIRST).unwrap();
        let seen: Vec<_> = facts
            .iter()
            .map(|fact| {
                let values = fact.values;
                (
                    fact.qualified_name.as_str(),
                    values.cc,
                    values.mi,
                    values.ce,
                )
            })
            .collect();
        assert_eq!(
            seen,
            [
                ("App", None, None, Some(4.0)),
                ("App.A.Run", Some(6.0), Some(71.0), None)
            ],
            "only objects that have a metric come, and a parameter is no object of a level"
        );
        assert_eq!(MetricLevel::of(facts[0].kind), Some(MetricLevel::Module));
    }

    #[test]
    fn the_naming_rules_are_read_per_snapshot() {
        let store = two_scans();
        assert_eq!(
            store.snapshot_naming_rules(FIRST).unwrap().as_deref(),
            Some("csharp=file")
        );
        assert_eq!(
            store.snapshot_naming_rules(SECOND).unwrap().as_deref(),
            Some("csharp=namespace")
        );
        assert_eq!(
            store.snapshot_naming_rules(SnapshotId::new(99)).unwrap(),
            None
        );
    }

    #[test]
    fn a_snapshot_that_retention_removed_has_no_facts() {
        let store = store(2);
        store.save_project(&project(1, "/work/app", &[])).unwrap();
        store.persist(first_scan()).unwrap();
        store.persist(second_scan()).unwrap();
        let mut third = data(header(30, ScanMode::Incremental));
        third.files = vec![file(30, 13, "src/c.cs", "hc")];
        store.persist(third).unwrap();
        assert_eq!(store.comparison_facts(FIRST).unwrap().len(), 0);
        assert_eq!(store.snapshot_naming_rules(FIRST).unwrap(), None);
        assert_ne!(store.comparison_facts(SECOND).unwrap().len(), 0);
    }

    #[test]
    fn the_usages_of_an_older_snapshot_are_read_in_that_snapshot() {
        let store = two_scans();
        let query = UsageQuery {
            symbol: SymbolId::new(24),
            direction: Direction::Incoming,
            depth: 1,
            include_tests: true,
        };
        let page = store.usages(FIRST, &query, 10).unwrap();
        assert_eq!(page.total, 1);
        assert_eq!(page.entries[0].from.id, SymbolId::new(23));
        assert_eq!(page.entries[0].path.as_str(), "src/a.cs");
        assert_eq!(page.entries[0].from.qualified_name.as_str(), "App.A.Run");
    }

    /// Scan 20 analyses `a.cs` again: `Gone` is deleted and `Run` returns a `long`.
    fn a_rewritten() -> codeskimr_application::ports::SnapshotData {
        let mut scan = data(header(20, ScanMode::Incremental));
        scan.files = vec![file(20, 12, "src/a.cs", "ha2")];
        let mut run = symbol(52, 12, SymbolKind::Method, "App.A.Run", Some(51));
        run.signature = Some("long Run(int count, string name)".to_owned());
        run.visibility = Some(Visibility::Private);
        scan.symbols = vec![symbol(51, 12, SymbolKind::Class, "App.A", None), run];
        scan.hashes = vec![hashed(51, "h22"), hashed(52, "h52")];
        scan.naming_rules = "csharp=file".to_owned();
        scan
    }

    #[test]
    fn a_rewritten_file_shows_the_difference_and_keeps_the_usages_of_the_base() {
        let store = registered_store();
        store.persist(first_scan()).unwrap();
        store.persist(a_rewritten()).unwrap();
        let found = compare_symbols(
            &store.comparison_facts(FIRST).unwrap(),
            &store.comparison_facts(SECOND).unwrap(),
        );
        let removed: Vec<&str> = found
            .removed
            .iter()
            .map(|fact| fact.qualified_name.as_str())
            .collect();
        assert_eq!(removed, ["App.A.Gone"]);
        assert_eq!(found.changed.len(), 1);
        let subjects = found.impact_subjects();
        assert_eq!(
            subjects.len(),
            2,
            "the removed symbol and the changed signature"
        );
        let query = |symbol| UsageQuery {
            symbol: SymbolId::new(symbol),
            direction: Direction::Incoming,
            depth: 1,
            include_tests: true,
        };
        let in_base = store.usages(FIRST, &query(24), 10).unwrap();
        assert_eq!(in_base.entries[0].from.id, SymbolId::new(23));
        assert!(found.touched_in_base().contains(&SymbolId::new(23)));
        assert_eq!(store.usages(SECOND, &query(24), 10).unwrap().total, 0);
    }
}
