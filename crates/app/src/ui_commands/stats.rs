//! The commands of the statistics screen (ADR 0081): the overview of a project and its rankings,
//! the most used symbols, the ones no usage was found for, the most complex, the largest and the
//! most coupled. Every row names the symbol, so the screen can open it in the explorer.
//!
//! A usage count is always the pair "certain" and "all": without level 2 most references are
//! only probable, so "all" is an upper bound (ADR 0033, ADR 0037). "No usage found" never means
//! that a symbol is dead; every such row carries an assessment.

use std::sync::Arc;

use codeskimr_application::access;
use codeskimr_application::config::MetricThresholds;
use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{CodeIndex, CodeStatistics, ProjectRepository};
use codeskimr_application::registry::{Command, CommandDescriptor, CommandId, Request};
use codeskimr_domain::{
    Assessment, KindGroup, LanguageId, Permission, ProjectId, Ranking, RankingQuery, Snapshot,
    SymbolFacts, SymbolId, UsageOrder, UsageTally, Visibility, complexity_histogram, rank,
};
use serde::{Deserialize, Serialize};

use super::consts::{
    STATS_LIMIT, STATS_LIMIT_MAX, STATS_OVERVIEW, STATS_OVERVIEW_DESCRIPTION, STATS_OVERVIEW_TITLE,
    STATS_RANKING, STATS_RANKING_DESCRIPTION, STATS_RANKING_TITLE, STATS_TOP,
};
use super::rating;

/// What a ranking sorts by, as the screen names it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RankingName {
    /// The symbols with the most usages.
    MostUsed,
    /// The symbols no usage was found for.
    Unused,
    /// The most complex callables.
    Complexity,
    /// The symbols with the most lines of code.
    Size,
    /// The symbols that depend on the most others.
    Coupling,
}

impl From<RankingName> for Ranking {
    fn from(name: RankingName) -> Self {
        match name {
            RankingName::MostUsed => Self::MostUsed,
            RankingName::Unused => Self::Unused,
            RankingName::Complexity => Self::Complexity,
            RankingName::Size => Self::Size,
            RankingName::Coupling => Self::Coupling,
        }
    }
}

/// Which kinds of symbols a ranking holds, as the screen names them.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kinds {
    /// Types and callables.
    #[default]
    All,
    /// Types only.
    Types,
    /// Callables only.
    Functions,
}

impl From<Kinds> for KindGroup {
    fn from(kinds: Kinds) -> Self {
        match kinds {
            Kinds::All => Self::All,
            Kinds::Types => Self::Types,
            Kinds::Functions => Self::Functions,
        }
    }
}

/// Which count the most used symbols are sorted by.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Order {
    /// By the certain usages.
    #[default]
    Certain,
    /// By all usages, the upper bound.
    All,
}

impl From<Order> for UsageOrder {
    fn from(order: Order) -> Self {
        match order {
            Order::Certain => Self::Certain,
            Order::All => Self::All,
        }
    }
}

/// An assessment of "no usage found", as the screen filters by it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssessmentName {
    /// Very likely unused.
    Likely,
    /// Public: code outside the project may use it.
    Public,
    /// In a language whose usages are an approximation.
    Approximate,
}

impl From<AssessmentName> for Assessment {
    fn from(name: AssessmentName) -> Self {
        match name {
            AssessmentName::Likely => Self::Likely,
            AssessmentName::Public => Self::Public,
            AssessmentName::Approximate => Self::Approximate,
        }
    }
}

/// The input of `stats.overview`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct OverviewInput {
    /// Id of the project.
    pub project: u64,
    /// Whether test code counts; it does not if left out.
    #[serde(default)]
    pub include_tests: bool,
}

/// The input of `stats.ranking`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RankingInput {
    /// Id of the project.
    pub project: u64,
    /// What to sort by.
    pub ranking: RankingName,
    /// The kinds of symbols; types and callables if left out.
    #[serde(default)]
    pub kinds: Kinds,
    /// Only symbols of this language (id of the language pack); all if left out.
    #[serde(default)]
    pub language: Option<String>,
    /// Whether test code is ranked; it is not if left out.
    #[serde(default)]
    pub include_tests: bool,
    /// Which count the most used symbols are sorted by; the certain usages if left out.
    #[serde(default)]
    pub order: Order,
    /// For the ranking `unused`: only rows with this assessment; all if left out.
    #[serde(default)]
    pub assessment: Option<AssessmentName>,
    /// Most rows of the answer; the default if left out, and never more than the maximum.
    #[serde(default)]
    pub limit: Option<u32>,
}

