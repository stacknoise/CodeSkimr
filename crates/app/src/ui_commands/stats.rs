//! The commands of the statistics screen (ADR 0081): the overview of a project and its rankings,
//! the most used symbols, the ones no usage was found for, the most complex, the largest and the
//! most coupled. Every row names the symbol, so the screen can open it in the explorer.
//!
//! A usage count is always the pair "certain" and "all": without level 2 most references are
//! only probable, so "all" is an upper bound (ADR 0033, ADR 0037). "No usage found" never means
//! that a symbol is dead; every such row carries an assessment.

use std::sync::Arc;

use codeskimr_application::access;
use codeskimr_application::config::{MetricThresholds, UnusedIgnore};
use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{
    CodeIndex, CodeStatistics, ProjectRepository, RankedRows, RankingRequest, StatisticsSetup,
    UnusedCounts,
};
use codeskimr_application::registry::{Command, CommandDescriptor, CommandId, Request};
use codeskimr_domain::{
    Assessment, KindGroup, LanguageId, Permission, ProjectId, Ranking, RankingQuery, Snapshot,
    SymbolFacts, SymbolId, UnusedReason, UsageOrder, Visibility,
};
use serde::{Deserialize, Serialize};

use super::consts::{
    STATS_LIMIT, STATS_LIMIT_MAX, STATS_OVERVIEW, STATS_OVERVIEW_DESCRIPTION, STATS_OVERVIEW_TITLE,
    STATS_RANKING, STATS_RANKING_DESCRIPTION, STATS_RANKING_TITLE, STATS_TOP,
};
use super::rating;
use super::unused::{self, Judged};

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
    /// For a symbol without usages: why it is judged so, `private`, `internal`, `public` or
    /// `approximate`; `None` otherwise (ADR 0101).
    pub reason: Option<&'static str>,
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
    /// Symbols without usages that the exception list covers; they are not in the counts above.
    pub left_out: u32,
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
    /// For the ranking `unused`: the symbols the exception list of the project covers, with the
    /// entry that does; they are not in `rows`, `total` and `unused` (ADR 0101).
    pub left_out: Vec<LeftOutOut>,
}

/// A symbol without usages that an entry of the exception list covers.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LeftOutOut {
    /// The symbol.
    pub row: RowOut,
    /// The entry of `.codeskimr.toml` that covers it.
    pub rule: UnusedIgnore,
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

impl StatsPorts {
    /// The newest snapshot of the project; `None` before the first scan.
    ///
    /// # Errors
    /// `PRJ-201` for an unknown project, a coded `STO` failure.
    fn latest(&self, project: u64) -> Result<Option<Snapshot>, CodedError> {
        let id = ProjectId::new(project);
        self.projects.project(id)?;
        self.index.latest_snapshot(id)
    }

    fn setup(&self) -> StatisticsSetup<'_> {
        StatisticsSetup {
            test_names: &self.test_names,
            approximate: &self.approximate,
        }
    }

    /// The first rows of a ranking of the snapshot, and how many rows it holds.
    fn ranked(
        &self,
        snapshot: &Snapshot,
        query: &RankingQuery,
        assessment: Option<Assessment>,
        limit: usize,
    ) -> Result<RankedRows, CodedError> {
        self.statistics.ranked(&RankingRequest {
            snapshot: snapshot.id,
            query,
            assessment,
            limit,
            setup: self.setup(),
        })
    }

    fn is_approximate(&self, fact: &SymbolFacts) -> bool {
        self.approximate
            .iter()
            .any(|language| language == fact.language.as_str())
    }

    fn assess(&self, fact: &SymbolFacts) -> Assessment {
        Assessment::of(fact.visibility, self.is_approximate(fact))
    }

    fn reason(&self, fact: &SymbolFacts) -> &'static str {
        UnusedReason::of(fact.visibility, self.is_approximate(fact)).label()
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
            reason: (fact.usage.total == 0).then(|| self.reason(fact)),
        }
    }

    fn rows(&self, facts: &[SymbolFacts]) -> Vec<RowOut> {
        facts.iter().map(|fact| self.row(fact)).collect()
    }
}

impl StatsPorts {
    /// The exception list of the project (ADR 0101); empty if it has none.
    fn rules(&self, project: u64) -> Result<Vec<UnusedIgnore>, CodedError> {
        let project = self.projects.project(ProjectId::new(project))?;
        unused::rules_of(&project)
    }

    /// The symbols without usages of the snapshot, split by the exception list.
    fn judged(
        &self,
        snapshot: &Snapshot,
        query: &RankingQuery,
        rules: &[UnusedIgnore],
    ) -> Result<Judged, CodedError> {
        let facts = self.statistics.symbol_facts(snapshot.id)?;
        Ok(unused::judge(
            facts,
            query,
            rules,
            &self.approximate,
            &self.test_names,
        ))
    }

    /// The counts per assessment of the symbols that stay in the ranking.
    fn counts_of(&self, judged: &Judged) -> UnusedOut {
        let mut counts = UnusedOut {
            tests_hidden: count(judged.tests_hidden),
            left_out: count(judged.left_out.len()),
            ..UnusedOut::default()
        };
        for fact in &judged.kept {
            let slot = match self.assess(fact) {
                Assessment::Likely => &mut counts.likely,
                Assessment::Public => &mut counts.public,
                Assessment::Approximate => &mut counts.approximate,
            };
            *slot = slot.saturating_add(1);
        }
        counts
    }

