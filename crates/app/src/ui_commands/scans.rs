//! The commands of the scan history and of the comparison of two scans (ADR 0097): the list of the
//! last scans, the comparison of two of them and the impact of the difference. The comparison is
//! one use case, `codeskimr_application::compare::ScanComparer`; the commands only choose how much
//! of its answer to ask for.

use std::sync::Arc;

use codeskimr_application::access;
use codeskimr_application::compare::ScanListAnswer;
use codeskimr_application::compare::{ComparisonAnswer, ComparisonRequest, ScanDiffs};
use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{CodeIndex, ProjectRepository};
use codeskimr_application::registry::{Command, CommandDescriptor, CommandId, Request};
use codeskimr_domain::{Permission, ProjectId, SnapshotId};
use serde::Deserialize;

use super::consts::{
    SCANS_COMPARE, SCANS_COMPARE_DESCRIPTION, SCANS_COMPARE_TITLE, SCANS_IMPACT,
    SCANS_IMPACT_DESCRIPTION, SCANS_IMPACT_TITLE, SCANS_LIST, SCANS_LIST_DESCRIPTION,
    SCANS_LIST_TITLE,
};

/// The input of `scans.list`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct ListInput {
    /// Id of the project.
    pub project: u64,
}

/// The input of `scans.compare` and `scans.impact`. Without `base` and `target` the newest scan is
/// compared with the one before it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct CompareInput {
    /// Id of the project.
    pub project: u64,
    /// Snapshot id of the older scan; the scan before the target if left out.
    #[serde(default)]
    pub base: Option<u64>,
    /// Snapshot id of the newer scan; the newest scan if left out.
    #[serde(default)]
    pub target: Option<u64>,
}

/// What the command reads.
pub struct ScanListPorts {
    pub(super) projects: Arc<dyn ProjectRepository>,
    pub(super) index: Arc<dyn CodeIndex>,
    /// `snapshots.retention_count`.
    pub(super) keep: u32,
}

/// Lists the retained scans of a project, newest first.
pub struct ScansList {
    pub(super) ports: Arc<ScanListPorts>,
}

impl Command for ScansList {
    type Input = ListInput;
    type Output = ScanListAnswer;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(SCANS_LIST),
        title_key: SCANS_LIST_TITLE,
        description_key: SCANS_LIST_DESCRIPTION,
        shortcut: None,
        read_only: true,
        permission: Permission::Read,
    };

    fn execute(&self, request: Request<'_, Self>) -> Result<ScanListAnswer, CodedError> {
        access::require(request.principal, Permission::Read)?;
        let project = ProjectId::new(request.input.project);
        self.ports.projects.project(project)?;
        let history = self.ports.index.scan_history(project)?;
        Ok(ScanListAnswer::new(&history, self.ports.keep))
    }
}

/// What the commands of the comparison read.
pub struct ComparePorts {
    pub(super) projects: Arc<dyn ProjectRepository>,
    pub(super) comparer: Arc<dyn ScanDiffs>,
}

/// The answer of the comparison, with or without the usages the difference hits.
fn answer(
    ports: &ComparePorts,
    input: &CompareInput,
    with_impact: bool,
) -> Result<ComparisonAnswer, CodedError> {
    let project = ProjectId::new(input.project);
    ports.projects.project(project)?;
    let comparison = ports.comparer.compare(&ComparisonRequest {
        project,
        base: input.base.map(SnapshotId::new),
        target: input.target.map(SnapshotId::new),
        with_impact,
    })?;
    Ok(ComparisonAnswer::new(&comparison, None))
}

/// The commands of the comparison. With `IMPACT` false (`scans.compare`) it compares two retained
/// scans: the symbols that were added, removed, changed or moved, the new layer violations and
/// cycles and the critical symbols the difference touches; `impact` is `null`. With `IMPACT` true
/// (`scans.impact`) it gives the same answer and, in `impact`, the direct usages of every removed
/// symbol and of every symbol whose signature or visibility changed, as adapted or not touched; one
/// call serves both tabs of the screen "Change".
pub struct ScansDiff<const IMPACT: bool> {
    pub(super) ports: Arc<ComparePorts>,
}

/// `scans.compare`: the comparison without the usages.
pub type ScansCompare = ScansDiff<false>;
/// `scans.impact`: the comparison with the usages the difference hits.
pub type ScansImpact = ScansDiff<true>;

impl<const IMPACT: bool> Command for ScansDiff<IMPACT> {
    type Input = CompareInput;
    type Output = ComparisonAnswer;

    const DESCRIPTOR: CommandDescriptor = if IMPACT {
        diff_descriptor(SCANS_IMPACT, SCANS_IMPACT_TITLE, SCANS_IMPACT_DESCRIPTION)
    } else {
        diff_descriptor(
            SCANS_COMPARE,
            SCANS_COMPARE_TITLE,
            SCANS_COMPARE_DESCRIPTION,
        )
    };

    fn execute(&self, request: Request<'_, Self>) -> Result<ComparisonAnswer, CodedError> {
        access::require(request.principal, Permission::Read)?;
        answer(&self.ports, &request.input, IMPACT)
    }
}

/// The descriptor of a read-only command of the comparison.
const fn diff_descriptor(
    id: &'static str,
    title_key: &'static str,
    description_key: &'static str,
) -> CommandDescriptor {
    CommandDescriptor {
        id: CommandId::new(id),
        title_key,
        description_key,
        shortcut: None,
        read_only: true,
        permission: Permission::Read,
    }
}
