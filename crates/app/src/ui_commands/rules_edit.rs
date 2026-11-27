//! The commands that change the rules of a folder project: `rules.set` writes them into the
//! project file, `rules.preview` judges them first without writing anything (ADR 0112).

use std::sync::Arc;

use codeskimr_application::access;
use codeskimr_application::config::{
    Config, ConfigLayer, CycleRule, ForbidRule, LayerLimit, LayerSource, PublicApiRule, Rules,
    with_layer_limits, with_rules,
};
use codeskimr_application::error::CodedError;
use codeskimr_application::registry::{Command, CommandDescriptor, CommandId, Request};
use codeskimr_codes::ErrorCode;
use codeskimr_domain::Permission;
use serde::{Deserialize, Serialize};

use super::consts::{
    MAX_RULES_PER_CALL, RULES_PREVIEW, RULES_PREVIEW_DESCRIPTION, RULES_PREVIEW_TITLE, RULES_SET,
    RULES_SET_DESCRIPTION, RULES_SET_TITLE,
};
use super::explorer::ExplorerPorts;
use super::layers::update_project_file;
use super::rules::{Candidate, RulesOut, judged};
use super::unused_keep::project_file;
use crate::project_file::read_project_file;
use crate::rules_check::files_of;

/// What one layer may use: no list is no limit, an empty list is nothing.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct LimitInput {
    /// Name of the layer.
    pub layer: String,
    /// The layers it may use.
    pub may_use: Option<Vec<String>>,
}

/// The input of `rules.set` and `rules.preview`: the rules the project file gets as a whole.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct EditInput {
    /// Id of the project.
    pub project: u64,
    /// Whether cycles between modules are a violation.
    pub cycles: bool,
    /// The entries of `[[rules.forbid]]`.
    pub forbid: Vec<ForbidRule>,
    /// The entries of `[[rules.public_api]]`.
    pub public_api: Vec<PublicApiRule>,
    /// What the layers may use; a layer without an entry keeps what it has.
    pub limits: Vec<LimitInput>,
}

/// The answer of `rules.set`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct RulesSaved {
    /// Number of rules of the project file now: forbid and public interface entries and cycles.
    pub rules: usize,
}

/// The text of the project file `old` with the wanted rules in it. At most `MAX_RULES_PER_CALL`
/// rules are judged or written in one call.
fn wanted_text(input: &EditInput, old: &str) -> Result<String, CodedError> {
    if input.forbid.len() + input.public_api.len() > MAX_RULES_PER_CALL {
        return Err(CodedError::new(ErrorCode::ConfigInvalidValue));
    }
    let wanted = Rules {
        cycles: if input.cycles {
            CycleRule::Deny
        } else {
            CycleRule::Off
        },
        forbid: input.forbid.clone(),
        public_api: input.public_api.clone(),
    };
    let limits: Vec<LayerLimit> = input
        .limits
        .iter()
        .map(|limit| (limit.layer.clone(), limit.may_use.clone()))
        .collect();
    let text = with_rules(old, &wanted).map_err(CodedError::caused_by)?;
    with_layer_limits(&text, &limits).map_err(CodedError::caused_by)
}

/// Writes the rules of a folder project into its `.codeskimr.toml`.
pub struct RulesSet {
    pub(super) ports: Arc<ExplorerPorts>,
}

impl Command for RulesSet {
    type Input = EditInput;
    type Output = RulesSaved;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(RULES_SET),
        title_key: RULES_SET_TITLE,
        description_key: RULES_SET_DESCRIPTION,
        shortcut: None,
        read_only: false,
        permission: Permission::Manage,
    };

    /// # Errors
    /// - `PRJ-201` for an unknown project, `PRJ-101` for a Git project (it has no project file).
    /// - `CFG-101` if a rule is not valid, `CFG-102` if the file cannot be read or is not TOML.
    /// - `CFG-104` if the file cannot be written.
    fn execute(&self, request: Request<'_, Self>) -> Result<RulesSaved, CodedError> {
        access::require(request.principal, Permission::Manage)?;
        let input = &request.input;
        let path = project_file(&self.ports, input.project)?;
        let saved = RulesSaved {
            rules: input.forbid.len() + input.public_api.len() + usize::from(input.cycles),
        };
        update_project_file(&path, |old| Ok((wanted_text(input, old)?, saved)))
    }
}

/// Judges wanted rules over the newest scan without writing them.
pub struct RulesPreview {
    pub(super) ports: Arc<ExplorerPorts>,
}

impl Command for RulesPreview {
    type Input = EditInput;
    type Output = RulesOut;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(RULES_PREVIEW),
        title_key: RULES_PREVIEW_TITLE,
        description_key: RULES_PREVIEW_DESCRIPTION,
        shortcut: None,
        read_only: true,
        permission: Permission::Read,
    };

    /// # Errors
    /// - `PRJ-201` for an unknown project, `PRJ-101` for a Git project.
    /// - `CFG-101` if a rule is not valid, `CFG-102` if the file cannot be read or is not TOML.
    fn execute(&self, request: Request<'_, Self>) -> Result<RulesOut, CodedError> {
        access::require(request.principal, Permission::Read)?;
        let input = &request.input;
        let path = project_file(&self.ports, input.project)?;
        let (project, snapshot) = self.ports.project_and_newest(input.project)?;
        let text = wanted_text(input, &read_project_file(&path)?)?;
        let layer =
            ConfigLayer::parse(LayerSource::Project, &text).map_err(CodedError::caused_by)?;
        let config = Config::resolve(&[layer]).map_err(CodedError::caused_by)?;
        let (_, baseline) = files_of(&project.source)?;
        judged(
            self.ports.references.as_ref(),
            &Candidate {
                rules: &config.rules,
                layers: &config.project_layers(),
                baseline: baseline.as_ref(),
                holdable: true,
            },
            snapshot.map(|snapshot| snapshot.id),
        )
    }
}
