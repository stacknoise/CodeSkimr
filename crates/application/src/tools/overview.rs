//! The listing of all indexers for the settings: no project, only this machine.

use std::sync::Arc;

use codeskimr_domain::ToolReport;

use super::judge::findings_of;
use super::view::ToolsView;
use crate::ports::{Clock, IndexRunner, IndexerSpec, IndexerStatus, ToolCatalog};

/// Checks the indexers of all language packs on this machine, without a project: what the
/// settings screen shows.
pub trait IndexerCheck: Send + Sync {
    /// Asks for the version of every indexer and of the toolchain it needs, where each was found
    /// and why one does not work. Starts version queries only, never project code (ADR 0033). A
    /// missing tool is a finding, never an error.
    fn check(&self) -> ToolsView;
}

/// The service behind [`IndexerCheck`].
pub struct IndexerOverview {
    catalog: Arc<dyn ToolCatalog>,
    runner: Arc<dyn IndexRunner>,
    clock: Arc<dyn Clock>,
}

impl IndexerOverview {
    /// Creates the service from the plans of the packs, the runner and the clock.
    #[must_use]
    pub fn new(
        catalog: Arc<dyn ToolCatalog>,
        runner: Arc<dyn IndexRunner>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            catalog,
            runner,
            clock,
        }
    }

    /// What the runner says about `spec`, asked once per distinct spec.
    fn status_of(
        &self,
        asked: &mut Vec<(IndexerSpec, IndexerStatus)>,
        spec: &IndexerSpec,
    ) -> IndexerStatus {
        if let Some((_, status)) = asked.iter().find(|(seen, _)| seen == spec) {
            return status.clone();
        }
        let status = self.runner.check(spec);
        asked.push((spec.clone(), status.clone()));
        status
    }
}

impl IndexerCheck for IndexerOverview {
    fn check(&self) -> ToolsView {
        // Languages with the same indexer (TypeScript, JavaScript, TSX) share it: ask once.
        let mut asked: Vec<(IndexerSpec, IndexerStatus)> = Vec::new();
        let mut findings = Vec::new();
        for (language, plan) in self.catalog.plans() {
            let spec = plan.indexer;
            let status = self.status_of(&mut asked, &spec);
            findings.extend(findings_of(&language, &spec, &status, &[]));
        }
        let report = ToolReport {
            checked_at: self.clock.now(),
            findings,
        };
        let mut view = ToolsView::of(&report);
        for tool in &mut view.tools {
            tool.path = self.runner.locate(&tool.tool);
        }
        view
    }
}
