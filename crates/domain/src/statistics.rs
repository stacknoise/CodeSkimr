//! Rankings over the symbols of one snapshot: the most used, the ones nobody uses, the most
//! complex, the largest and the most coupled (ADR 0081).
//!
//! Everything here is a pure function over [`SymbolFacts`]. A count of usages is always a pair:
//! the references that are certain, and all of them, which is an upper bound as long as level 2
//! did not check them (ADR 0033, ADR 0037).

use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};

use crate::consts::{
    ASSESSMENT_APPROXIMATE, ASSESSMENT_LIKELY, ASSESSMENT_PUBLIC, EXTENSION_SEPARATOR, GROUP_ALL,
    GROUP_FUNCTIONS, GROUP_TYPES, ORDER_ALL, ORDER_CERTAIN, PATH_SEPARATOR, RANKING_COMPLEXITY,
    RANKING_COUPLING, RANKING_MOST_USED, RANKING_SIZE, RANKING_UNUSED, TEST_NAME_JOINT,
};
use crate::file::{LanguageId, RelativePath};
use crate::qualified_name::QualifiedName;
use crate::symbol::{SymbolId, SymbolKind, SymbolName, Visibility};
use crate::vocabulary::labeled_enum;

/// How often a symbol is used by others: references of kind `test` and references of the symbol
/// to itself do not count.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct UsageTally {
    /// References that are certain: resolved without doubt, or verified by level 2.
    pub certain: u32,
    /// All references, the certain and the probable ones: the upper bound.
    pub total: u32,
}

impl UsageTally {
    /// Counts one more reference.
    pub const fn count(&mut self, certain: bool) {
        self.total = self.total.saturating_add(1);
        if certain {
            self.certain = self.certain.saturating_add(1);
        }
    }

    /// References that are only probable.
    #[must_use]
    pub const fn probable(self) -> u32 {
        self.total.saturating_sub(self.certain)
    }
}

/// The measured values of a symbol that the rankings sort by. A value that was not measured is
/// `None`.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Measures {
    /// Cyclomatic complexity.
    pub complexity: Option<f64>,
    /// Lines of code.
    pub loc: Option<f64>,
    /// Maintainability index.
    pub maintainability: Option<f64>,
    /// Efferent coupling: how many others it depends on.
    pub efferent: Option<f64>,
    /// Afferent coupling: how many others depend on it.
    pub afferent: Option<f64>,
}

/// What the statistics know about one symbol of a snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct SymbolFacts {
    /// The symbol.
    pub id: SymbolId,
    /// Its simple name.
    pub name: SymbolName,
    /// Its qualified name.
    pub qualified_name: QualifiedName,
    /// Its kind.
    pub kind: SymbolKind,
    /// The enclosing symbol, `None` for top-level symbols.
    pub parent: Option<SymbolId>,
    /// Its visibility, if the language tells one.
    pub visibility: Option<Visibility>,
    /// File that declares it.
    pub path: RelativePath,
    /// Language of that file.
    pub language: LanguageId,
    /// Whether the symbol is test code, see [`mark_tests`]. The storage sets it for the symbols
    /// of files that the scan knew as test files.
    pub test: bool,
    /// Whether the symbol overrides a member of a base type or implements one of an interface or
    /// trait. Such a member is called through the member it stands in for, so a missing direct
    /// usage says nothing about it.
    pub overrides: bool,
    /// How often it is used.
    pub usage: UsageTally,
    /// Its measured values.
    pub measures: Measures,
}

impl SymbolFacts {
    /// Whether the symbol is used without a direct reference to it: a constructor runs when its
    /// type is built, an overriding member when the overridden one is called. Only direct usages
    /// are counted, so "no usage found" would be wrong for both.
    #[must_use]
    pub const fn is_indirect(&self) -> bool {
        self.overrides || matches!(self.kind, SymbolKind::Constructor)
    }
}

labeled_enum!(
    /// What a ranking sorts by.
    Ranking {
        /// The symbols with the most usages first.
        MostUsed => RANKING_MOST_USED,
        /// The symbols for which no usage was found. Constructors and overriding members are left
        /// out: they are reached through their type and through the member they override.
        Unused => RANKING_UNUSED,
        /// The callables with the highest cyclomatic complexity first.
        Complexity => RANKING_COMPLEXITY,
        /// The symbols with the most lines of code first.
        Size => RANKING_SIZE,
        /// The symbols that depend on the most others first.
        Coupling => RANKING_COUPLING,
    }
);