/// One symbol of a ranking.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RowOut {
    /// Id of the symbol.
    pub id: u64,
    /// Simple name.
    pub name: String,
    /// Qualified name.
    pub qualified_name: String,
    /// Kind label.
    pub kind: &'static str,
    /// File that declares it.
    pub path: String,
    /// Id of the language of that file.
    pub language: String,
    /// The enclosing symbol; `None` for a top-level symbol.
    pub parent: Option<u64>,
    /// Visibility label, if the language tells one.
    pub visibility: Option<&'static str>,
    /// Whether it is test code.
    pub test: bool,
    /// Usages that are certain.
    pub certain: u32,
    /// All usages, the certain and the probable ones: the upper bound.
    pub total: u32,
    /// Cyclomatic complexity, if measured.
    pub complexity: Option<f64>,
    /// `good`, `medium` or `poor` for the complexity of a callable; `None` if not measured.
    pub complexity_rating: Option<&'static str>,
    /// Lines of code, if measured.
    pub loc: Option<f64>,
    /// Efferent coupling, if measured.
    pub efferent: Option<f64>,
    /// Afferent coupling, if measured.
    pub afferent: Option<f64>,
    /// For a symbol without usages: `likely`, `public` or `approximate`; `None` otherwise.
    pub assessment: Option<&'static str>,
}

/// How many symbols without usages there are per assessment.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct UnusedOut {
    /// Not public, in a language with exact usages: very likely unused.
    pub likely: u32,
    /// Public: code outside the project may use it.
    pub public: u32,
    /// In a language whose usages are an approximation.
    pub approximate: u32,
    /// Test code without usages that the answer leaves out; zero if test code counts.
    pub tests_hidden: u32,
}

/// The output of `stats.ranking`.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct RankingOut {
    /// Id of the snapshot the ranking is of; `None` before the first scan.
    pub snapshot: Option<u64>,
    /// The rows, best first.
    pub rows: Vec<RowOut>,
    /// Number of symbols the ranking holds, of which `rows` are the first.
    pub total: u32,
    /// For the ranking `unused`: the rows per assessment, before the filter by assessment.
    pub unused: Option<UnusedOut>,
}

/// How the usages of the snapshot split up.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct ReferencesOut {
    /// References that are certain.
    pub certain: u64,
    /// References that are only probable.
    pub probable: u64,
}

/// One bar of the complexity histogram.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct BarOut {
    /// The cyclomatic complexity.
    pub complexity: u32,
    /// Number of callables with it.
    pub count: u32,
}

/// The complexity of the callables of the snapshot.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct ComplexityOut {
    /// Highest complexity that is not rated poor (`metrics.cyclomatic_medium_max`).
    pub limit: u32,
    /// Number of callables above the limit.
    pub over_limit: u32,
    /// The callables per complexity, by rising complexity.
    pub histogram: Vec<BarOut>,
}

/// The first rows of the rankings, for the cards of the overview.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct TopOut {
    /// The most used types.
    pub most_used: Vec<RowOut>,
    /// Symbols without usages, the very likely unused ones first.
    pub unused: Vec<RowOut>,
    /// The most complex callables.
    pub complexity: Vec<RowOut>,
    /// The types that depend on the most others.
    pub coupling: Vec<RowOut>,
}

/// The output of `stats.overview`.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct OverviewOut {
    /// Id of the snapshot; `None` before the first scan.
    pub snapshot: Option<u64>,
    /// Number of files of the snapshot.
    pub files: u32,
    /// Number of symbols of the snapshot.
    pub symbols: u32,
    /// Number of types and callables the rankings hold.
    pub ranked: u32,
    /// The usages of all symbols.
    pub references: ReferencesOut,
    /// The symbols without usages per assessment.
    pub unused: UnusedOut,
    /// The complexity of the callables.
    pub complexity: ComplexityOut,
    /// The first rows of the rankings.
    pub top: TopOut,
}

/// What the statistics read, and the settings they judge by.
pub struct StatsPorts {
    pub(super) projects: Arc<dyn ProjectRepository>,
    pub(super) index: Arc<dyn CodeIndex>,
    pub(super) statistics: Arc<dyn CodeStatistics>,
    pub(super) thresholds: MetricThresholds,
    /// `conventions.test_folders`: names of folders and modules that hold test code.
    pub(super) test_names: Vec<String>,
    /// `ui.approximate_languages`: languages whose usages are an approximation.
    pub(super) approximate: Vec<String>,
}

