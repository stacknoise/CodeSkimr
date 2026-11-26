//! Tests of the rankings of the statistics screen (ADR 0081).

use codeskimr_domain::{
    Assessment, Confidence, FileId, KindGroup, LanguageId, Measures, ModelError, Position,
    QualifiedName, Ranking, RankingQuery, Reference, ReferenceId, ReferenceKind, RelativePath,
    ReverseIndex, Span, SymbolFacts, SymbolId, SymbolKind, SymbolName, UnusedReason, UsageOrder,
    UsageTally, Visibility, complexity_histogram, mark_tests, rank,
};

fn fact(id: u64, kind: SymbolKind, name: &str, path: &str) -> Result<SymbolFacts, ModelError> {
    Ok(SymbolFacts {
        id: SymbolId::new(id),
        name: SymbolName::new(name)?,
        qualified_name: QualifiedName::new(name)?,
        kind,
        parent: None,
        visibility: None,
        path: RelativePath::new(path)?,
        language: LanguageId::new("rust")?,
        test: false,
        overrides: false,
        usage: UsageTally::default(),
        measures: Measures::default(),
    })
}

fn used(mut fact: SymbolFacts, certain: u32, total: u32) -> SymbolFacts {
    fact.usage = UsageTally { certain, total };
    fact
}

fn query(ranking: Ranking) -> RankingQuery {
    RankingQuery {
        ranking,
        kinds: KindGroup::All,
        language: None,
        include_tests: false,
        order: UsageOrder::Certain,
    }
}

fn names(facts: &[SymbolFacts], query: &RankingQuery) -> Vec<String> {
    rank(facts, query, usize::MAX)
        .rows
        .iter()
        .map(|fact| fact.name.to_string())
        .collect()
}

fn reference(id: u64, ends: (u64, u64), kind: ReferenceKind) -> Result<Reference, ModelError> {
    let at = Position { line: 0, column: 0 };
    Ok(Reference {
        id: ReferenceId::new(id),
        from: SymbolId::new(ends.0),
        to: SymbolId::new(ends.1),
        file: FileId::new(ends.0),
        span: Span::new(at, at)?,
        kind,
        confidence: Confidence::Certain,
        verified: false,
    })
}

#[test]
fn the_most_used_are_sorted_by_certain_usages_or_by_the_upper_bound() -> Result<(), ModelError> {
    let facts = [
        used(fact(1, SymbolKind::Function, "collect", "a.rs")?, 1, 800),
        used(fact(2, SymbolKind::Struct, "Failure", "b.rs")?, 28, 73),
        fact(3, SymbolKind::Function, "idle", "c.rs")?,
        used(fact(4, SymbolKind::Field, "count", "d.rs")?, 99, 99),
    ];
    let mut wanted = query(Ranking::MostUsed);
    assert_eq!(names(&facts, &wanted), ["Failure", "collect"]);
    wanted.order = UsageOrder::All;
    assert_eq!(names(&facts, &wanted), ["collect", "Failure"]);
    wanted.kinds = KindGroup::Types;
    assert_eq!(names(&facts, &wanted), ["Failure"]);
    Ok(())
}

#[test]
fn unused_symbols_are_those_without_any_usage_in_a_fixed_order() -> Result<(), ModelError> {
    let facts = [
        fact(1, SymbolKind::Function, "zeta", "b.rs")?,
        used(fact(2, SymbolKind::Function, "busy", "a.rs")?, 0, 1),
        fact(3, SymbolKind::Function, "alpha", "b.rs")?,
        fact(4, SymbolKind::Function, "first", "a.rs")?,
    ];
    let ranked = rank(&facts, &query(Ranking::Unused), 2);
    let found: Vec<&str> = ranked.rows.iter().map(|fact| fact.name.as_str()).collect();
    assert_eq!(found, ["first", "alpha"]);
    assert_eq!(ranked.total, 3);
    Ok(())
}

#[test]
fn rankings_by_a_measure_leave_out_what_was_not_measured() -> Result<(), ModelError> {
    let mut simple = fact(1, SymbolKind::Function, "simple", "a.rs")?;
    simple.measures.complexity = Some(1.0);
    let mut hard = fact(2, SymbolKind::Method, "hard", "a.rs")?;
    hard.measures.complexity = Some(22.0);
    let mut module = fact(3, SymbolKind::Struct, "Big", "a.rs")?;
    module.measures.complexity = Some(40.0);
    module.measures.loc = Some(300.0);
    let facts = [
        simple,
        hard,
        module,
        fact(4, SymbolKind::Function, "bare", "a.rs")?,
    ];
    assert_eq!(
        names(&facts, &query(Ranking::Complexity)),
        ["hard", "simple"]
    );
    assert_eq!(names(&facts, &query(Ranking::Size)), ["Big"]);
    assert_eq!(complexity_histogram(&facts, false), [(1, 1), (22, 1)]);
    Ok(())
}

