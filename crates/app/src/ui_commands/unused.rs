//! The symbols without a usage under the exception list of the project (ADR 0101).
//!
//! The stored statistics (ADR 0091) know nothing of the list, which stands in the project file.
//! A project that has entries is therefore judged from the facts of its newest snapshot: the
//! symbols an entry covers are taken out of the ranking and the counts and listed apart with the
//! reason of the entry. A project without entries keeps the stored numbers.

use std::path::Path;

use codeskimr_application::config::{
    Config, ConfigLayer, LayerSource, PROJECT_CONFIG_FILE, UnusedIgnore,
};
use codeskimr_application::error::CodedError;
use codeskimr_domain::{
    Assessment, Project, ProjectSource, RankingQuery, SymbolFacts, mark_tests, rank,
};

use super::layers::read_project_file;

/// The exception list of a folder project; empty for a Git project or without a project file.
///
/// # Errors
/// - `CFG-102` if the project file cannot be read or is not TOML.
/// - `CFG-101` / `CFG-103` if it holds an invalid value, for example an entry without a reason.
///
/// The statistics fail with the code of the scan instead of showing every symbol again as if the
/// list were empty.
pub(super) fn rules_of(project: &Project) -> Result<Vec<UnusedIgnore>, CodedError> {
    let ProjectSource::Folder(folder) = &project.source else {
        return Ok(Vec::new());
    };
    let path = Path::new(folder.as_str()).join(PROJECT_CONFIG_FILE);
    let text = read_project_file(&path)?;
    let layer = ConfigLayer::parse(LayerSource::Project, &text).map_err(CodedError::caused_by)?;
    let config = Config::resolve(&[layer]).map_err(CodedError::caused_by)?;
    Ok(config.unused.ignore)
}

/// The symbols without a usage, split by the exception list.
pub(super) struct Judged {
    /// The symbols that stay in the ranking, the very likely unused ones first, then by place.
    pub(super) kept: Vec<SymbolFacts>,
    /// The symbols an entry covers, with the entry that does.
    pub(super) left_out: Vec<(SymbolFacts, UnusedIgnore)>,
    /// Test code without usages that the ranking leaves out; zero if test code counts.
    pub(super) tests_hidden: usize,
}

/// The first entry that covers the symbol.
fn covering<'a>(rules: &'a [UnusedIgnore], fact: &SymbolFacts) -> Option<&'a UnusedIgnore> {
    rules
        .iter()
        .find(|rule| rule.covers(fact.path.as_str(), fact.qualified_name.as_str()))
}

/// How the ranking of unused symbols orders the assessments.
fn assessment_rank(assessment: Assessment) -> usize {
    Assessment::ALL
        .iter()
        .position(|candidate| *candidate == assessment)
        .unwrap_or(Assessment::ALL.len())
}

/// Judges the facts of a snapshot: the ranking `query` (of unused symbols) split by `rules`.
/// `approximate` names the languages whose usages are an approximation, `test_names` the folders
/// and modules that hold test code.
pub(super) fn judge(
    mut facts: Vec<SymbolFacts>,
    query: &RankingQuery,
    rules: &[UnusedIgnore],
    approximate: &[String],
    test_names: &[String],
) -> Judged {
    mark_tests(&mut facts, test_names);
    // One ranking with test code; the ranking without it is the same rows minus the test ones.
    let all = RankingQuery {
        include_tests: true,
        ..query.clone()
    };
    let mut ranked: Vec<&SymbolFacts> = rank(&facts, &all, usize::MAX).rows;
    let with_tests = ranked.len();
    if !query.include_tests {
        ranked.retain(|fact| !fact.test);
    }
    let tests_hidden = with_tests.saturating_sub(ranked.len());
    let assess = |fact: &SymbolFacts| {
        let language = approximate
            .iter()
            .any(|candidate| candidate == fact.language.as_str());
        assessment_rank(Assessment::of(fact.visibility, language))
    };
    ranked.sort_by_key(|fact| assess(fact));
    let mut judged = Judged {
        kept: Vec::new(),
        left_out: Vec::new(),
        tests_hidden,
    };
    for fact in ranked {
        match covering(rules, fact) {
            Some(rule) => judged.left_out.push((fact.clone(), rule.clone())),
            None => judged.kept.push(fact.clone()),
        }
    }
    judged
}