    /// The ranking of unused symbols from the judged facts: the first rows of the wanted
    /// assessment, and the symbols the list leaves out.
    fn judged_ranking(
        &self,
        judged: &Judged,
        assessment: Option<Assessment>,
        limit: usize,
    ) -> RankingOut {
        let kept: Vec<&SymbolFacts> = judged
            .kept
            .iter()
            .filter(|fact| assessment.is_none_or(|wanted| self.assess(fact) == wanted))
            .collect();
        let rows: Vec<RowOut> = kept.iter().take(limit).map(|fact| self.row(fact)).collect();
        let left_out = judged
            .left_out
            .iter()
            .take(limit)
            .map(|(fact, rule)| LeftOutOut {
                row: self.row(fact),
                rule: rule.clone(),
            })
            .collect();
        RankingOut {
            snapshot: None,
            rows,
            total: count(kept.len()),
            unused: Some(self.counts_of(judged)),
            left_out,
        }
    }
}

fn unused_out(counts: UnusedCounts) -> UnusedOut {
    UnusedOut {
        likely: counts.likely,
        public: counts.public,
        approximate: counts.approximate,
        tests_hidden: counts.tests_hidden,
        left_out: 0,
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
        snapshot: &Snapshot,
        query: &RankingQuery,
        input: &RankingInput,
    ) -> Result<RankingOut, CodedError> {
        let ports = &self.ports;
        let rules = ports.rules(input.project)?;
        if !rules.is_empty() {
            let judged = ports.judged(snapshot, query, &rules)?;
            let wanted = input.assessment.map(Assessment::from);
            return Ok(ports.judged_ranking(&judged, wanted, limit_of(input)));
        }
        let counts = ports
            .statistics
            .unused_counts(snapshot.id, query, ports.setup())?;
        let ranked = ports.ranked(
            snapshot,
            query,
            input.assessment.map(Assessment::from),
            limit_of(input),
        )?;
        Ok(RankingOut {
            snapshot: None,
            rows: ports.rows(&ranked.rows),
            total: count(ranked.total),
            unused: Some(unused_out(counts)),
            left_out: Vec::new(),
        })
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
        let Some(snapshot) = self.ports.latest(input.project)? else {
            return Ok(RankingOut::default());
        };
        let mut out = if query.ranking == Ranking::Unused {
            self.unused_ranking(&snapshot, &query, &input)?
        } else {
            let ranked = self
                .ports
                .ranked(&snapshot, &query, None, limit_of(&input))?;
            RankingOut {
                snapshot: None,
                rows: self.ports.rows(&ranked.rows),
                total: count(ranked.total),
                unused: None,
                left_out: Vec::new(),
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

impl StatsOverview {
    fn complexity_out(&self, histogram: Vec<(u32, u32)>) -> ComplexityOut {
        let limit = self.ports.thresholds.cyclomatic_medium_max;
        let histogram: Vec<BarOut> = histogram
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

    /// The symbols without usages of the overview: the counts and the first rows. A project
    /// with entries in the exception list (ADR 0101) is judged once from the facts of its
    /// snapshot, the others from the stored numbers.
    fn unused_out(
        &self,
        snapshot: &Snapshot,
        all: &RankingQuery,
        project: u64,
    ) -> Result<(UnusedOut, Vec<RowOut>), CodedError> {
        let ports = &self.ports;
        let top = usize::try_from(STATS_TOP).unwrap_or(usize::MAX);
        let rules = ports.rules(project)?;
        if rules.is_empty() {
            let counts = ports
                .statistics
                .unused_counts(snapshot.id, all, ports.setup())?;
            let found = ports.ranked(snapshot, all, None, top)?;
            return Ok((unused_out(counts), ports.rows(&found.rows)));
        }
        let judged = ports.judged(snapshot, all, &rules)?;
        let rows = judged.kept.iter().take(top).map(|fact| ports.row(fact));
        Ok((ports.counts_of(&judged), rows.collect()))
    }

    fn top_out(&self, snapshot: &Snapshot, tests: bool) -> Result<TopOut, CodedError> {
        let ports = &self.ports;
        let top = usize::try_from(STATS_TOP).unwrap_or(usize::MAX);
        let first = |ranking: Ranking, kinds: KindGroup| -> Result<Vec<RowOut>, CodedError> {
            let found = ports.ranked(snapshot, &query(ranking, kinds, tests), None, top)?;
            Ok(ports.rows(&found.rows))
        };
        Ok(TopOut {
            most_used: first(Ranking::MostUsed, KindGroup::Types)?,
            unused: Vec::new(),
            complexity: first(Ranking::Complexity, KindGroup::Functions)?,
            coupling: first(Ranking::Coupling, KindGroup::Types)?,
        })
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
        let Some(snapshot) = self.ports.latest(request.input.project)? else {
            return Ok(OverviewOut::default());
        };
        let ports = &self.ports;
        let all = query(Ranking::Unused, KindGroup::All, tests);
        let (unused, unused_top) = self.unused_out(&snapshot, &all, request.input.project)?;
        let sums = ports
            .statistics
            .summary(snapshot.id, tests, ports.setup())?;
        let mut out = OverviewOut {
            snapshot: Some(snapshot.id.get()),
            files: snapshot.stats.files,
            symbols: snapshot.stats.symbols,
            ranked: sums.ranked,
            references: ReferencesOut {
                certain: sums.certain_references,
                probable: sums.probable_references,
            },
            unused,
            complexity: self.complexity_out(sums.histogram),
            top: self.top_out(&snapshot, tests)?,
        };
        out.top.unused = unused_top;
        Ok(out)
    }
}
