//! `get_dependencies` with `level` `module` (ADR 0108): the modules of the project with their
//! coupling and stability, the rings and the dependencies that end them, the dependencies
//! against the layering and the modules that stand out. The numbers come from
//! [`ModuleReport`], the same report the tab "Modules" of the UI reads.

use codeskimr_application::module_report::{
    CycleOut, ModuleReport, ModuleRow, RiskOut, ViolationOut,
};
use serde::Serialize;
use serde_json::Value;

use super::usages::cut_notice;
use super::{SnapshotInfo, Tools, to_json};
use crate::args::DependenciesArgs;
use crate::consts::{ARG_LEVEL, LEVEL_MODULE};
use crate::error::{CodeReport, McpError};

#[derive(Serialize)]
struct ModulesAnswer {
    snapshot: SnapshotInfo,
    level: &'static str,
    /// Number of modules, also those that were cut from the list.
    total: usize,
    /// The most referenced modules first.
    modules: Vec<ModuleRow>,
    /// The rings among all modules, each with the dependencies that end it.
    cycles: Vec<CycleOut>,
    /// The dependencies against the layering among all modules.
    violations: Vec<ViolationOut>,
    /// The modules that stand out, the most used first.
    risks: Vec<RiskOut>,
    notice: Option<CodeReport>,
}

/// Whether the arguments ask for the modules and for nothing else.
fn asks_for_modules(args: &DependenciesArgs) -> bool {
    args.level.as_deref() == Some(LEVEL_MODULE)
        && args.symbol.is_none()
        && args.layer.is_none()
        && args.direction.is_none()
        && args.include_tests.is_none()
}

impl Tools {
    /// The answer for `level` `module`. Every other value of `level`, and any of `symbol`,
    /// `layer`, `direction` and `include_tests` next to it, is `MCP-101` with
    /// `params.detail = level`.
    pub(super) fn module_answer(&self, args: &DependenciesArgs) -> Result<Value, McpError> {
        if !asks_for_modules(args) {
            return Err(McpError::ArgumentsInvalid(ARG_LEVEL.to_owned()));
        }
        let scope = self.scope(args.project)?;
        let limit = self.apply_limit(args.limit);
        let depth = usize::try_from(args.depth.unwrap_or_default()).unwrap_or_default();
        let graph = self
            .index()
            .namespace_graph(scope.snapshot.id)?
            .collapsed(depth);
        let report = ModuleReport::of(&graph, &scope.project.layers);
        let shown = report.kept(limit.value as usize);
        to_json(&ModulesAnswer {
            snapshot: scope.info,
            level: LEVEL_MODULE,
            total: graph.namespaces.len(),
            modules: report.rows(&shown),
            cycles: report.cycles(),
            violations: report.violation_rows(),
            risks: report.risks(self.risks()),
            notice: cut_notice(
                limit.value,
                limit.clamped,
                shown.len(),
                graph.namespaces.len(),
                None,
            ),
        })
    }
}
