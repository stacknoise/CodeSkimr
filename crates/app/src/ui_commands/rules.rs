//! The command behind the screen "Rules": the architecture rules of the project file with their
//! violations, judged against the held state (ADR 0104).

use std::path::Path;
use std::sync::Arc;

use codeskimr_application::access;
use codeskimr_application::config::{CycleRule, Rules};
use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{Clock, ReferencePlaces};
use codeskimr_application::registry::{Command, CommandDescriptor, CommandId, Request};
use codeskimr_application::rules::{BASELINE_FILE, Baseline, Outcome, Ring, RuleKind, report};
use codeskimr_codes::ErrorCode;
use codeskimr_domain::{
    Layer, Permission, Project, ProjectSource, QualifiedName, SnapshotId, Timestamp,
};
use serde::{Deserialize, Serialize};

use super::consts::{
    CYCLES, FORBID, LAYERS, PUBLIC_API, RULES_HOLD, RULES_HOLD_DESCRIPTION, RULES_HOLD_TITLE,
    RULES_SHOW, RULES_SHOW_DESCRIPTION, RULES_SHOW_TITLE,
};
use super::explorer::ExplorerPorts;
use super::layers::write_project_file;
use crate::clock::SystemClock;
use crate::rules_check::{files_of, rules_of};

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
    /// For a cycle, the ring behind it; `from` and `to` are then the ends of its weakest edge.
    pub ring: Option<RingOut>,
}

/// The ring of modules behind a cycle violation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RingOut {
    /// Every module of the ring, sorted.
    pub modules: Vec<String>,
    /// A way round the ring, starting with the weakest edge and ending where it starts.
    pub chain: Vec<String>,
    /// How many references the weakest edge has.
    pub weakest_count: u32,
    /// Over how many edges the ring is resolved.
    pub resolution_edges: usize,
}

/// What one layer may use.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AllowedOut {
    /// The layer.
    pub layer: String,
    /// The layers it may use; empty if it may use none.
    pub may_use: Vec<String>,
}

/// One rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RuleOut {
    /// The id of the rule, as the held state writes it.
    pub id: String,
    /// `layers`, `cycles`, `forbid` or `public_api`.
    pub kind: &'static str,
    /// `forbid`: the module that must not use `to`. `public_api`: the module itself.
    pub from: String,
    /// `forbid`: the module that must not be used. `public_api`: empty.
    pub to: String,
    /// `public_api`: the prefixes of the inside.
    pub internal: Vec<String>,
    /// The reason the project gave, if any.
    pub reason: Option<String>,
    /// `layers`: what each layer that limits its use may use; empty for the other kinds.
    pub allowed: Vec<AllowedOut>,
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
    /// Whether the state can be held: the project is a folder, so the file has a place.
    pub holdable: bool,
    /// Whether the cycle rule is switched on in the project file.
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

fn ring_out(ring: &Ring) -> RingOut {
    let names = |list: &[QualifiedName]| list.iter().map(ToString::to_string).collect();
    RingOut {
        modules: names(&ring.modules),
        chain: names(&ring.chain),
        weakest_count: ring.weakest_count,
        resolution_edges: ring.resolution_edges,
    }
}

fn violations_of(outcome: &Outcome) -> Vec<ViolationOut> {
    outcome
        .violations
        .iter()
        .map(|violation| ViolationOut {
            from: violation.from.to_string(),
            to: violation.to.to_string(),
            old: violation.old,
            ring: violation.ring.as_ref().map(ring_out),
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

/// The rules of the project file in the order of their outcomes: every `forbid` entry, then every
/// `public_api` entry.
fn shapes(rules: &Rules) -> Vec<Shape<'_>> {
    let cycles = (rules.cycles == CycleRule::Deny).then_some((
        CYCLES,
        Default::default(),
        Default::default(),
        &[][..],
        None,
    ));
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
    cycles.into_iter().chain(forbid).chain(public_api).collect()
}

/// The layers that limit what they may use, as the layer rule states them.
fn allowed_of(layers: &[Layer]) -> Vec<AllowedOut> {
    layers
        .iter()
        .filter_map(|layer| {
            layer.may_use.as_ref().map(|names| AllowedOut {
                layer: layer.name.to_string(),
                may_use: names.iter().map(ToString::to_string).collect(),
            })
        })
        .collect()
}

/// One rule as the screen shows it, with what its outcome found.
fn described(
    (kind, from, to, internal, reason): Shape<'_>,
    allowed: Vec<AllowedOut>,
    outcome: &Outcome,
) -> RuleOut {
    RuleOut {
        id: outcome.id.clone(),
        kind,
        from: from.to_owned(),
        to: to.to_owned(),
        internal: internal.to_vec(),
        reason: reason.cloned(),
        allowed,
        violations: violations_of(outcome),
        new: outcome.new_violations(),
    }
}

/// Every rule of the answer: the layer rule first if there is one, then the rules of the project
/// file in their order.
fn rules_out(outcomes: &[Outcome], rules: &Rules, layers: &[Layer]) -> Vec<RuleOut> {
    let mut rest = shapes(rules).into_iter();
    outcomes
        .iter()
        .filter_map(|outcome| match outcome.kind {
            RuleKind::Layers => {
                let none: &[String] = &[];
                let shape = (LAYERS, Default::default(), Default::default(), none, None);
                Some(described(shape, allowed_of(layers), outcome))
            }
            _ => rest
                .next()
                .map(|shape| described(shape, Vec::new(), outcome)),
        })
        .collect()
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
        let holdable = matches!(project.source, ProjectSource::Folder(_));
        judged(
            self.ports.references.as_ref(),
            &Candidate {
                rules: &rules,
                layers: &project.layers,
                baseline: baseline.as_ref(),
                holdable,
            },
            snapshot.map(|snapshot| snapshot.id),
        )
    }
}

/// The rules to judge, the layers they are judged with and the held state they are held against.
pub(super) struct Candidate<'a> {
    pub(super) rules: &'a Rules,
    pub(super) layers: &'a [Layer],
    pub(super) baseline: Option<&'a Baseline>,
    pub(super) holdable: bool,
}

