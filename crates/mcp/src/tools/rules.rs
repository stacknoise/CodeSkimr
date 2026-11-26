//! `check_rules`.

use codeskimr_application::access;
use codeskimr_application::rules::{Outcome, Place, RuleKind, Violation};
use codeskimr_codes::ErrorCode;
use codeskimr_domain::Principal;
use serde::Serialize;
use serde_json::Value;

use super::{SnapshotInfo, Tools, to_json, truncation_notice};
use crate::args::RulesArgs;
use crate::consts::{LINE_OFFSET, READ_TOOL_PERMISSION, RULE_FORBID, RULE_LAYERS, RULE_PUBLIC_API};
use crate::error::{CodeReport, McpError};

#[derive(Serialize)]
struct PlaceRow {
    path: String,
    line: u32,
    column: u32,
}

#[derive(Serialize)]
struct ViolationRow {
    code: &'static str,
    /// The symbol that refers.
    from: String,
    /// The symbol it must not reach.
    to: String,
    /// Whether the held state has it already: an old burden.
    old: bool,
    places: Vec<PlaceRow>,
}

#[derive(Serialize)]
struct RuleRow {
    id: String,
    /// `forbid` or `public_api`.
    kind: &'static str,
    /// How many violations the rule has, old ones included.
    total: usize,
    /// How many of them are new.
    new: usize,
    violations: Vec<ViolationRow>,
}

#[derive(Serialize)]
struct Answer {
    snapshot: SnapshotInfo,
    /// The day the state was held; `None` if it never was.
    held_on: Option<String>,
    /// Violations the held state has.
    old: usize,
    /// Violations it does not have: the number that fails the CI.
    new: usize,
    /// Held violations that are gone.
    resolved: usize,
    rules: Vec<RuleRow>,
    notice: Option<CodeReport>,
}

fn place(place: &Place) -> PlaceRow {
    PlaceRow {
        path: place.path.as_str().to_owned(),
        line: place.span.start().line + LINE_OFFSET,
        column: place.span.start().column + LINE_OFFSET,
    }
}

fn violation(violation: &Violation) -> ViolationRow {
    ViolationRow {
        code: ErrorCode::RuleViolation.id(),
        from: violation.from.to_string(),
        to: violation.to.to_string(),
        old: violation.old,
        places: violation.places.iter().map(place).collect(),
    }
}

const fn kind_label(kind: RuleKind) -> &'static str {
    match kind {
        RuleKind::Layers => RULE_LAYERS,
        RuleKind::Forbid => RULE_FORBID,
        RuleKind::PublicApi => RULE_PUBLIC_API,
    }
}

fn rule(outcome: &Outcome, shown: usize) -> RuleRow {
    RuleRow {
        id: outcome.id.clone(),
        kind: kind_label(outcome.kind),
        total: outcome.violations.len(),
        new: outcome.new_violations(),
        violations: outcome
            .violations
            .iter()
            .take(shown)
            .map(violation)
            .collect(),
    }
}

impl Tools {
    /// Judges the architecture rules of the project file against the newest scan, with the held
    /// state of old violations (ADR 0104). Reads the scan, not the files as they are now.
    ///
    /// # Errors
    /// `AUTH-401` if the principal may not read, `MCP-201` and `MCP-203` for the project,
    /// `CFG-101` / `CFG-102` if the project file or the held state is invalid, or the code of a
    /// failing port.
    pub fn check_rules(&self, principal: &Principal, args: &RulesArgs) -> Result<Value, McpError> {
        access::require(principal, READ_TOOL_PERMISSION)?;
        let scope = self.scope(args.project)?;
        let limit = self.apply_limit(args.limit);
        let report = self
            .rules_checks()
            .check(&scope.project, scope.snapshot.id)?;
        let shown = usize::try_from(limit.value).unwrap_or(usize::MAX);
        let cut = report
            .outcomes
            .iter()
            .any(|outcome| outcome.violations.len() > shown);
        to_json(&Answer {
            snapshot: scope.info,
            held_on: report.held_on.clone(),
            old: report.old_burdens(),
            new: report.new_violations(),
            resolved: report.resolved.len(),
            rules: report
                .outcomes
                .iter()
                .map(|outcome| rule(outcome, shown))
                .collect(),
            notice: cut.then(|| truncation_notice(limit.value, None)),
        })
    }
}