#[test]
fn constructors_and_overriding_members_are_never_listed_as_unused() -> Result<(), ModelError> {
    let mut overriding = fact(2, SymbolKind::Method, "Draw", "b.cs")?;
    overriding.overrides = true;
    let facts = [
        fact(1, SymbolKind::Constructor, "Shape", "a.cs")?,
        overriding,
        fact(3, SymbolKind::Method, "Orphan", "c.cs")?,
    ];
    let wanted = query(Ranking::Unused);
    assert_eq!(names(&facts, &wanted), ["Orphan"]);
    assert_eq!(rank(&facts, &wanted, usize::MAX).total, 1);
    Ok(())
}

/// Marks the test code among `facts` by the name `tests` and tells which facts are marked.
fn marked(facts: &mut [SymbolFacts]) -> Vec<bool> {
    mark_tests(facts, &["tests".to_owned()]);
    facts.iter().map(|fact| fact.test).collect()
}

#[test]
fn test_code_is_marked_by_folder_and_by_the_symbol_around_it() -> Result<(), ModelError> {
    let mut inner = fact(2, SymbolKind::Function, "checks", "src/lib.rs")?;
    inner.parent = Some(SymbolId::new(1));
    let mut facts = [
        fact(1, SymbolKind::Module, "tests", "src/lib.rs")?,
        inner,
        fact(3, SymbolKind::Function, "helper", "crates/x/tests/it.rs")?,
        fact(4, SymbolKind::Function, "real", "src/lib.rs")?,
    ];
    assert_eq!(marked(&mut facts), [true, true, true, false]);
    let mut wanted = query(Ranking::Unused);
    assert_eq!(rank(&facts, &wanted, usize::MAX).total, 1);
    wanted.include_tests = true;
    assert_eq!(rank(&facts, &wanted, usize::MAX).total, 3);
    Ok(())
}

#[test]
fn a_module_or_file_named_after_the_tests_is_test_code() -> Result<(), ModelError> {
    let mut inner = fact(2, SymbolKind::Function, "checks", "src/env.rs")?;
    inner.parent = Some(SymbolId::new(1));
    let mut facts = [
        fact(1, SymbolKind::Module, "search_tests", "src/env.rs")?,
        inner,
        fact(3, SymbolKind::Function, "case", "src/stats_tests.rs")?,
        fact(4, SymbolKind::Function, "contests", "src/contests.rs")?,
        fact(5, SymbolKind::Function, "run_tests", "src/runner.rs")?,
    ];
    assert_eq!(marked(&mut facts), [true, true, true, false, false]);
    Ok(())
}

#[test]
fn a_language_filter_keeps_only_that_language() -> Result<(), ModelError> {
    let mut other = fact(2, SymbolKind::Function, "other", "a.py")?;
    other.language = LanguageId::new("python")?;
    let facts = [fact(1, SymbolKind::Function, "mine", "a.rs")?, other];
    let mut wanted = query(Ranking::Unused);
    wanted.language = Some(LanguageId::new("python")?);
    assert_eq!(names(&facts, &wanted), ["other"]);
    Ok(())
}

#[test]
fn the_assessment_depends_on_visibility_and_on_the_language() {
    assert_eq!(
        Assessment::of(Some(Visibility::Private), false),
        Assessment::Likely
    );
    assert_eq!(
        Assessment::of(Some(Visibility::Public), false),
        Assessment::Public
    );
    assert_eq!(Assessment::of(None, false), Assessment::Public);
    assert_eq!(
        Assessment::of(Some(Visibility::Private), true),
        Assessment::Approximate
    );
}

#[test]
fn the_reason_agrees_with_the_assessment() {
    let cases = [
        (Some(Visibility::Private), false, UnusedReason::Private),
        (Some(Visibility::Internal), false, UnusedReason::Internal),
        (Some(Visibility::Protected), false, UnusedReason::Internal),
        (Some(Visibility::Public), false, UnusedReason::Public),
        (None, false, UnusedReason::Public),
        (Some(Visibility::Private), true, UnusedReason::Approximate),
    ];
    for (visibility, approximate, reason) in cases {
        assert_eq!(UnusedReason::of(visibility, approximate), reason);
    }
    assert_eq!(
        "internal".parse::<UnusedReason>(),
        Ok(UnusedReason::Internal)
    );
}

#[test]
fn tallies_leave_out_tests_and_self_references_and_split_certain_from_probable()
-> Result<(), ModelError> {
    let mut probable = reference(2, (1, 3), ReferenceKind::Call)?;
    probable.confidence = Confidence::Probable;
    let mut checked = reference(3, (4, 3), ReferenceKind::Call)?;
    checked.confidence = Confidence::Probable;
    checked.verified = true;
    let index = ReverseIndex::new([
        reference(1, (1, 3), ReferenceKind::Call)?,
        probable,
        checked,
        reference(4, (3, 3), ReferenceKind::Call)?,
        reference(5, (9, 3), ReferenceKind::Test)?,
    ]);
    let tallies = index.tallies();
    let tally = tallies.get(&SymbolId::new(3)).copied().unwrap_or_default();
    assert_eq!((tally.certain, tally.total, tally.probable()), (2, 3, 1));
    assert!(index.test_files().contains(&FileId::new(9)));
    assert_eq!(index.test_files().len(), 1);
    Ok(())
}
