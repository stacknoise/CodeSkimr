//! The service behind `ProjectToolCheck`.

use std::sync::Arc;

use codeskimr_domain::{ProjectId, ProjectSource, ToolReport};

use super::judge::findings_of;
use crate::error::CodedError;
use crate::ports::{
    Clock, IndexRunner, IndexerSpec, IndexerStatus, ProjectCatalog, ProjectRepository,
    ProjectToolCheck, RequirementReader, ToolCatalog, ToolPlan, ToolReportStore, ToolVersion,
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
    /// that is newer than what the pack asks for.
    fn asked_for(&self, source: &ProjectSource, plan: ToolPlan) -> IndexerSpec {
        let mut spec = plan.indexer;
        for requirement in &plan.requirements {
            let Some(text) = self.ports.requirements.read(source, &requirement.file) else {
                continue;
            };
            let Some(asked) = ToolVersion::parse_requirement(&text) else {
                continue;
            };
            for tool in &mut spec.toolchain {
                if tool.program == requirement.program && tool.min_version < asked {
                    tool.min_version = asked;
                }
            }
        }
        spec
    }
}

impl ToolChecker {
    /// What the runner says about `spec`, asked once per distinct spec.
    fn status_of(
        &self,
        checked: &mut Vec<(IndexerSpec, IndexerStatus)>,
        spec: &IndexerSpec,
    ) -> IndexerStatus {
        if let Some((_, status)) = checked.iter().find(|(seen, _)| seen == spec) {
            return status.clone();
        }
        let status = self.ports.runner.check(spec);
        checked.push((spec.clone(), status.clone()));
        status
    }
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
            let spec = self.asked_for(&source, plan);
            let status = self.status_of(&mut checked, &spec);
            findings.extend(findings_of(&share.language, &spec, &status));
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
