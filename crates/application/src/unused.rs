//! The symbols without a usage under the exception list of a project (ADR 0101).
//!
//! The stored statistics (ADR 0091) know nothing of the list, which stands in the project file.
//! A project that has entries is therefore judged from the facts of its newest snapshot: the
//! symbols an entry covers are taken out of the ranking and listed apart with the entry that
//! covers them. The statistics screen, `codeskimr unused` and the tool `find_unused` judge the
//! same way.

use codeskimr_domain::{
    Assessment, Project, RankingQuery, SnapshotId, SymbolFacts, SymbolId, SymbolKind, UnusedReason,
};

use crate::config::UnusedIgnore;
use crate::error::CodedError;

/// The symbols without a usage, split by the exception list.
pub struct Judged {
    /// The symbols that stay in the ranking, the very likely unused ones first, then by place.
    pub kept: Vec<SymbolFacts>,
    /// The symbols an entry covers, with the position of the entry that does.
    pub left_out: Vec<(SymbolFacts, usize)>,
    /// Test code without usages that the ranking leaves out; zero if test code counts.
    pub tests_hidden: usize,
}

/// The position of the first entry that covers the symbol.
fn covering(rules: &[UnusedIgnore], fact: &SymbolFacts) -> Option<usize> {
    rules
        .iter()
        .position(|rule| rule.covers(fact.path.as_str(), fact.qualified_name.as_str()))
}

/// How the ranking of unused symbols orders the assessments.
fn assessment_rank(assessment: Assessment) -> usize {
    Assessment::ALL
        .iter()
        .position(|candidate| *candidate == assessment)
        .unwrap_or(Assessment::ALL.len())
}

/// What the judgement is asked: whether test code counts, the entries of the exception list and
/// the languages whose usages are an approximation.
pub struct Judgement<'a> {
    /// Whether test code stays in the ranking.
    pub include_tests: bool,
    /// The exception list of the project.
    pub rules: &'a [UnusedIgnore],
    /// The languages whose usages are an approximation.
    pub approximate: &'a [String],
}

/// The position of the assessment of a symbol in the order of the ranking.
fn slot_of(fact: &SymbolFacts, approximate: &[String]) -> usize {
    assessment_rank(Assessment::of(
        fact.visibility,
        is_approximate(fact, approximate),
    ))
}

/// Judges the symbols without usages of a snapshot (`unused`, in the order of their place, test
/// code marked by the store, see `CodeStatistics::unused_facts`): splits them by the exception
/// list and puts the very likely unused ones first. The symbols are moved, not copied.
#[must_use]
pub fn judge(unused: Vec<SymbolFacts>, ask: &Judgement<'_>) -> Judged {
    let mut judged = Judged {
        kept: Vec::new(),
        left_out: Vec::new(),
        tests_hidden: 0,
    };
    // One bucket per assessment keeps the order of the places inside it and needs no sort.
    let mut buckets: Vec<Vec<SymbolFacts>> = vec![Vec::new(); Assessment::ALL.len()];
    for fact in unused {
        if fact.test && !ask.include_tests {
            judged.tests_hidden += 1;
        } else if let Some(rule) = covering(ask.rules, &fact) {
            judged.left_out.push((fact, rule));
        } else if let Some(bucket) = buckets.get_mut(slot_of(&fact, ask.approximate)) {
            bucket.push(fact);
        }
    }
    judged.kept = buckets.into_iter().flatten().collect();
    judged
}

/// One symbol without a usage, as the tool `find_unused` and the command `unused` name it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// The symbol.
    pub id: SymbolId,
    /// Its simple name.
    pub name: String,
    /// Its qualified name.
    pub qualified_name: String,
    /// Its kind.
    pub kind: SymbolKind,
    /// The file that declares it.
    pub path: String,
    /// The language of that file.
    pub language: String,
    /// How sure the finding is.
    pub assessment: Assessment,
    /// Why the symbol counts as unused.
    pub reason: UnusedReason,
    /// Whether it is test code.
    pub test: bool,
}

/// Whether the usages of the language of `fact` are an approximation.
fn is_approximate(fact: &SymbolFacts, approximate: &[String]) -> bool {
    approximate
        .iter()
        .any(|language| language == fact.language.as_str())
}

impl Finding {
    /// The finding for a symbol without usages; `approximate` names the languages whose usages
    /// are an approximation.
    #[must_use]
    pub fn of(fact: &SymbolFacts, approximate: &[String]) -> Self {
        let approximate = is_approximate(fact, approximate);
        Self {
            id: fact.id,
            name: fact.name.to_string(),
            qualified_name: fact.qualified_name.to_string(),
            kind: fact.kind,
            path: fact.path.as_str().to_owned(),
            language: fact.language.to_string(),
            assessment: Assessment::of(fact.visibility, approximate),
            reason: UnusedReason::of(fact.visibility, approximate),
            test: fact.test,
        }
    }
}

/// The symbols without a usage of a snapshot, split by the exception list.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Findings {
    /// The symbols that stay in the ranking: the very likely unused ones first, then by place.
    pub kept: Vec<Finding>,
    /// The symbols an entry of the exception list covers, with the entry.
    pub left_out: Vec<(Finding, UnusedIgnore)>,
    /// Test code without usages that was left out; zero if test code counts.
    pub tests_hidden: usize,
}

impl Findings {
    /// The findings of a judged snapshot.
    #[must_use]
    pub fn of(judged: &Judged, rules: &[UnusedIgnore], approximate: &[String]) -> Self {
        Self {
            kept: judged
                .kept
                .iter()
                .map(|fact| Finding::of(fact, approximate))
                .collect(),
            left_out: judged
                .left_out
                .iter()
                .filter_map(|(fact, rule)| {
                    Some((Finding::of(fact, approximate), rules.get(*rule)?.clone()))
                })
                .collect(),
            tests_hidden: judged.tests_hidden,
        }
    }
}

/// Answers which symbols of a snapshot have no usage, with the exception list of the project
/// applied (ADR 0101).
pub trait UnusedSymbols: Send + Sync {
    /// The symbols without a usage that the ranking `query` holds.
    ///
    /// # Errors
    /// - `CFG-102` / `CFG-101` if the project file of a folder project cannot be read or holds an
    ///   invalid entry of the exception list.
    /// - A coded failure of the `STO` group.
    fn unused(
        &self,
        project: &Project,
        snapshot: SnapshotId,
        query: &RankingQuery,
    ) -> Result<Findings, CodedError>;
}
