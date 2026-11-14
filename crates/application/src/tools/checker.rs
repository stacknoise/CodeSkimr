//! The service behind `ProjectToolCheck`.

use std::collections::HashMap;
use std::sync::Arc;

use codeskimr_domain::{ProjectId, ProjectSource, ToolReport, ToolSpec};

use super::judge::judge;
use crate::error::CodedError;
use crate::ports::{
    Clock, ProbeOutcome, ProjectCatalog, ProjectRepository, ProjectToolCheck, RequirementReader,
    ToolCatalog, ToolProbe, ToolReportStore,
};

/// Everything the check needs from the adapters, wired by the composition root.
pub struct ToolCheckPorts {
    /// The tools the language packs ask for.
    pub catalog: Arc<dyn ToolCatalog>,
    /// Starts a tool for its version.
    pub probe: Arc<dyn ToolProbe>,
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

    /// The tools of all languages of the project, in the order of the languages.
    fn specs(&self, project: ProjectId) -> Result<Vec<ToolSpec>, CodedError> {
        Ok(self
            .ports
            .overview
            .languages(project)?
            .iter()
            .flat_map(|share| self.ports.catalog.tools_of(&share.language))
            .collect())
    }

    /// Probes every distinct command once: TypeScript and JavaScript share their tools.
    fn probe_all(&self, specs: &[ToolSpec]) -> HashMap<(String, Vec<String>), ProbeOutcome> {
        let mut outcomes = HashMap::new();
        for spec in specs {
            outcomes
                .entry((spec.command.clone(), spec.version_args.clone()))
                .or_insert_with(|| self.ports.probe.probe(&spec.command, &spec.version_args));
        }
        outcomes
    }

    fn requirement(&self, source: &ProjectSource, spec: &ToolSpec) -> Option<String> {
        let requirement = spec.requirement.as_ref()?;
        self.ports.requirements.read(source, requirement)
    }
}

impl ProjectToolCheck for ToolChecker {
    fn check(&self, project: ProjectId) -> Result<ToolReport, CodedError> {
        let source = self.ports.projects.project(project)?.source;
        let specs = self.specs(project)?;
        let outcomes = self.probe_all(&specs);
        let findings = specs
            .iter()
            .filter_map(|spec| {
                let key = (spec.command.clone(), spec.version_args.clone());
                let outcome = outcomes.get(&key)?;
                let required = self.requirement(&source, spec);
                Some(judge(spec, outcome, required.as_deref()))
            })
            .collect();
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