/// The answer of `rules.show` for `candidate` over the references of `snapshot`; without a scan
/// there is nothing to judge.
pub(super) fn judged(
    references: &dyn ReferencePlaces,
    candidate: &Candidate<'_>,
    snapshot: Option<SnapshotId>,
) -> Result<RulesOut, CodedError> {
    let cycles = candidate.rules.cycles == CycleRule::Deny;
    let holdable = candidate.holdable;
    let Some(snapshot) = snapshot else {
        return Ok(RulesOut {
            scanned: false,
            holdable,
            cycles,
            held_on: None,
            rules: Vec::new(),
            resolved: 0,
            old: 0,
            new: 0,
        });
    };
    let answer = report(
        candidate.rules,
        candidate.layers,
        candidate.baseline,
        references,
        snapshot,
    )?;
    Ok(RulesOut {
        scanned: true,
        holdable,
        cycles,
        held_on: answer.held_on.clone(),
        old: answer.old_burdens(),
        new: answer.new_violations(),
        resolved: answer.resolved.len(),
        rules: rules_out(&answer.outcomes, candidate.rules, candidate.layers),
    })
}

/// What holding the state wrote.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Held {
    /// The day written into the file.
    pub held_on: String,
    /// How many violations the file holds now.
    pub entries: usize,
}

/// Writes every violation of the newest scan as an old burden into the file next to the project
/// file, in place of the one that was there (ADR 0104, decision 4).
///
/// # Errors
/// - `PRJ-101` for a Git project: it has no folder the file could stand in.
/// - `CFG-101` / `CFG-102` if the project file cannot be read (the old held state is replaced, so
///   it is not read), a coded failure of the `STO` group, `CFG-104` if the file cannot be written.
pub(crate) fn hold(
    references: &dyn ReferencePlaces,
    project: &Project,
    snapshot: SnapshotId,
    now: Timestamp,
) -> Result<Held, CodedError> {
    let ProjectSource::Folder(folder) = &project.source else {
        return Err(CodedError::new(ErrorCode::ProjectSourceInvalid));
    };
    let rules = rules_of(&project.source)?;
    let answer = report(&rules, &project.layers, None, references, snapshot)?;
    let held_on = now.iso_date();
    let state = Baseline::holding(&held_on, &answer.outcomes);
    write_project_file(
        &Path::new(folder.as_str()).join(BASELINE_FILE),
        &state.to_text()?,
    )?;
    Ok(Held {
        held_on,
        entries: state.entries.len(),
    })
}

/// Holds the state of the rules of a folder project.
pub struct RulesHold {
    pub(super) ports: Arc<ExplorerPorts>,
}

impl Command for RulesHold {
    type Input = ShowInput;
    type Output = Held;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(RULES_HOLD),
        title_key: RULES_HOLD_TITLE,
        description_key: RULES_HOLD_DESCRIPTION,
        shortcut: None,
        read_only: false,
        permission: Permission::Manage,
    };

    /// # Errors
    /// - `PRJ-201` for an unknown project, `PRJ-101` for a Git project (it has no folder for the
    ///   file), `MCP-203` before the first scan.
    /// - `CFG-101` / `CFG-102` if the project file cannot be read (the old held state is replaced,
    ///   so it is not read), `CFG-104` if the file cannot be written.
    fn execute(&self, request: Request<'_, Self>) -> Result<Held, CodedError> {
        access::require(request.principal, Permission::Manage)?;
        let (project, snapshot) = self.ports.project_and_newest(request.input.project)?;
        if !matches!(project.source, ProjectSource::Folder(_)) {
            return Err(CodedError::new(ErrorCode::ProjectSourceInvalid));
        }
        let snapshot = snapshot.ok_or_else(|| CodedError::new(ErrorCode::ProjectNotScanned))?;
        hold(
            self.ports.references.as_ref(),
            &project,
            snapshot.id,
            SystemClock.now(),
        )
    }
}
