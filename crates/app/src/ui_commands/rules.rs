//! The command behind the screen "Rules": the architecture rules of the project file with their
//! violations, judged against the held state (ADR 0104).

use std::path::Path;
use std::sync::Arc;

use codeskimr_application::config::{
    Config, ConfigLayer, CycleRule, LayerSource, PROJECT_CONFIG_FILE, Rules,
};
use codeskimr_application::error::CodedError;
use codeskimr_application::registry::{Command, CommandDescriptor, CommandId, Request};
use codeskimr_application::rules::{BASELINE_FILE, Baseline, Outcome, report};
use codeskimr_domain::{Permission, ProjectSource};
use serde::{Deserialize, Serialize};

use super::consts::{FORBID, PUBLIC_API, RULES_SHOW, RULES_SHOW_DESCRIPTION, RULES_SHOW_TITLE};
use super::explorer::ExplorerPorts;
use crate::project_file::read_project_file;

/// The input of `rules.show`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct ShowInput {
    /// Id of the project.
    pub project: u64,
}

/// One place of a violation, as the editor counts: lines and columns start at 1.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PlaceOut {
    /// File, relative to the project.
    pub path: String,
    /// Line of the reference.
    pub line: u32,
    /// Column of the reference.
    pub column: u32,
}

/// One violation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ViolationOut {
    /// The symbol that refers.
    pub from: String,
    /// The symbol it must not reach.
    pub to: String,
    /// Whether the held state has it already: an old burden.
    pub old: bool,
    /// Every place of the reference.
    pub places: Vec<PlaceOut>,
}

/// One rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RuleOut {
    /// The id of the rule, as the held state writes it.
    pub id: String,
    /// `forbid` or `public_api`.
    pub kind: &'static str,
    /// `forbid`: the module that must not use `to`. `public_api`: the module itself.
    pub from: String,
    /// `forbid`: the module that must not be used. `public_api`: empty.
    pub to: String,
    /// `public_api`: the prefixes of the inside.
    pub internal: Vec<String>,
    /// The reason the project gave, if any.
    pub reason: Option<String>,
    /// The violations, old ones included.
    pub violations: Vec<ViolationOut>,
    /// How many of them are new.
    pub new: usize,
}

/// The answer of `rules.show`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RulesOut {
    /// Whether the project has a scan; without one no rule can be judged.
    pub scanned: bool,
    /// Whether the cycle rule is switched on in the project file (built in a later step).
    pub cycles: bool,
    /// The day the state was held, `None` if it never was.
    pub held_on: Option<String>,
    /// The rules, in the order of the project file.
    pub rules: Vec<RuleOut>,
    /// Held violations that are gone.
    pub resolved: usize,
    /// Violations that the held state has.
    pub old: usize,
    /// Violations that it does not have: the number that fails the CI.
    pub new: usize,
}

/// The rules of the project file and the held state; both empty for a Git project.
fn files_of(source: &ProjectSource) -> Result<(Rules, Option<Baseline>), CodedError> {
    let ProjectSource::Folder(folder) = source else {
        return Ok((Rules::default(), None));
    };
    let root = Path::new(folder.as_str());
    let text = read_project_file(&root.join(PROJECT_CONFIG_FILE))?;
    let layer = ConfigLayer::parse(LayerSource::Project, &text).map_err(CodedError::caused_by)?;
    let config = Config::resolve(&[layer]).map_err(CodedError::caused_by)?;
    let held = read_project_file(&root.join(BASELINE_FILE))?;
    let baseline = if held.trim().is_empty() {
        None
    } else {
        Some(Baseline::parse(&held)?)
    };
    Ok((config.rules, baseline))
}

fn violations_of(outcome: &Outcome) -> Vec<ViolationOut> {
    outcome
        .violations
        .iter()
        .map(|violation| ViolationOut {
            from: violation.from.to_string(),
            to: violation.to.to_string(),
            old: violation.old,
            places: violation
                .places
                .iter()
                .map(|place| PlaceOut {
                    path: place.path.to_string(),
                    line: place.span.start().line + 1,
                    column: place.span.start().column + 1,
                })
                .collect(),
        })
        .collect()
}

/// What the project file says about one rule: kind, `from` (or the module), `to`, the inside and
/// the reason.
type Shape<'a> = (
    &'static str,
    &'a str,
    &'a str,
    &'a [String],
    Option<&'a String>,
);

/// The rules in the order of their outcomes: every `forbid` entry, then every `public_api` entry.
fn shapes(rules: &Rules) -> Vec<Shape<'_>> {
    let forbid = rules.forbid.iter().map(|rule| {
        let none: &[String] = &[];
        (
            FORBID,
            rule.from.as_str(),
            rule.to.as_str(),
            none,
            rule.reason.as_ref(),
        )
    });
    let public_api = rules.public_api.iter().map(|rule| {
        (
            PUBLIC_API,
            rule.module.as_str(),
            Default::default(),
            rule.internal.as_slice(),
            None,
        )
    });
    forbid.chain(public_api).collect()
}

/// One rule as the screen shows it, with what its outcome found.
fn described((kind, from, to, internal, reason): Shape<'_>, outcome: &Outcome) -> RuleOut {
    RuleOut {
        id: outcome.id.clone(),
        kind,
        from: from.to_owned(),
        to: to.to_owned(),
        internal: internal.to_vec(),
        reason: reason.cloned(),
        violations: violations_of(outcome),
        new: outcome.new_violations(),
    }
}

/// Shows the rules of a project.
pub struct RulesShow {
    pub(super) ports: Arc<ExplorerPorts>,
}

impl Command for RulesShow {
    type Input = ShowInput;
    type Output = RulesOut;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(RULES_SHOW),
        title_key: RULES_SHOW_TITLE,
        description_key: RULES_SHOW_DESCRIPTION,
        shortcut: None,
        read_only: true,
        permission: Permission::Read,
    };

    /// # Errors
    /// - `PRJ-201` for an unknown project.
    /// - `CFG-102` / `CFG-101` if the project file or the held state cannot be read or holds an
    ///   invalid value: the screen fails instead of showing every rule as kept.
    fn execute(&self, request: Request<'_, Self>) -> Result<RulesOut, CodedError> {
        let (project, snapshot) = self.ports.project_and_newest(request.input.project)?;
        let (rules, baseline) = files_of(&project.source)?;
        let cycles = rules.cycles == CycleRule::Deny;
        let Some(snapshot) = snapshot else {
            return Ok(RulesOut {
                scanned: false,
                cycles,
                held_on: None,
                rules: Vec::new(),
                resolved: 0,
                old: 0,
                new: 0,
            });
        };
        let answer = report(
            &rules,
            baseline.as_ref(),
            self.ports.references.as_ref(),
            snapshot.id,
        )?;
        let described: Vec<RuleOut> = answer
            .outcomes
            .iter()
            .zip(shapes(&rules))
            .map(|(outcome, shape)| described(shape, outcome))
            .collect();
        Ok(RulesOut {
            scanned: true,
            cycles,
            held_on: answer.held_on.clone(),
            old: answer.old_burdens(),
            new: answer.new_violations(),
            resolved: answer.resolved.len(),
            rules: described,
        })
    }
}
