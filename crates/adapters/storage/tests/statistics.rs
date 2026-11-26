//! The symbols of a snapshot with their usage counts through the `CodeStatistics` port (ADR 0081).

#[cfg(test)]
mod common;

#[cfg(test)]
mod tests {
    use codeskimr_application::ports::{CodeStatistics, RankingRequest, StatisticsSetup};
    use codeskimr_domain::{
        Assessment, Confidence, KindGroup, MetricKind, Ranking, RankingQuery, ReferenceKind,
        SnapshotId, SymbolFacts, SymbolId, SymbolKind, UsageOrder, UsageTally,
        complexity_histogram, mark_tests, rank,
    };

    use super::common::{metric, reference, scanned, symbol};

    const SNAPSHOT: SnapshotId = SnapshotId::new(10);

    /// `Main` calls `Save` for certain and once only probably and overrides it (which makes no
    /// sense, but is a reference like any other), `Save` calls itself, and a test calls `Save`.
    fn store() -> codeskimr_storage::ProjectDb {
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

    /// The usages of `App.Repo.Save`: the certain and the probable call, not the test, not itself.
    const SAVE_USAGE: UsageTally = UsageTally {
        certain: 2,
        total: 3,
    };

    fn usage_of_save(facts: &[SymbolFacts]) -> UsageTally {
        let save = facts.iter().find(|fact| fact.id.get() == 23).unwrap();
        save.usage
    }

    #[test]
    fn usages_are_counted_without_tests_and_without_the_symbol_itself() {
        let facts = store().symbol_facts(SNAPSHOT).unwrap();
        assert_eq!(usage_of_save(&facts), SAVE_USAGE);
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

    /// Three files with types and members of every kind: used and unused, public and private,
    /// ties in every value, a constructor, an override and a test file that calls into the app.
    fn varied() -> codeskimr_storage::ProjectDb {
        filled(super::common::store(3))
    }

    fn filled(store: codeskimr_storage::ProjectDb) -> codeskimr_storage::ProjectDb {
        use codeskimr_application::ports::SnapshotStore;
        use codeskimr_domain::ScanMode;
        store
            .save_project(&super::common::project(1, "/work/app", &[]))
            .unwrap();
        let mut scan = super::common::data(super::common::header(10, ScanMode::Full));
        scan.files = vec![
            super::common::file(10, 11, "src/order.cs", "a"),
            super::common::file(10, 12, "src/cart.cs", "b"),
            super::common::file(10, 13, "tests/order_tests.cs", "c"),
        ];
        scan.symbols = varied_symbols();
        scan.references = varied_references();
        scan.metrics = varied_metrics();
        store.persist(scan).unwrap();
        store
    }

    fn varied_symbols() -> Vec<codeskimr_domain::Symbol> {
        use codeskimr_domain::Visibility;
        let kinds = [
            SymbolKind::Class,
            SymbolKind::Method,
            SymbolKind::Method,
            SymbolKind::Constructor,
            SymbolKind::Interface,
            SymbolKind::Function,
        ];
        let mut symbols = Vec::new();
        for (index, kind) in (0_u64..).zip(kinds.into_iter().cycle().take(18)) {
            let name = format!("App.S{:02}", 17 - index);
            let mut made = symbol(100 + index, 11 + index % 3, kind, &name, None);
            if index % 4 == 1 {
                made.visibility = Some(Visibility::Private);
            }
            if index % 7 == 3 {
                made.visibility = None;
            }
            symbols.push(made);
        }
        symbols
    }

    fn varied_references() -> Vec<codeskimr_domain::Reference> {
        let mut references = Vec::new();
        for id in 0_u64..40 {
            let mut made = reference(500 + id, 11 + id % 3, 100 + id % 18, 100 + (id * 7) % 11);
            made.confidence = if id % 3 == 0 {
                Confidence::Probable
            } else {
                Confidence::Certain
            };
            if id % 11 == 0 {
                made.kind = ReferenceKind::Override;
            }
            if id == 5 {
                made.kind = ReferenceKind::Test;
                made.file = codeskimr_domain::FileId::new(13);
            }
            references.push(made);
        }
        references
    }

    fn varied_metrics() -> Vec<codeskimr_domain::Metric> {
        let mut metrics = Vec::new();
        for id in 100_u64..118 {
            let value = f64::from(u32::try_from(id % 5).unwrap());
            metrics.push(metric(id, MetricKind::CyclomaticComplexity, value));
            metrics.push(metric(id, MetricKind::Loc, 10.0 + value));
            if id % 2 == 0 {
                metrics.push(metric(id, MetricKind::EfferentCoupling, value));
            }
        }
        metrics
    }

    fn setup<'a>(names: &'a [String], approximate: &'a [String]) -> StatisticsSetup<'a> {
        StatisticsSetup {
            test_names: names,
            approximate,
        }
    }

    fn queries() -> Vec<RankingQuery> {
        let mut all = Vec::new();
        for ranking in [
            Ranking::MostUsed,
            Ranking::Unused,
            Ranking::Complexity,
            Ranking::Size,
            Ranking::Coupling,
        ] {
            for kinds in [KindGroup::All, KindGroup::Types, KindGroup::Functions] {
                for include_tests in [false, true] {
                    for order in [UsageOrder::Certain, UsageOrder::All] {
                        all.push(RankingQuery {
                            ranking,
                            kinds,
                            language: None,
                            include_tests,
                            order,
                        });
                    }
                }
            }
        }
        all
    }

    fn marked(store: &codeskimr_storage::ProjectDb, names: &[String]) -> Vec<SymbolFacts> {
        let mut facts = store.symbol_facts(SNAPSHOT).unwrap();
        mark_tests(&mut facts, names);
        facts
    }

    /// The assessment order of the unused ranking, as the screen sorts it.
    fn expected(
        facts: &[SymbolFacts],
        query: &RankingQuery,
        approximate: &[String],
        wanted: Option<Assessment>,
        limit: usize,
    ) -> (Vec<u64>, usize) {
        let assess = |fact: &SymbolFacts| {
            Assessment::of(
                fact.visibility,
                approximate
                    .iter()
                    .any(|name| name == fact.language.as_str()),
            )
        };
        let mut rows = rank(facts, query, usize::MAX).rows;
        if query.ranking == Ranking::Unused {
            rows.sort_by_key(|fact| assess(fact));
            if let Some(wanted) = wanted {
                rows.retain(|fact| assess(fact) == wanted);
            }
        }
        let total = rows.len();
        (
            rows.iter().take(limit).map(|fact| fact.id.get()).collect(),
            total,
        )
    }

    /// Two more projects with files of their own, so that the references of the first are read
    /// through its files and not in one pass over the table.
    fn next_to_larger_projects(
        store: codeskimr_storage::ProjectDb,
    ) -> codeskimr_storage::ProjectDb {
        use codeskimr_application::ports::SnapshotStore;
        use codeskimr_domain::{ProjectId, ScanMode};

        use super::common::{data, file, header, project};

        for (id, snapshot) in [(2, 20), (3, 30)] {
            let folder = format!("/work/other{id}");
            store.save_project(&project(id, &folder, &[])).unwrap();
            let mut head = header(snapshot, ScanMode::Full);
            head.project = ProjectId::new(id);
            let mut scan = data(head);
            scan.files = (1..=3)
                .map(|at| file(snapshot, snapshot * 100 + at, &format!("src/f{at}.cs"), "h"))
                .collect();
            store.persist(scan).unwrap();
        }
        store
    }

    #[test]
    fn every_ranking_equals_the_calculation_over_all_symbols() {
        rankings_equal_the_calculation(&varied());
    }

    #[test]
    fn the_rankings_are_the_same_when_the_references_are_read_through_the_files() {
        rankings_equal_the_calculation(&next_to_larger_projects(varied()));
    }

    fn rankings_equal_the_calculation(store: &codeskimr_storage::ProjectDb) {
        let names = vec!["tests".to_owned()];
        let facts = marked(store, &names);
        assert!(facts.iter().any(|fact| fact.test) && facts.iter().any(|fact| !fact.test));
        for approximate in [Vec::new(), vec!["csharp".to_owned()]] {
            for query in queries() {
                for wanted in [None, Some(Assessment::Likely), Some(Assessment::Public)] {
                    for limit in [3, 1000] {
                        let found = store
                            .ranked(&RankingRequest {
                                snapshot: SNAPSHOT,
                                query: &query,
                                assessment: wanted,
                                limit,
                                setup: setup(&names, &approximate),
                            })
                            .unwrap();
                        let ids: Vec<u64> = found.rows.iter().map(|fact| fact.id.get()).collect();
                        let (want_ids, want_total) =
                            expected(&facts, &query, &approximate, wanted, limit);
                        assert_eq!((ids, found.total), (want_ids, want_total), "{query:?}");
                        for row in &found.rows {
                            let same = facts.iter().find(|fact| fact.id == row.id).unwrap();
                            assert_eq!(row, same);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn the_sums_of_the_overview_equal_the_calculation_over_all_symbols() {
        let store = varied();
        let names = vec!["tests".to_owned()];
        let facts = marked(&store, &names);
        for include_tests in [false, true] {
            let summary = store
                .summary(SNAPSHOT, include_tests, setup(&names, &[]))
                .unwrap();
            let ranked = facts
                .iter()
                .filter(|fact| KindGroup::All.holds(fact.kind) && (include_tests || !fact.test))
                .count();
            assert_eq!(summary.ranked as usize, ranked);
            assert_eq!(
                summary.histogram,
                complexity_histogram(&facts, include_tests)
            );
            let sum = |part: fn(UsageTally) -> u32| -> u64 {
                facts.iter().map(|fact| u64::from(part(fact.usage))).sum()
            };
            assert_eq!(summary.certain_references, sum(|tally| tally.certain));
            assert_eq!(summary.probable_references, sum(UsageTally::probable));
        }
    }

    #[test]
    fn the_counts_of_the_unused_symbols_equal_the_calculation_over_all_symbols() {
        let store = varied();
        let names = vec!["tests".to_owned()];
        let facts = marked(&store, &names);
        for approximate in [Vec::new(), vec!["csharp".to_owned()]] {
            let mut query = queries()[0].clone();
            query.ranking = Ranking::Unused;
            for include_tests in [false, true] {
                query.include_tests = include_tests;
                let counts = store
                    .unused_counts(SNAPSHOT, &query, setup(&names, &approximate))
                    .unwrap();
                let shown = rank(&facts, &query, usize::MAX).rows;
                let with_tests = RankingQuery {
                    include_tests: true,
                    ..query.clone()
                };
                let per = |wanted: Assessment| {
                    shown
                        .iter()
                        .filter(|fact| {
                            Assessment::of(
                                fact.visibility,
                                approximate.contains(&fact.language.to_string()),
                            ) == wanted
                        })
                        .count()
                };
                assert_eq!(counts.likely as usize, per(Assessment::Likely));
                assert_eq!(counts.public as usize, per(Assessment::Public));
                assert_eq!(counts.approximate as usize, per(Assessment::Approximate));
                assert_eq!(
                    counts.tests_hidden as usize,
                    rank(&facts, &with_tests, usize::MAX).total - shown.len()
                );
            }
        }
    }

    #[test]
    fn the_unused_facts_are_the_unused_symbols_of_the_calculation_in_the_order_of_their_place() {
        let store = varied();
        let names = vec!["tests".to_owned()];
        let facts = marked(&store, &names);
        for query in queries()
            .into_iter()
            .filter(|query| query.ranking == Ranking::Unused && query.order == UsageOrder::Certain)
        {
            let read = store
                .unused_facts(SNAPSHOT, &query, setup(&names, &[]))
                .unwrap();
            // Test code comes back too: the caller decides whether it counts.
            let with_tests = RankingQuery {
                include_tests: true,
                ..query.clone()
            };
            let expected = rank(&facts, &with_tests, usize::MAX).rows;
            assert_eq!(read, expected.into_iter().cloned().collect::<Vec<_>>());
        }
    }

    #[test]
    fn other_test_folders_count_the_numbers_again() {
        let store = varied();
        let query = RankingQuery {
            ranking: Ranking::MostUsed,
            kinds: KindGroup::All,
            language: None,
            include_tests: false,
            order: UsageOrder::Certain,
        };
        let ask = |names: &[String]| {
            store
                .ranked(&RankingRequest {
                    snapshot: SNAPSHOT,
                    query: &query,
                    assessment: None,
                    limit: 1000,
                    setup: setup(names, &[]),
                })
                .unwrap()
                .total
        };
        let none: Vec<String> = Vec::new();
        let named = vec!["src".to_owned()];
        let without = ask(&none);
        assert_eq!(ask(&named), 0);
        assert_eq!(ask(&none), without);
    }

    #[test]
    fn a_snapshot_without_the_numbers_gets_them_at_the_first_question() {
        let db = super::common::TempDb::new();
        let store = filled(
            codeskimr_storage::ProjectDb::open(&db.0, std::num::NonZeroU32::new(3).unwrap())
                .unwrap(),
        );
        let conn = rusqlite::Connection::open(&db.0).unwrap();
        conn.execute_batch("DELETE FROM symbol_statistics; DELETE FROM snapshot_statistics;")
            .unwrap();
        let none: Vec<String> = Vec::new();
        let facts = marked(&store, &none);
        let summary = store.summary(SNAPSHOT, true, setup(&none, &[])).unwrap();
        assert_eq!(
            summary.ranked as usize,
            facts
                .iter()
                .filter(|f| KindGroup::All.holds(f.kind))
                .count()
        );
    }

    /// With larger projects in the database, the references of a snapshot are read through its
    /// files; the counts are the same as in a database of its own.
    #[test]
    fn the_usages_are_the_same_next_to_larger_projects() {
        let store = next_to_larger_projects(store());
        let facts = store.symbol_facts(SNAPSHOT).unwrap();
        assert_eq!(usage_of_save(&facts), SAVE_USAGE);
        assert!(facts.iter().all(|fact| fact.test));
    }
}
