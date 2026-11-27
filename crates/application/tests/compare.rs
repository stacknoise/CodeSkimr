//! The comparison of two scans and its impact (ADR 0097) over fake ports.

#[cfg(test)]
#[path = "common/alerts.rs"]
mod fakes;

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex, PoisonError};

    use codeskimr_application::compare::{
        CompareLimits, ComparePorts, ComparisonRequest, CriticalOutcome, ScanComparer, UsageStatus,
    };
    use codeskimr_application::error::CodedError;
    use codeskimr_application::ports::{
        CodeIndex, CodeSpan, ComparisonSource, FileHit, ScanEntry, SearchHit, SymbolDetail,
        SymbolKey, SymbolLabel, TypeFocus, UsageEntry, UsageIndex, UsagePage,
    };
    use codeskimr_codes::{Coded, ErrorCode};
    use codeskimr_domain::{
        Confidence, ContentHash, FileId, LayerEdge, LayerName, NamespaceGraph, Position, ProjectId,
        QualifiedName, Reference, ReferenceId, ReferenceKind, RelativePath, ScanMode, Snapshot,
        SnapshotId, Span, SymbolFact, SymbolId, SymbolKind, Timestamp, UsageQuery, Visibility,
    };

    use super::fakes::{MemoryAnnotations, PROJECT};

    const OLD: SnapshotId = SnapshotId::new(10);
    const MID: SnapshotId = SnapshotId::new(20);
    const NEW: SnapshotId = SnapshotId::new(30);

    fn entry(id: SnapshotId) -> ScanEntry {
        ScanEntry {
            scan: id.get(),
            snapshot: id,
            created: Timestamp(id.get().try_into().unwrap()),
            commit: None,
            mode: ScanMode::Full,
            files: 1,
            symbols: 1,
            duration_ms: None,
            files_changed: None,
            steps: Vec::new(),
        }
    }

    fn fact(id: u64, name: &str, signature: &str, hash: &str) -> SymbolFact {
        SymbolFact {
            id: SymbolId::new(id),
            kind: SymbolKind::Method,
            qualified_name: QualifiedName::new(name).unwrap(),
            signature: Some(signature.to_owned()),
            visibility: Some(Visibility::Public),
            path: RelativePath::new("src/a.cs").unwrap(),
            content_hash: Some(ContentHash::new(hash).unwrap()),
        }
    }

    fn edge(from: &str, to: &str, violates: bool, cycle: bool) -> LayerEdge {
        LayerEdge {
            from: LayerName::new(from).unwrap(),
            to: LayerName::new(to).unwrap(),
            count: 3,
            violates_layering: violates,
            in_cycle: cycle,
        }
    }

    #[derive(Default, Clone)]
    struct Index {
        history: Vec<ScanEntry>,
        edges: HashMap<SnapshotId, Vec<LayerEdge>>,
        /// The history is empty once it was read this many times: retention removed the scans.
        vanish_after: Option<usize>,
        reads: Arc<Mutex<usize>>,
    }

    impl CodeIndex for Index {
        fn scan_history(&self, _: ProjectId) -> Result<Vec<ScanEntry>, CodedError> {
            let mut reads = self.reads.lock().unwrap_or_else(PoisonError::into_inner);
            *reads += 1;
            if self.vanish_after.is_some_and(|after| *reads > after) {
                return Ok(Vec::new());
            }
            Ok(self.history.clone())
        }

        fn latest_snapshot(&self, _: ProjectId) -> Result<Option<Snapshot>, CodedError> {
            Ok(None)
        }

        fn search(&self, _: SnapshotId, _: &str, _: u32) -> Result<Vec<SearchHit>, CodedError> {
            Ok(Vec::new())
        }

        fn symbol(&self, _: SnapshotId, _: &SymbolKey) -> Result<Option<SymbolDetail>, CodedError> {
            Ok(None)
        }

        fn search_files(&self, _: SnapshotId, _: &str, _: u32) -> Result<Vec<FileHit>, CodedError> {
            Ok(Vec::new())
        }

        fn code_spans(
            &self,
            _: SnapshotId,
            _: &RelativePath,
            _: (u32, u32),
        ) -> Result<Vec<CodeSpan>, CodedError> {
            Ok(Vec::new())
        }

        fn layer_edges(&self, snapshot: SnapshotId) -> Result<Vec<LayerEdge>, CodedError> {
            Ok(self.edges.get(&snapshot).cloned().unwrap_or_default())
        }

        fn namespace_graph(&self, _: SnapshotId) -> Result<NamespaceGraph, CodedError> {
            Err(CodedError::new(ErrorCode::UnexpectedInternalError))
        }

        fn type_focus(&self, _: SnapshotId, _: SymbolId) -> Result<Option<TypeFocus>, CodedError> {
            Ok(None)
        }
    }

    #[derive(Default, Clone)]
    struct Source {
        facts: HashMap<SnapshotId, Vec<SymbolFact>>,
        rules: HashMap<SnapshotId, String>,
    }

    impl ComparisonSource for Source {
        fn comparison_facts(&self, snapshot: SnapshotId) -> Result<Vec<SymbolFact>, CodedError> {
            Ok(self.facts.get(&snapshot).cloned().unwrap_or_default())
        }

        fn snapshot_naming_rules(
            &self,
            snapshot: SnapshotId,
        ) -> Result<Option<String>, CodedError> {
            Ok(self.rules.get(&snapshot).cloned())
        }
    }

    /// The usages by the id of the used symbol: (id of the symbol around it, file).
    #[derive(Default, Clone)]
    struct Usages {
        of: HashMap<u64, Vec<(u64, &'static str)>>,
        asked: Arc<Mutex<Vec<(SnapshotId, u64, u32)>>>,
    }

    fn label(id: u64) -> SymbolLabel {
        SymbolLabel {
            id: SymbolId::new(id),
            qualified_name: QualifiedName::new(format!("App.S{id}")).unwrap(),
            kind: SymbolKind::Method,
        }
    }

    impl UsageIndex for Usages {
        fn usages(
            &self,
            snapshot: SnapshotId,
            query: &UsageQuery,
            limit: u32,
        ) -> Result<UsagePage, CodedError> {
            let symbol = query.symbol.get();
            self.asked
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push((snapshot, symbol, query.depth));
            let all = self.of.get(&symbol).cloned().unwrap_or_default();
            let entries = all
                .iter()
                .take(limit as usize)
                .map(|(from, path)| UsageEntry {
                    depth: 1,
                    reference: Reference {
                        id: ReferenceId::new(1),
                        from: SymbolId::new(*from),
                        to: query.symbol,
                        file: FileId::new(1),
                        span: Span::new(
                            Position { line: 0, column: 0 },
                            Position { line: 0, column: 1 },
                        )
                        .unwrap(),
                        kind: ReferenceKind::Call,
                        confidence: Confidence::Certain,
                        verified: false,
                    },
                    path: RelativePath::new(*path).unwrap(),
                    from: label(*from),
                    to: label(symbol),
                })
                .collect();
            Ok(UsagePage {
                total: all.len(),
                entries,
            })
        }

        fn impact(
            &self,
            _: SnapshotId,
            _: SymbolId,
            _: u32,
            _: u32,
        ) -> Result<codeskimr_application::ports::ImpactPage, CodedError> {
            Err(CodedError::new(ErrorCode::UnexpectedInternalError))
        }

        fn dependencies(
            &self,
            _: SnapshotId,
            _: &codeskimr_application::ports::DependencyQuery,
            _: u32,
        ) -> Result<codeskimr_application::ports::DependencyPage, CodedError> {
            Err(CodedError::new(ErrorCode::UnexpectedInternalError))
        }
    }

    struct World {
        index: Index,
        source: Source,
        usages: Usages,
        annotations: Arc<MemoryAnnotations>,
    }

    fn index() -> Index {
        let mut index = Index {
            history: vec![entry(NEW), entry(MID), entry(OLD)],
            ..Index::default()
        };
        index
            .edges
            .insert(MID, vec![edge("Ui", "Domain", false, false)]);
        index.edges.insert(
            NEW,
            vec![
                edge("Ui", "Domain", false, false),
                edge("Domain", "Ui", true, true),
            ],
        );
        index
    }

    fn source() -> Source {
        let base = vec![
            fact(1, "App.Gone", "void Gone()", "g"),
            fact(2, "App.Sig", "int Sig()", "s"),
            fact(3, "App.Body", "void Body()", "b"),
            fact(4, "App.Caller", "void Caller()", "c"),
            fact(5, "App.Adapter", "void Adapter()", "a"),
        ];
        let target = vec![
            fact(12, "App.Sig", "long Sig()", "s2"),
            fact(13, "App.Body", "void Body()", "b2"),
            fact(14, "App.Caller", "void Caller()", "c"),
            fact(15, "App.Adapter", "void Adapter()", "a2"),
            fact(16, "App.Fresh", "void Fresh()", "f"),
        ];
        let mut source = Source::default();
        source
            .facts
            .insert(OLD, vec![fact(100, "App.Old", "void Old()", "o")]);
        source.facts.insert(MID, base);
        source.facts.insert(NEW, target);
        for id in [OLD, MID, NEW] {
            source.rules.insert(id, "rules-1".to_owned());
        }
        source
    }

    /// Three retained scans. The base (20) has `Gone`, `Sig`, `Body` and the callers `Caller`
    /// (untouched) and `Adapter` (changed itself); the target (30) lost `Gone`, widened `Sig`,
    /// rewrote `Body` and `Adapter` and added `Fresh`.
    fn world() -> World {
        let mut usages = Usages::default();
        usages.of.insert(1, vec![(4, "src/c.cs"), (5, "src/d.cs")]);
        usages
            .of
            .insert(2, vec![(4, "src/c.cs"), (5, "src/d.cs"), (4, "src/e.cs")]);
        World {
            index: index(),
            source: source(),
            usages,
            annotations: Arc::new(MemoryAnnotations::default()),
        }
    }

    fn comparer(world: &World, limits: CompareLimits) -> ScanComparer {
        let ports = ComparePorts {
            index: Arc::new(Index {
                reads: Arc::default(),
                ..world.index.clone()
            }),
            source: Arc::new(world.source.clone()),
            usages: Arc::new(world.usages.clone()),
            annotations: world.annotations.clone(),
        };
        ScanComparer::new(ports, limits)
    }

    const LIMITS: CompareLimits = CompareLimits {
        max_symbols: 100,
        max_usages: 10,
    };

    fn ask(
        base: Option<SnapshotId>,
        target: Option<SnapshotId>,
        impact: bool,
    ) -> ComparisonRequest {
        ComparisonRequest {
            project: PROJECT,
            base,
            target,
            with_impact: impact,
        }
    }

    fn names(facts: &[SymbolFact]) -> Vec<&str> {
        facts
            .iter()
            .map(|fact| fact.qualified_name.as_str())
            .collect()
    }

    #[test]
    fn the_default_compares_the_scan_before_the_newest_with_the_newest() {
        let answer = comparer(&world(), LIMITS)
            .compare(&ask(None, None, false))
            .unwrap();
        assert_eq!((answer.base.snapshot, answer.target.snapshot), (MID, NEW));
        assert_eq!(names(&answer.removed.items), vec!["App.Gone"]);
        assert_eq!(names(&answer.added.items), vec!["App.Fresh"]);
        let changed: Vec<&str> = answer
            .changed
            .items
            .iter()
            .map(|matched| matched.target.qualified_name.as_str())
            .collect();
        assert_eq!(changed, vec!["App.Adapter", "App.Body", "App.Sig"]);
        assert!(answer.impact.is_none(), "no impact unless asked for");
        assert!(!answer.is_cut());
    }

    #[test]
    fn a_chosen_pair_is_compared_as_named() {
        let answer = comparer(&world(), LIMITS)
            .compare(&ask(Some(OLD), Some(MID), false))
            .unwrap();
        assert_eq!((answer.base.snapshot, answer.target.snapshot), (OLD, MID));
        assert_eq!(answer.removed.total, 1);
        assert_eq!(answer.added.total, 5);
    }

    #[test]
    fn a_snapshot_that_is_not_retained_gives_cmp_201() {
        let gone = SnapshotId::new(5);
        for request in [ask(Some(gone), None, false), ask(None, Some(gone), false)] {
            let error = comparer(&world(), LIMITS).compare(&request).unwrap_err();
            assert_eq!(error.code(), ErrorCode::SnapshotNotFound);
        }
    }

    #[test]
    fn a_project_with_one_scan_has_nothing_to_compare() {
        let mut world = world();
        world.index.history = vec![entry(NEW)];
        let error = comparer(&world, LIMITS)
            .compare(&ask(None, None, false))
            .unwrap_err();
        assert_eq!(error.code(), ErrorCode::SnapshotNotFound);
        world.index.history.clear();
        let error = comparer(&world, LIMITS)
            .compare(&ask(None, None, false))
            .unwrap_err();
        assert_eq!(error.code(), ErrorCode::SnapshotNotFound);
    }

    #[test]
    fn a_base_that_is_not_older_than_the_target_gives_cmp_101() {
        let request = ask(Some(NEW), Some(MID), false);
        let error = comparer(&world(), LIMITS).compare(&request).unwrap_err();
        assert_eq!(error.code(), ErrorCode::ComparisonOrderInvalid);
    }

    /// The history of three scans of `NEW` (the first wrote it, two found nothing changed), `MID`
    /// before them: every scan is an entry of its own (ADR 0093).
    fn world_with_repeated_scans() -> World {
        let mut world = world();
        let again = |scan: u64| ScanEntry {
            scan,
            files_changed: Some(0),
            ..entry(NEW)
        };
        world.index.history = vec![again(103), again(102), entry(NEW), entry(MID), entry(OLD)];
        world
    }

    #[test]
    fn the_default_of_two_scans_in_a_row_without_changes_is_an_empty_comparison() {
        let answer = comparer(&world_with_repeated_scans(), LIMITS)
            .compare(&ask(None, None, true))
            .unwrap();
        assert_eq!((answer.base.scan, answer.target.scan), (102, 103));
        assert_eq!((answer.base.snapshot, answer.target.snapshot), (NEW, NEW));
        assert_eq!(
            answer.added.total + answer.removed.total + answer.changed.total,
            0
        );
        assert_eq!(answer.layers.new_violations.len(), 0);
    }

    #[test]
    fn a_snapshot_named_twice_is_compared_with_itself_from_its_first_to_its_last_scan() {
        let answer = comparer(&world_with_repeated_scans(), LIMITS)
            .compare(&ask(Some(NEW), Some(NEW), false))
            .unwrap();
        assert_eq!((answer.base.scan, answer.target.scan), (30, 103));
        assert_eq!(answer.changed.total, 0);
    }

    #[test]
    fn a_named_snapshot_stands_for_its_newest_scan() {
        let answer = comparer(&world_with_repeated_scans(), LIMITS)
            .compare(&ask(Some(MID), None, false))
            .unwrap();
        assert_eq!((answer.base.snapshot, answer.target.scan), (MID, 103));
    }

    #[test]
    fn different_naming_rules_ask_for_a_full_scan() {
        let mut world = world();
        world.source.rules.insert(NEW, "rules-2".to_owned());
        let error = comparer(&world, LIMITS)
            .compare(&ask(None, None, false))
            .unwrap_err();
        assert_eq!(error.code(), ErrorCode::NamingRulesDiffer);
        assert!(error.code().retryable());
        world.source.rules.remove(&NEW);
        let error = comparer(&world, LIMITS)
            .compare(&ask(None, None, false))
            .unwrap_err();
        assert_eq!(
            error.code(),
            ErrorCode::NamingRulesDiffer,
            "a record against none"
        );
        world.source.rules.remove(&MID);
        assert!(
            comparer(&world, LIMITS)
                .compare(&ask(None, None, false))
                .is_ok()
        );
    }

    #[test]
    fn a_scan_that_retention_removes_while_reading_is_reported() {
        let world = world();
        let mut world = world;
        world.index.vanish_after = Some(1);
        let error = comparer(&world, LIMITS)
            .compare(&ask(None, None, false))
            .unwrap_err();
        assert_eq!(error.code(), ErrorCode::SnapshotNotFound);
    }

    #[test]
    fn usages_are_adapted_when_the_symbol_around_them_is_touched() {
        let answer = comparer(&world(), LIMITS)
            .compare(&ask(None, None, true))
            .unwrap();
        let impact = answer.impact.unwrap();
        let subjects: Vec<&str> = impact
            .symbols
            .items
            .iter()
            .map(|one| one.subject.symbol.qualified_name.as_str())
            .collect();
        assert_eq!(
            subjects,
            vec!["App.Gone", "App.Sig"],
            "content changes have none"
        );
        let gone = &impact.symbols.items[0];
        let status: Vec<UsageStatus> = gone.usages.items.iter().map(|usage| usage.status).collect();
        assert_eq!(status, vec![UsageStatus::NotTouched, UsageStatus::Adapted]);
        assert_eq!(gone.not_touched(), 1);
        let sig = &impact.symbols.items[1];
        assert_eq!(sig.usages.total, 3);
        assert_eq!(sig.not_touched(), 2);
    }

    #[test]
    fn the_usages_are_read_in_the_base_scan_one_step_deep() {
        let world = world();
        comparer(&world, LIMITS)
            .compare(&ask(Some(MID), Some(NEW), true))
            .unwrap();
        let asked = world.usages.asked.lock().unwrap().clone();
        assert_eq!(asked, vec![(MID, 1, 1), (MID, 2, 1)]);
    }

    #[test]
    fn the_lists_say_when_they_are_cut() {
        let limits = CompareLimits {
            max_symbols: 1,
            max_usages: 1,
        };
        let answer = comparer(&world(), limits)
            .compare(&ask(None, None, true))
            .unwrap();
        assert_eq!(answer.changed.total, 3);
        assert_eq!(answer.changed.items.len(), 1);
        assert!(answer.changed.is_cut() && answer.is_cut());
        let impact = answer.impact.unwrap();
        assert_eq!((impact.symbols.total, impact.symbols.items.len()), (2, 1));
        let gone = &impact.symbols.items[0];
        assert_eq!((gone.usages.total, gone.usages.items.len()), (2, 1));
    }

    #[test]
    fn new_layer_violations_and_cycles_are_those_the_base_did_not_have() {
        let answer = comparer(&world(), LIMITS)
            .compare(&ask(None, None, false))
            .unwrap();
        assert_eq!(answer.layers.new_violations.len(), 1);
        assert_eq!(answer.layers.new_violations[0].from.as_str(), "Domain");
        assert_eq!(answer.layers.new_cycles.len(), 1);
        let reverse = comparer(&world(), LIMITS).compare(&ask(Some(MID), Some(OLD), false));
        assert!(reverse.is_err(), "older as target is refused");
        let mut same = world();
        same.index.edges.insert(MID, same.index.edges[&NEW].clone());
        let answer = comparer(&same, LIMITS)
            .compare(&ask(None, None, false))
            .unwrap();
        assert!(answer.layers.new_violations.is_empty() && answer.layers.new_cycles.is_empty());
    }

    #[test]
    fn critical_symbols_are_named_when_a_change_touches_them() {
        let world = world();
        world
            .annotations
            .critical(SymbolKind::Method, "App.Gone", &[]);
        world
            .annotations
            .critical(SymbolKind::Method, "App.Body", &[]);
        world
            .annotations
            .critical(SymbolKind::Method, "App.Caller", &[]);
        let answer = comparer(&world, LIMITS)
            .compare(&ask(None, None, false))
            .unwrap();
        let named: Vec<(&str, CriticalOutcome)> = answer
            .critical
            .iter()
            .map(|change| (change.name.as_str(), change.outcome))
            .collect();
        assert_eq!(
            named,
            vec![
                ("App.Gone", CriticalOutcome::Removed),
                ("App.Body", CriticalOutcome::Changed)
            ]
        );
    }
}