labeled_enum!(
    /// Which kinds of symbols a ranking holds.
    KindGroup {
        /// Types and callables.
        All => GROUP_ALL,
        /// Classes, interfaces, traits, records, structs, enums, unions and type aliases.
        Types => GROUP_TYPES,
        /// Functions, methods and constructors.
        Functions => GROUP_FUNCTIONS,
    }
);

impl KindGroup {
    /// Whether a symbol of `kind` belongs to the group.
    #[must_use]
    pub const fn holds(self, kind: SymbolKind) -> bool {
        let is_type = kind.is_type() || matches!(kind, SymbolKind::TypeAlias);
        match self {
            Self::All => is_type || kind.is_callable(),
            Self::Types => is_type,
            Self::Functions => kind.is_callable(),
        }
    }
}

labeled_enum!(
    /// Which count the most used symbols are sorted by.
    UsageOrder {
        /// By the certain usages, then by all.
        Certain => ORDER_CERTAIN,
        /// By all usages, the upper bound, then by the certain ones.
        All => ORDER_ALL,
    }
);

labeled_enum!(
    /// How much "no usage found" says about a symbol. It never says that the symbol is dead:
    /// level 1 reads the source text only.
    Assessment {
        /// Not public and in a language whose usages are exact enough: very likely unused.
        Likely => ASSESSMENT_LIKELY,
        /// Public (or without a visibility): code outside the project may use it.
        Public => ASSESSMENT_PUBLIC,
        /// In a language whose usages are an approximation: uncertain.
        Approximate => ASSESSMENT_APPROXIMATE,
    }
);

impl Assessment {
    /// The assessment of an unused symbol with `visibility` in a language whose usages are an
    /// approximation or not.
    #[must_use]
    pub const fn of(visibility: Option<Visibility>, approximate: bool) -> Self {
        if approximate {
            return Self::Approximate;
        }
        match visibility {
            Some(Visibility::Private | Visibility::Internal | Visibility::Protected) => {
                Self::Likely
            }
            Some(Visibility::Public) | None => Self::Public,
        }
    }
}

/// What a ranking is asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RankingQuery {
    /// What to sort by.
    pub ranking: Ranking,
    /// The kinds of symbols to hold.
    pub kinds: KindGroup,
    /// Only symbols of this language, all if `None`.
    pub language: Option<LanguageId>,
    /// Whether test code is ranked as well.
    pub include_tests: bool,
    /// Which count the most used symbols are sorted by.
    pub order: UsageOrder,
}

/// The answer of a ranking: the first rows and how many rows there are in all.
#[derive(Debug, Clone, PartialEq)]
pub struct Ranked<'a> {
    /// The rows, best first, at most as many as asked for.
    pub rows: Vec<&'a SymbolFacts>,
    /// Number of symbols the ranking holds.
    pub total: usize,
}

/// Marks test code: a symbol of a test file (already marked), a symbol below a folder or in a file
/// with one of the `names`, and a symbol inside a module or namespace with one of the `names` (a
/// module `tests` inside a production file). A name that ends with one of the `names` after an
/// underscore counts as well: `search_tests`, `stats_tests.rs`. Functions and types are not judged
/// by their own name, so a production function `run_tests` stays.
pub fn mark_tests(facts: &mut [SymbolFacts], names: &[String]) {
    let chain: HashMap<SymbolId, (Option<SymbolId>, bool)> = facts
        .iter()
        .map(|fact| {
            let is_named = matches!(fact.kind, SymbolKind::Module | SymbolKind::Namespace)
                && is_test_name(fact.name.as_str(), names);
            (fact.id, (fact.parent, is_named))
        })
        .collect();
    for fact in facts.iter_mut() {
        fact.test = fact.test || in_test_folder(&fact.path, names) || inside_named(fact, &chain);
    }
}

fn in_test_folder(path: &RelativePath, names: &[String]) -> bool {
    path.as_str()
        .split(PATH_SEPARATOR)
        .any(|part| is_test_name(stem(part), names))
}

/// A part of a path without its extension.
fn stem(part: &str) -> &str {
    part.split_once(EXTENSION_SEPARATOR)
        .map_or(part, |(stem, _)| stem)
}

/// Whether `candidate` is one of the test names, or ends with one after an underscore.
fn is_test_name(candidate: &str, names: &[String]) -> bool {
    names.iter().any(|name| {
        candidate == name
            || candidate
                .strip_suffix(name.as_str())
                .is_some_and(|front| front.ends_with(TEST_NAME_JOINT))
    })
}

