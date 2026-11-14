//! The tools and the context they share. Everything here is synchronous; `server` wraps it for
//! the protocol.

mod architecture;
mod constants;
mod conventions;
mod dependencies;
mod project;
mod rescan;
mod search;
mod similar;
mod symbol;
mod usages;

use std::sync::Arc;

use codeskimr_application::config::McpLimits;
use codeskimr_application::conventions::ConventionChecker;
use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{
    Clock, CodeIndex, ProjectCatalog, ProjectRepository, ProjectScanner, ProjectToolCheck,
    SimilarityIndex, SnippetFingerprinter, SymbolDetail, SymbolKey, UsageIndex,
};
use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_domain::{Project, ProjectId, QualifiedName, Snapshot, SymbolId};
use codeskimr_i18n::Messages;
use serde::Serialize;
use serde_json::Value;

use crate::consts::{ARG_DEPTH, ARG_SYMBOL, MIN_LIMIT, PARAM_PROJECT};
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
    /// The stored fingerprints, for `find_similar_code`.
    pub similarity: Arc<dyn SimilarityIndex>,
    /// Fingerprints of a snippet, with the same normalisation as a scan.
    pub snippets: Arc<dyn SnippetFingerprinter>,
    /// The convention check, which reads the files of a project afresh.
    pub checker: Arc<ConventionChecker>,
    /// The time, for the age of a snapshot.
    pub clock: Arc<dyn Clock>,
    /// Scans a project again, for `rescan`.
    pub scanner: Arc<dyn ProjectScanner>,
    /// The languages of a project, for `get_project`.
    pub catalog: Arc<dyn ProjectCatalog>,
    /// Checks the tools that level 2 needs, for `get_project` and `rescan`.
    pub tool_check: Arc<dyn ProjectToolCheck>,
    /// The texts of the tools (what they are for, how to install them).
    pub messages: Arc<Messages>,
}

/// The tools of the server; all of them only read except `rescan`.
pub struct Tools {
    projects: Arc<dyn ProjectRepository>,
    index: Arc<dyn CodeIndex>,
    usages: Arc<dyn UsageIndex>,
    similarity: Arc<dyn SimilarityIndex>,
    snippets: Arc<dyn SnippetFingerprinter>,
    checker: Arc<ConventionChecker>,
    clock: Arc<dyn Clock>,
    scanner: Arc<dyn ProjectScanner>,
    catalog: Arc<dyn ProjectCatalog>,
    tool_check: Arc<dyn ProjectToolCheck>,
    messages: Arc<Messages>,
    limits: McpLimits,
    default_project: Option<ProjectId>,
}

impl Tools {
    /// Builds the tools. `default_project` answers calls that name no project.
    #[must_use]
    pub fn new(ports: Ports, limits: McpLimits, default_project: ProjectId) -> Self {
        Self::build(ports, limits, Some(default_project))
    }

    /// Builds tools without a default project, for a server that serves every registered
    /// project (the app). A call that names no project is `MCP-101` with `params.detail =
    /// project`.
    #[must_use]
    pub fn unscoped(ports: Ports, limits: McpLimits) -> Self {
        Self::build(ports, limits, None)
    }

    fn build(ports: Ports, limits: McpLimits, default_project: Option<ProjectId>) -> Self {
        Self {
            projects: ports.projects,
            index: ports.index,
            usages: ports.usages,
            similarity: ports.similarity,
            snippets: ports.snippets,
            checker: ports.checker,
            clock: ports.clock,
            scanner: ports.scanner,
            catalog: ports.catalog,
            tool_check: ports.tool_check,
            messages: ports.messages,
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

    /// Finds the registered project, without looking at its snapshots.
    pub(crate) fn project(&self, requested: Option<u64>) -> Result<Project, McpError> {
        let id = match (requested, self.default_project) {
            (Some(id), _) => ProjectId::new(id),
            (None, Some(default)) => default,
            (None, None) => return Err(McpError::ArgumentsInvalid(PARAM_PROJECT.to_owned())),
        };
        self.projects.project(id).map_err(|error| {
            if error.code() == ErrorCode::ProjectNotFound {
                McpError::ProjectNotRegistered(id.get())
            } else {
                McpError::from(error)
            }
        })
    }

    /// Finds the registered project and its newest snapshot.
    pub(crate) fn scope(&self, requested: Option<u64>) -> Result<Scope, McpError> {
        let project = self.project(requested)?;
        let id = project.id;
        let snapshot = self
            .index
            .latest_snapshot(id)?
            .ok_or(McpError::ProjectNotScanned(id.get()))?;
        let info = self.snapshot_info(&snapshot);
        Ok(Scope {
            project,
            snapshot,
            info,
        })
    }

    /// Which snapshot it is and how old it is now.
    pub(crate) fn snapshot_info(&self, snapshot: &Snapshot) -> SnapshotInfo {
        let age = self.clock.now().0.saturating_sub(snapshot.created.0);
        SnapshotInfo {
            id: snapshot.id.get(),
            age_seconds: u64::try_from(age).unwrap_or_default(),
        }
    }

    pub(crate) fn scanner(&self) -> &dyn ProjectScanner {
        self.scanner.as_ref()
    }

    pub(crate) fn catalog(&self) -> &dyn ProjectCatalog {
        self.catalog.as_ref()
    }

    pub(crate) fn tool_check(&self) -> &dyn ProjectToolCheck {
        self.tool_check.as_ref()
    }

    /// The text of a message key of the language packs, or the key if there is none.
    pub(crate) fn text(&self, key: &str) -> String {
        self.messages
            .format(crate::consts::TEXT_LANGUAGE, key, None)
            .unwrap_or_else(|_| key.to_owned())
    }

    pub(crate) fn index(&self) -> &dyn CodeIndex {
        self.index.as_ref()
    }

    pub(crate) fn similarity(&self) -> &dyn SimilarityIndex {
        self.similarity.as_ref()
    }

    pub(crate) fn snippets(&self) -> &dyn SnippetFingerprinter {
        self.snippets.as_ref()
    }

    pub(crate) fn checker(&self) -> &ConventionChecker {
        self.checker.as_ref()
    }

    pub(crate) const fn limits(&self) -> &McpLimits {
        &self.limits
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
