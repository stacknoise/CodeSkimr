//! The symbols of a snapshot with their usage counts through the `CodeStatistics` port (ADR 0081).

#[cfg(test)]
mod common;

#[cfg(test)]
mod tests {
    use codeskimr_application::ports::CodeStatistics;
    use codeskimr_domain::{
        Confidence, MetricKind, ReferenceKind, SnapshotId, SymbolId, SymbolKind, UsageTally,
    };

    use super::common::{metric, reference, scanned, symbol};

    const SNAPSHOT: SnapshotId = SnapshotId::new(10);

    /// `Main` calls `Save` for certain and once only probably and overrides it (which makes no
    /// sense, but is a reference like any other), `Save` calls itself, and a test calls `Save`.
    fn store() -> codeskimr_storage::SqliteStore {
        let symbols = vec![
            symbol(20, 11, SymbolKind::Class, "App.Repo", None),
            symbol(21, 11, SymbolKind::Method, "App.Main", None),
            symbol(23, 11, SymbolKind::Method, "App.Repo.Save", Some(20)),
        ];
        let mut probable = reference(32, 11, 21, 23);
        probable.confidence = Confidence::Probable;
        let mut in_test = reference(34, 11, 21, 23);
        in_test.kind = ReferenceKind::Test;
        let mut overrides = reference(35, 11, 21, 23);
        overrides.kind = ReferenceKind::Override;
        let references = vec![
            overrides,
            reference(31, 11, 21, 23),
            probable,
            reference(33, 11, 23, 23),
            in_test,
        ];
        let metrics = vec![
            metric(23, MetricKind::CyclomaticComplexity, 7.0),
            metric(23, MetricKind::Loc, 12.0),
            metric(20, MetricKind::EfferentCoupling, 3.0),
        ];
        scanned(&[], symbols, references, metrics)
    }

    #[test]
    fn usages_are_counted_without_tests_and_without_the_symbol_itself() {
        let facts = store().symbol_facts(SNAPSHOT).unwrap();
        let save = facts.iter().find(|fact| fact.id.get() == 23).unwrap();
        assert_eq!(
            save.usage,
            UsageTally {
                certain: 2,
                total: 3
            }
        );
        let main = facts.iter().find(|fact| fact.id.get() == 21).unwrap();
        assert_eq!(main.usage, UsageTally::default());
    }

    #[test]
    fn a_fact_carries_the_place_the_parent_and_the_measured_values() {
        let facts = store().symbol_facts(SNAPSHOT).unwrap();
        let ids: Vec<u64> = facts.iter().map(|fact| fact.id.get()).collect();
        assert_eq!(ids, [20, 21, 23]);
        let save = &facts[2];
        assert_eq!(save.qualified_name.as_str(), "App.Repo.Save");
        assert_eq!(save.parent, Some(SymbolId::new(20)));
        assert_eq!(save.language.as_str(), "csharp");
        assert_eq!(save.measures.complexity, Some(7.0));
        assert_eq!(save.measures.loc, Some(12.0));
        assert_eq!(facts[0].measures.efferent, Some(3.0));
        assert_eq!(facts[0].measures.complexity, None);
    }

    #[test]
    fn the_source_of_an_override_reference_is_marked_as_overriding() {
        let facts = store().symbol_facts(SNAPSHOT).unwrap();
        let marked: Vec<bool> = facts.iter().map(|fact| fact.overrides).collect();
        assert_eq!(marked, [false, true, false]);
    }

    #[test]
    fn the_symbols_of_a_file_with_a_test_reference_are_test_code() {
        let facts = store().symbol_facts(SNAPSHOT).unwrap();
        assert!(facts.iter().all(|fact| fact.test));
    }

    #[test]
    fn the_shared_facts_are_loaded_once_and_marked_with_the_test_folders() {
        let store = store();
        let none: Vec<String> = Vec::new();
        let first = store.shared_facts(SNAPSHOT, &none).unwrap();
        let again = store.shared_facts(SNAPSHOT, &none).unwrap();
        assert!(std::sync::Arc::ptr_eq(&first, &again));
        assert_eq!(first.len(), 3);
        // Other test folders ask for their own marking, never a copy marked for different ones.
        let named = vec!["app".to_owned()];
        let other = store.shared_facts(SNAPSHOT, &named).unwrap();
        assert!(!std::sync::Arc::ptr_eq(&first, &other));
    }
}
