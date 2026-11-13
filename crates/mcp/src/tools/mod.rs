//! The tools and the context they share. Everything here is synchronous; `server` wraps it for
//! the protocol.

mod architecture;
mod search;
mod symbol;
mod usages;

use std::sync::Arc;

use codeskimr_application::config::McpLimits;
use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{
    Clock, CodeIndex, ProjectRepository, SymbolDetail, SymbolKey, UsageIndex,
};
use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_domain::{Project, ProjectId, QualifiedName, Snapshot, SymbolId};
use serde::Serialize;
use serde_json::Value;

use crate::consts::{ARG_DEPTH, ARG_SYMBOL, MIN_LIMIT};
use crate::error::{CodeReport, McpError};

/// Which snapshot an answer is based on and how old it is.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct SnapshotInfo {
    /// Id of the snapshot.
    pub id: u64,
    /// Seconds since the snapshot was taken, zero if the clock went backwards.
    pub age_seconds: u64,
}

/// A project and its newest snapshot.
pub(crate) struct Scope {
    pub project: Project,
    pub snapshot: Snapshot,
    pub info: SnapshotInfo,
}

/// A limit after it was held to the configured range.
pub(crate) struct AppliedLimit {
    pub value: u32,
    /// The caller asked for more than the maximum.
    pub clamped: bool,
}

/// A depth after it was held to the configured range.
pub(crate) struct AppliedDepth {
    pub value: u32,
    /// The caller asked for more than the maximum.
    pub clamped: bool,
}

/// The ports the tools read through. A tool that needs a new port adds a field here, so the
/// constructor of [`Tools`] keeps its shape.
pub struct Ports {
    /// The registered projects.
    pub projects: Arc<dyn ProjectRepository>,
    /// Symbols, search and layers of the newest snapshots.
    pub index: Arc<dyn CodeIndex>,
    /// Usages and impact (the reverse index).
    pub usages: Arc<dyn UsageIndex>,
    /// The time, for the age of a snapshot.
    pub clock: Arc<dyn Clock>,
}

/// The read-only tools of the server.
pub struct Tools {
    projects: Arc<dyn ProjectRepository>,
    index: Arc<dyn CodeIndex>,
    usages: Arc<dyn UsageIndex>,
    clock: Arc<dyn Clock>,
    limits: McpLimits,
    default_project: ProjectId,
}

impl Tools {
    /// Builds the tools. `default_project` answers calls that name no project.
    #[must_use]
    pub fn new(ports: Ports, limits: McpLimits, default_project: ProjectId) -> Self {
        Self {
            projects: ports.projects,
            index: ports.index,
            usages: ports.usages,
            clock: ports.clock,
            limits,
            default_project,
        }
    }

    /// Holds a requested limit to `1..=max_limit`.
    pub(crate) fn apply_limit(&self, requested: Option<u32>) -> AppliedLimit {
        let wanted = requested.unwrap_or(self.limits.default_limit);
        AppliedLimit {
            value: wanted.clamp(MIN_LIMIT, self.limits.max_limit),
            clamped: wanted > self.limits.max_limit,
        }
    }

    /// Holds a requested depth to `1..=max_depth`; zero is not a depth.
    pub(crate) fn apply_depth(&self, requested: Option<u32>) -> Result<AppliedDepth, McpError> {
        let wanted = requested.unwrap_or(self.limits.default_depth);
        if wanted == 0 {
            return Err(McpError::ArgumentsInvalid(ARG_DEPTH.to_owned()));
        }
        Ok(AppliedDepth {
            value: wanted.min(self.limits.max_depth),
            clamped: wanted > self.limits.max_depth,
        })
    }

    /// The deepest walk the configuration allows.
    pub(crate) const fn max_depth(&self) -> u32 {
        self.limits.max_depth
    }

    /// The largest number of results a caller may ask for.
    pub(crate) const fn max_limit(&self) -> u32 {
        self.limits.max_limit
    }

    /// Finds the registered project and its newest snapshot.
    pub(crate) fn scope(&self, requested: Option<u64>) -> Result<Scope, McpError> {
        let id = requested.map_or(self.default_project, ProjectId::new);
        let project = self.projects.project(id).map_err(|error| {
            if error.code() == ErrorCode::ProjectNotFound {
                McpError::ProjectNotRegistered(id.get())
            } else {
                McpError::from(error)
            }
        })?;
        let snapshot = self
            .index
            .latest_snapshot(id)?
            .ok_or(McpError::ProjectNotScanned(id.get()))?;
        let age = self.clock.now().0.saturating_sub(snapshot.created.0);
        let info = SnapshotInfo {
            id: snapshot.id.get(),
            age_seconds: u64::try_from(age).unwrap_or_default(),
        };
        Ok(Scope {
            project,
            snapshot,
            info,
        })
    }

    pub(crate) fn index(&self) -> &dyn CodeIndex {
        self.index.as_ref()
    }

    pub(crate) fn usage_index(&self) -> &dyn UsageIndex {
        self.usages.as_ref()
    }

    /// The symbol `text` names (an id or a qualified name) in the snapshot of `scope`.
    pub(crate) fn find_symbol(&self, scope: &Scope, text: &str) -> Result<SymbolDetail, McpError> {
        let key = parse_key(text)?;
        self.index()
            .symbol(scope.snapshot.id, &key)?
            .ok_or_else(|| McpError::SymbolNotFound(text.to_owned()))
    }
}

/// A symbol id if the text is all digits, otherwise a qualified name.
pub(crate) fn parse_key(text: &str) -> Result<SymbolKey, McpError> {
    let invalid = || McpError::ArgumentsInvalid(ARG_SYMBOL.to_owned());
    let digits_only = text.bytes().all(|byte| byte.is_ascii_digit());
    if let Some(id) = digits_only.then(|| text.parse::<u64>().ok()).flatten() {
        return Ok(SymbolKey::Id(SymbolId::new(id)));
    }
    QualifiedName::new(text)
        .map(SymbolKey::Qualified)
        .map_err(|_| invalid())
}

/// The first layer that owns the symbol, by its namespace.
pub(crate) fn layer_of(project: &Project, qualified: &QualifiedName) -> Option<String> {
    project
        .layers
        .iter()
        .find(|layer| layer.owns(qualified.as_str()))
        .map(|layer| layer.name.as_str().to_owned())
}

/// Turns an answer into JSON. Serialising these plain types does not fail; if it ever does, the
/// caller gets the internal error code.
pub(crate) fn to_json<T: Serialize>(answer: &T) -> Result<Value, McpError> {
    serde_json::to_value(answer)
        .map_err(|_| McpError::Port(CodedError::new(ErrorCode::UnexpectedInternalError)))
}

/// The notice that a result was cut to the configured limit (`MCP-501`).
pub(crate) fn truncation_notice(limit: u32, total: Option<usize>) -> CodeReport {
    let mut report = CodeReport::new(
        ErrorCode::ResponseLimitExceeded,
        [(crate::consts::PARAM_LIMIT, limit.to_string())],
    );
    if let Some(total) = total {
        report
            .params
            .insert(crate::consts::PARAM_TOTAL, total.to_string());
    }
    report
}
