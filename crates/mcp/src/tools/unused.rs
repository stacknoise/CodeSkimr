//! `find_unused`.

use codeskimr_application::access;
use codeskimr_application::config::UnusedIgnore;
use codeskimr_application::unused::{Finding, Findings};
use codeskimr_domain::{
    Assessment, KindGroup, LanguageId, Principal, Ranking, RankingQuery, UsageOrder,
};
use serde::Serialize;
use serde_json::Value;

use super::{SnapshotInfo, Tools, optional_label, to_json, truncation_notice};
use crate::args::UnusedArgs;
use crate::consts::{ARG_ASSESSMENT, ARG_KIND, ARG_LANGUAGE, READ_TOOL_PERMISSION};
use crate::error::{CodeReport, McpError};

#[derive(Serialize)]
struct Row {
    id: u64,
    name: String,
    qualified_name: String,
    kind: &'static str,
    path: String,
    language: String,
    /// `likely`, `public` or `approximate`: how much "no usage found" says.
    assessment: &'static str,
    /// `private`, `internal`, `public` or `approximate`: why the symbol counts as unused.
    reason: &'static str,
    test: bool,
}

#[derive(Serialize)]
struct Rule {
    paths: Vec<String>,
    symbols: Vec<String>,
    reason: String,
}

#[derive(Serialize)]
struct LeftOut {
    #[serde(flatten)]
    row: Row,
    /// The entry of `[[unused.ignore]]` in the project file that takes the symbol out.
    rule: Rule,
}

#[derive(Serialize)]
struct Counts {
    likely: usize,
    public: usize,
    approximate: usize,
    /// Test code without usages that was left out; zero if `include_tests` was set.
    tests_hidden: usize,
    /// Symbols the exception list of the project takes out.
    left_out: usize,
}

#[derive(Serialize)]
struct Answer {
    snapshot: SnapshotInfo,
    /// How many symbols without a usage the question holds, before the limit cut the list.
    total: usize,
    counts: Counts,
    unused: Vec<Row>,
    left_out: Vec<LeftOut>,
    notice: Option<CodeReport>,
}

fn row(finding: &Finding) -> Row {
    Row {
        id: finding.id.get(),
        name: finding.name.clone(),
        qualified_name: finding.qualified_name.clone(),
        kind: finding.kind.label(),
        path: finding.path.clone(),
        language: finding.language.clone(),
        assessment: finding.assessment.label(),
        reason: finding.reason.label(),
        test: finding.test,
    }
}

fn rule(entry: &UnusedIgnore) -> Rule {
    Rule {
        paths: entry.paths.clone(),
        symbols: entry.symbols.clone(),
        reason: entry.reason.clone(),
    }
}

/// What the arguments ask: the ranking and the assessment the list is cut to.
fn question(args: &UnusedArgs) -> Result<(RankingQuery, Option<Assessment>), McpError> {
    let kinds = optional_label::<KindGroup>(args.kind.as_deref(), ARG_KIND)?;
    let wanted = optional_label::<Assessment>(args.assessment.as_deref(), ARG_ASSESSMENT)?;
    let language = args
        .language
        .as_deref()
        .map(LanguageId::new)
        .transpose()
        .map_err(|_| McpError::ArgumentsInvalid(ARG_LANGUAGE.to_owned()))?;
    let query = RankingQuery {
        ranking: Ranking::Unused,
        kinds: kinds.unwrap_or(KindGroup::All),
        language,
        include_tests: args.include_tests.unwrap_or(false),
        order: UsageOrder::Certain,
    };
    Ok((query, wanted))
}

fn counts(found: &Findings) -> Counts {
    let count = |assessment: Assessment| {
        found
            .kept
            .iter()
            .filter(|finding| finding.assessment == assessment)
            .count()
    };
    Counts {
        likely: count(Assessment::Likely),
        public: count(Assessment::Public),
        approximate: count(Assessment::Approximate),
        tests_hidden: found.tests_hidden,
        left_out: found.left_out.len(),
    }
}

impl Tools {
    /// Lists the symbols for which no usage was found, with the reason, so that an agent does not
    /// build on code nobody calls. A symbol that the exception list of the project takes out is
    /// listed apart with its entry. "No usage found" is never "dead": the assessment says how
    /// much it means.
    ///
    /// # Errors
    /// `AUTH-401` if the principal may not read, `MCP-101` for an unknown `kind`, `language` or
    /// `assessment`, `MCP-201` and `MCP-203` for the project, `CFG-101` / `CFG-102` if the
    /// exception list of the project file is invalid, or the code of a failing port.
    pub fn find_unused(&self, principal: &Principal, args: &UnusedArgs) -> Result<Value, McpError> {
        access::require(principal, READ_TOOL_PERMISSION)?;
        let (query, wanted) = question(args)?;
        let scope = self.scope(args.project)?;
        let limit = self.apply_limit(args.limit);
        let found = self
            .unused_symbols()
            .unused(&scope.project, scope.snapshot.id, &query)?;
        let rows: Vec<&Finding> = found
            .kept
            .iter()
            .filter(|finding| wanted.is_none_or(|assessment| finding.assessment == assessment))
            .collect();
        let max = usize::try_from(limit.value).unwrap_or(usize::MAX);
        let cut = rows.len() > max || found.left_out.len() > max;
        to_json(&Answer {
            snapshot: scope.info,
            total: rows.len(),
            counts: counts(&found),
            unused: rows.iter().take(max).map(|finding| row(finding)).collect(),
            left_out: found
                .left_out
                .iter()
                .take(max)
                .map(|(finding, entry)| LeftOut {
                    row: row(finding),
                    rule: rule(entry),
                })
                .collect(),
            notice: cut.then(|| truncation_notice(limit.value, Some(rows.len()))),
        })
    }
}