fn count(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

/// The newest snapshot of a project with the shared facts of its symbols.
type SnapshotFacts = (Snapshot, Arc<Vec<SymbolFacts>>);

impl StatsPorts {
    /// The newest snapshot of the project and the facts of its symbols, with test code marked;
    /// `None` before the first scan. The facts are shared by all views and loaded once.
    ///
    /// # Errors
    /// `PRJ-201` for an unknown project, a coded `STO` failure.
    fn facts(&self, project: u64) -> Result<Option<SnapshotFacts>, CodedError> {
        let id = ProjectId::new(project);
        self.projects.project(id)?;
        let Some(snapshot) = self.index.latest_snapshot(id)? else {
            return Ok(None);
        };
        let facts = self
            .statistics
            .shared_facts(snapshot.id, &self.test_names)?;
        Ok(Some((snapshot, facts)))
    }

    fn assess(&self, fact: &SymbolFacts) -> Assessment {
        let approximate = self
            .approximate
            .iter()
            .any(|language| language == fact.language.as_str());
        Assessment::of(fact.visibility, approximate)
    }

    fn row(&self, fact: &SymbolFacts) -> RowOut {
        let complexity = fact.measures.complexity.filter(|_| fact.kind.is_callable());
        RowOut {
            id: fact.id.get(),
            name: fact.name.to_string(),
            qualified_name: fact.qualified_name.to_string(),
            kind: fact.kind.label(),
            path: fact.path.to_string(),
            language: fact.language.to_string(),
            parent: fact.parent.map(SymbolId::get),
            visibility: fact.visibility.map(Visibility::label),
            test: fact.test,
            certain: fact.usage.certain,
            total: fact.usage.total,
            complexity,
            complexity_rating: complexity.map(|value| rating::complexity(value, &self.thresholds)),
            loc: fact.measures.loc,
            efferent: fact.measures.efferent,
            afferent: fact.measures.afferent,
            assessment: (fact.usage.total == 0).then(|| self.assess(fact).label()),
        }
    }

    fn rows(&self, facts: &[&SymbolFacts]) -> Vec<RowOut> {
        facts.iter().map(|fact| self.row(fact)).collect()
    }

    /// The symbols without usages per assessment. `shown` are the rows the ranking holds,
    /// `with_tests` the same ranking with test code.
    fn unused_out(&self, shown: &[&SymbolFacts], with_tests: usize) -> UnusedOut {
        let of = |wanted: Assessment| {
            count(
                shown
                    .iter()
                    .filter(|fact| self.assess(fact) == wanted)
                    .count(),
            )
        };
        UnusedOut {
            likely: of(Assessment::Likely),
            public: of(Assessment::Public),
            approximate: of(Assessment::Approximate),
            tests_hidden: count(with_tests.saturating_sub(shown.len())),
        }
    }

    /// The rows without usages, the very likely unused ones first, and their counts.
    fn unused<'a>(
        &self,
        facts: &'a [SymbolFacts],
        query: &RankingQuery,
    ) -> (Vec<&'a SymbolFacts>, UnusedOut) {
        let mut rows = rank(facts, query, usize::MAX).rows;
        rows.sort_by_key(|fact| self.assess(fact));
        let with_tests = RankingQuery {
            include_tests: true,
            ..query.clone()
        };
        let all = rank(facts, &with_tests, usize::MAX).total;
        let counts = self.unused_out(&rows, all);
        (rows, counts)
    }
}

fn query_of(input: &RankingInput) -> Result<RankingQuery, CodedError> {
    let language = input
        .language
        .as_deref()
        .map(LanguageId::new)
        .transpose()
        .map_err(CodedError::caused_by)?;
    Ok(RankingQuery {
        ranking: input.ranking.into(),
        kinds: input.kinds.into(),
        language,
        include_tests: input.include_tests,
        order: input.order.into(),
    })
}

fn limit_of(input: &RankingInput) -> usize {
    let limit = input.limit.unwrap_or(STATS_LIMIT).min(STATS_LIMIT_MAX);
    usize::try_from(limit).unwrap_or(usize::MAX)
}

/// Ranks the symbols of the newest scan of a project.
pub struct StatsRanking {
    pub(super) ports: Arc<StatsPorts>,
}

impl StatsRanking {
    fn unused_ranking(
        &self,
        facts: &[SymbolFacts],
        query: &RankingQuery,
        input: &RankingInput,
    ) -> RankingOut {
        let (mut rows, counts) = self.ports.unused(facts, query);
        if let Some(wanted) = input.assessment.map(Assessment::from) {
            rows.retain(|fact| self.ports.assess(fact) == wanted);
        }
        let total = count(rows.len());
        rows.truncate(limit_of(input));
        RankingOut {
            snapshot: None,
            rows: self.ports.rows(&rows),
            total,
            unused: Some(counts),
        }
    }
}