/// Whether the symbol or one of the symbols around it has one of the test names. The walk ends
/// after as many steps as there are symbols, so a broken parent chain cannot loop.
fn inside_named(fact: &SymbolFacts, named: &HashMap<SymbolId, (Option<SymbolId>, bool)>) -> bool {
    let mut current = Some(fact.id);
    for _ in 0..=named.len() {
        let Some((parent, is_named)) = current.and_then(|id| named.get(&id)) else {
            return false;
        };
        if *is_named {
            return true;
        }
        current = *parent;
    }
    false
}

fn value_of(fact: &SymbolFacts, ranking: Ranking) -> Option<f64> {
    match ranking {
        Ranking::Complexity => fact.measures.complexity,
        Ranking::Size => fact.measures.loc,
        Ranking::Coupling => fact.measures.efferent,
        Ranking::MostUsed | Ranking::Unused => None,
    }
}

fn belongs(fact: &SymbolFacts, query: &RankingQuery) -> bool {
    let wanted = query.kinds.holds(fact.kind)
        && (query.include_tests || !fact.test)
        && query
            .language
            .as_ref()
            .is_none_or(|language| *language == fact.language);
    wanted
        && match query.ranking {
            Ranking::MostUsed => fact.usage.total > 0,
            Ranking::Unused => fact.usage.total == 0 && !fact.is_indirect(),
            Ranking::Complexity => fact.kind.is_callable() && fact.measures.complexity.is_some(),
            Ranking::Size | Ranking::Coupling => value_of(fact, query.ranking).is_some(),
        }
}

fn by_usage(order: UsageOrder, left: &SymbolFacts, right: &SymbolFacts) -> Ordering {
    let key = |fact: &SymbolFacts| match order {
        UsageOrder::Certain => (fact.usage.certain, fact.usage.total),
        UsageOrder::All => (fact.usage.total, fact.usage.certain),
    };
    key(right).cmp(&key(left))
}

fn by_place(left: &SymbolFacts, right: &SymbolFacts) -> Ordering {
    (&left.path, &left.qualified_name, left.id).cmp(&(&right.path, &right.qualified_name, right.id))
}

fn compare(query: &RankingQuery, left: &SymbolFacts, right: &SymbolFacts) -> Ordering {
    let first = match query.ranking {
        Ranking::MostUsed => by_usage(query.order, left, right),
        Ranking::Unused => Ordering::Equal,
        Ranking::Complexity | Ranking::Size | Ranking::Coupling => {
            let (ours, theirs) = (
                value_of(left, query.ranking),
                value_of(right, query.ranking),
            );
            theirs
                .unwrap_or_default()
                .total_cmp(&ours.unwrap_or_default())
        }
    };
    first.then_with(|| by_place(left, right))
}

/// Ranks the symbols: the ones the query holds, best first, cut to `limit` rows. Rows with the
/// same value come in the order of path, name and id, so the answer is always the same.
#[must_use]
pub fn rank<'a>(facts: &'a [SymbolFacts], query: &RankingQuery, limit: usize) -> Ranked<'a> {
    let mut rows: Vec<&SymbolFacts> = facts.iter().filter(|fact| belongs(fact, query)).collect();
    rows.sort_by(|left, right| compare(query, left, right));
    let total = rows.len();
    rows.truncate(limit);
    Ranked { rows, total }
}

/// How many callables have each cyclomatic complexity, by rising complexity. Test code counts
/// only with `include_tests`.
#[must_use]
pub fn complexity_histogram(facts: &[SymbolFacts], include_tests: bool) -> Vec<(u32, u32)> {
    complexity_bars(
        facts
            .iter()
            .filter(|fact| fact.kind.is_callable() && (include_tests || !fact.test))
            .filter_map(|fact| fact.measures.complexity.map(|value| (value, 1))),
    )
}

/// The bars of the complexity histogram from `(complexity, how many callables)` pairs, by rising
/// complexity: values that round to the same whole number share a bar.
#[must_use]
pub fn complexity_bars(counted: impl IntoIterator<Item = (f64, u32)>) -> Vec<(u32, u32)> {
    let mut counts: BTreeMap<u32, u32> = BTreeMap::new();
    for (value, callables) in counted {
        if let Some(step) = whole(value) {
            let count = counts.entry(step).or_default();
            *count = count.saturating_add(callables);
        }
    }
    counts.into_iter().collect()
}

/// A measured value as a whole number; `None` if it is negative or too large. The value goes
/// through its rounded text, which needs no lossy cast.
fn whole(value: f64) -> Option<u32> {
    format!("{value:.0}").parse().ok()
}
