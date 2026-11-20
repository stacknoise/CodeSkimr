//! The service behind `ProjectToolCheck`.

use std::sync::Arc;

use codeskimr_domain::{Ceiling, ProjectId, ProjectSource, ToolReport};

use super::judge::{ToolLimit, findings_of, status_once};
use crate::error::CodedError;
use crate::ports::{
    Clock, IndexRunner, IndexerSpec, IndexerStatus, ProjectCatalog, ProjectRepository,
    ProjectToolCheck, RequirementReader, RequirementValue, ToolCatalog, ToolPlan, ToolReportStore,
    ToolRequirement, ToolVersion,
};

/// Everything the check needs from the adapters, wired by the composition root.
pub struct ToolCheckPorts {
    /// The plans the language packs ask for.
    pub catalog: Arc<dyn ToolCatalog>,
    /// Finds out whether an indexer and its toolchain are installed (starts no project code).
    pub runner: Arc<dyn IndexRunner>,
    /// Reads the versions the project requires.
    pub requirements: Arc<dyn RequirementReader>,
    /// Keeps the report with the project.
    pub store: Arc<dyn ToolReportStore>,
    /// Reads the project.
    pub projects: Arc<dyn ProjectRepository>,
    /// Reads the languages of the newest snapshot.
    pub overview: Arc<dyn ProjectCatalog>,
    /// The time of the check.
    pub clock: Arc<dyn Clock>,
}

/// Checks the tools of a project and stores the result.
pub struct ToolChecker {
    ports: ToolCheckPorts,
}

impl ToolChecker {
    /// Creates the checker.
    #[must_use]
    pub const fn new(ports: ToolCheckPorts) -> Self {
        Self { ports }
    }

    /// The indexer of `plan` with every toolchain version raised to what the project asks for, if
    /// that is newer than what the pack asks for, and the ceilings that the files of the project
    /// set for the tools.
    fn asked_for(&self, source: &ProjectSource, plan: ToolPlan) -> (IndexerSpec, Vec<ToolLimit>) {
        let mut spec = plan.indexer;
        let mut limits = Vec::new();
        for requirement in &plan.requirements {
            // Every manifest folder asks; the newest version counts.
            for value in self.ports.requirements.read_all(source, &requirement.file) {
                let Some(asked) = ToolVersion::parse_requirement(&value.version) else {
                    continue;
                };
                raise(&mut spec, &requirement.program, asked);
                limits.extend(limit_of(requirement, &value, asked));
            }
        }
        (spec, limits)
    }
}

/// Raises the minimum of `program` to `asked` if that is newer.
fn raise(spec: &mut IndexerSpec, program: &str, asked: ToolVersion) {
    for tool in &mut spec.toolchain {
        if tool.program == program && tool.min_version < asked {
            tool.min_version = asked;
        }
    }
}

/// The ceiling that one file sets, if it sets one: a file whose pack names no `rollForward`
/// rule, or whose policy lets everything through, limits nothing.
fn limit_of(
    requirement: &ToolRequirement,
    value: &RequirementValue,
    asked: ToolVersion,
) -> Option<ToolLimit> {
    let rule = requirement.file.roll_forward.as_ref()?;
    let ceiling = rule.ceiling_of(value.policy.as_deref());
    (ceiling != Ceiling::Any).then(|| ToolLimit {
        program: requirement.program.clone(),
        asked,
        ceiling,
    })
}

impl ProjectToolCheck for ToolChecker {
    fn check(&self, project: ProjectId) -> Result<ToolReport, CodedError> {
        let source = self.ports.projects.project(project)?.source;
        // TypeScript, JavaScript and TSX share their tools: each distinct spec is checked once.
        let mut checked: Vec<(IndexerSpec, IndexerStatus)> = Vec::new();
        let mut findings = Vec::new();
        for share in self.ports.overview.languages(project)? {
            let Some(plan) = self.ports.catalog.plan_of(&share.language) else {
                continue;
            };
            let (spec, limits) = self.asked_for(&source, plan);
            let status = status_once(self.ports.runner.as_ref(), &mut checked, &spec);
            findings.extend(findings_of(&share.language, &spec, &status, &limits));
        }
        let report = ToolReport {
            checked_at: self.ports.clock.now(),
            findings,
        };
        self.ports.store.replace(project, &report)?;
        Ok(report)
    }

    fn stored(&self, project: ProjectId) -> Result<Option<ToolReport>, CodedError> {
        self.ports.store.load(project)
    }
}
