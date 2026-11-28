//! `impact_of_diff`: what the difference between two scans of the project hits (ADR 0097).

use codeskimr_application::access;
use codeskimr_application::compare::{
    ComparisonAnswer, ComparisonRequest, FilterWanted, MetricsAnswer, MetricsWanted, change_filter,
    metrics_request,
};
use codeskimr_codes::i18n::COMPARE_DIRECT_ONLY_NOTE;
use codeskimr_domain::{Principal, ProjectId, SnapshotId};
use serde_json::Value;

use super::{Tools, to_json};
use crate::args::DiffArgs;
use crate::consts::{METRICS_KEY, READ_TOOL_PERMISSION};
use crate::error::McpError;

impl Tools {
    /// The comparison of two retained scans of the project with the direct usages of every removed
    /// symbol and of every symbol whose signature or visibility changed (ADR 0097). Without
    /// `base` and `target` it compares the newest scan with the one before it. It reads the
    /// stored scans, so a change in the files that no scan has seen is not in it.
    ///
    /// # Errors
    /// `AUTH-401` if the principal may not read, `MCP-201` for the project, `CMP-201` if a scan is
    /// not retained or there is only one scan, `CMP-101` if the base is not older than the target,
    /// `CMP-301` if the scans used other naming rules, or the code of a failing port.
    pub fn impact_of_diff(
        &self,
        principal: &Principal,
        args: &DiffArgs,
    ) -> Result<Value, McpError> {
        access::require(principal, READ_TOOL_PERMISSION)?;
        let project = self.project(args.project)?;
        let comparison = self.scan_diffs().compare(&ComparisonRequest {
            project: project.id,
            base: args.base.map(SnapshotId::new),
            target: args.target.map(SnapshotId::new),
            with_impact: true,
            filter: change_filter(FilterWanted {
                name: args.name.as_deref(),
                cause: args.cause.as_deref(),
                kind: args.kind.as_deref(),
                public_only: args.public_only == Some(true),
                only_open: args.only_open == Some(true),
            })?,
        })?;
        let mut answer = to_json(&ComparisonAnswer::new(
            &comparison,
            Some(self.text(COMPARE_DIRECT_ONLY_NOTE)),
        ))?;
        if args.metrics == Some(true) {
            let metrics = self.metric_changes(project.id, args)?;
            if let Some(object) = answer.as_object_mut() {
                object.insert(METRICS_KEY.to_owned(), metrics);
            }
        }
        Ok(answer)
    }

    /// The metric changes between the same two scans, as `scans.metrics` answers them (ADR 0114).
    fn metric_changes(&self, project: ProjectId, args: &DiffArgs) -> Result<Value, McpError> {
        let request = metrics_request(
            project,
            (
                args.base.map(SnapshotId::new),
                args.target.map(SnapshotId::new),
            ),
            MetricsWanted {
                level: args.metrics_level.as_deref(),
                only: &[],
                include_tests: args.include_tests == Some(true),
            },
        )?;
        let found = self.scan_diffs().compare_metrics(&request)?;
        to_json(&MetricsAnswer::new(&found))
    }
}