impl Command for StatsRanking {
    type Input = RankingInput;
    type Output = RankingOut;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(STATS_RANKING),
        title_key: STATS_RANKING_TITLE,
        description_key: STATS_RANKING_DESCRIPTION,
        shortcut: None,
        read_only: true,
        permission: Permission::Read,
    };

    fn execute(&self, request: Request<'_, Self>) -> Result<RankingOut, CodedError> {
        access::require(request.principal, Permission::Read)?;
        let input = request.input;
        let query = query_of(&input)?;
        let Some((snapshot, facts)) = self.ports.facts(input.project)? else {
            return Ok(RankingOut::default());
        };
        let mut out = if query.ranking == Ranking::Unused {
            self.unused_ranking(&facts, &query, &input)
        } else {
            let ranked = rank(&facts, &query, limit_of(&input));
            RankingOut {
                snapshot: None,
                rows: self.ports.rows(&ranked.rows),
                total: count(ranked.total),
                unused: None,
            }
        };
        out.snapshot = Some(snapshot.id.get());
        Ok(out)
    }
}

/// Summarises the newest scan of a project for the cards of the statistics screen.
pub struct StatsOverview {
    pub(super) ports: Arc<StatsPorts>,
}

fn references_out(facts: &[SymbolFacts]) -> ReferencesOut {
    let sum =
        |part: fn(UsageTally) -> u32| facts.iter().map(|fact| u64::from(part(fact.usage))).sum();
    ReferencesOut {
        certain: sum(|tally| tally.certain),
        probable: sum(UsageTally::probable),
    }
}

impl StatsOverview {
    fn complexity_out(&self, facts: &[SymbolFacts], include_tests: bool) -> ComplexityOut {
        let limit = self.ports.thresholds.cyclomatic_medium_max;
        let histogram: Vec<BarOut> = complexity_histogram(facts, include_tests)
            .into_iter()
            .map(|(complexity, count)| BarOut { complexity, count })
            .collect();
        ComplexityOut {
            limit,
            over_limit: histogram
                .iter()
                .filter(|bar| bar.complexity > limit)
                .map(|bar| bar.count)
                .sum(),
            histogram,
        }
    }

    fn top_out(&self, facts: &[SymbolFacts], unused: &[&SymbolFacts], tests: bool) -> TopOut {
        let ports = &self.ports;
        let top = usize::try_from(STATS_TOP).unwrap_or(usize::MAX);
        let first = |ranking: Ranking, kinds: KindGroup| {
            let query = query(ranking, kinds, tests);
            ports.rows(&rank(facts, &query, top).rows)
        };
        TopOut {
            most_used: first(Ranking::MostUsed, KindGroup::Types),
            unused: ports.rows(unused.get(..top.min(unused.len())).unwrap_or_default()),
            complexity: first(Ranking::Complexity, KindGroup::Functions),
            coupling: first(Ranking::Coupling, KindGroup::Types),
        }
    }
}

/// The query of one card of the overview.
const fn query(ranking: Ranking, kinds: KindGroup, include_tests: bool) -> RankingQuery {
    RankingQuery {
        ranking,
        kinds,
        language: None,
        include_tests,
        order: UsageOrder::Certain,
    }
}

impl Command for StatsOverview {
    type Input = OverviewInput;
    type Output = OverviewOut;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(STATS_OVERVIEW),
        title_key: STATS_OVERVIEW_TITLE,
        description_key: STATS_OVERVIEW_DESCRIPTION,
        shortcut: None,
        read_only: true,
        permission: Permission::Read,
    };

    fn execute(&self, request: Request<'_, Self>) -> Result<OverviewOut, CodedError> {
        access::require(request.principal, Permission::Read)?;
        let tests = request.input.include_tests;
        let Some((snapshot, facts)) = self.ports.facts(request.input.project)? else {
            return Ok(OverviewOut::default());
        };
        let all = query(Ranking::Unused, KindGroup::All, tests);
        let (unused_rows, unused) = self.ports.unused(&facts, &all);
        let ranked = facts
            .iter()
            .filter(|fact| KindGroup::All.holds(fact.kind) && (tests || !fact.test))
            .count();
        Ok(OverviewOut {
            snapshot: Some(snapshot.id.get()),
            files: snapshot.stats.files,
            symbols: snapshot.stats.symbols,
            ranked: count(ranked),
            references: references_out(&facts),
            unused,
            complexity: self.complexity_out(&facts, tests),
            top: self.top_out(&facts, &unused_rows, tests),
        })
    }
}
