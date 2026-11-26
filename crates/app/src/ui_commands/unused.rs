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
use codeskimr_domain::{Assessment, Project, ProjectSource, SymbolFacts};

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
    /// The symbols an entry covers, with the position of the entry that does.
    pub(super) left_out: Vec<(SymbolFacts, usize)>,
    /// Test code without usages that the ranking leaves out; zero if test code counts.
    pub(super) tests_hidden: usize,
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
pub(super) struct Judgement<'a> {
    /// Whether test code stays in the ranking.
    pub(super) include_tests: bool,
    /// The exception list of the project.
    pub(super) rules: &'a [UnusedIgnore],
    /// The languages whose usages are an approximation.
    pub(super) approximate: &'a [String],
}

/// The position of the assessment of a symbol in the order of the ranking.
fn slot_of(fact: &SymbolFacts, approximate: &[String]) -> usize {
    let language = approximate
        .iter()
        .any(|candidate| candidate == fact.language.as_str());
    assessment_rank(Assessment::of(fact.visibility, language))
}

/// Judges the symbols without usages of a snapshot (`unused`, in the order of their place, test
/// code marked by the store): splits them by the exception list and puts the very likely unused
/// ones first. The symbols are moved, not copied.
pub(super) fn judge(unused: Vec<SymbolFacts>, ask: &Judgement<'_>) -> Judged {
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
