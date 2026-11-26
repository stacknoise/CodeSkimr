//! `codeskimr symbols`, `usages`, `impact`, `context`, `unused` and `preview-rename`: the
//! questions of the MCP tools
//! of the same names, asked from the command line; and `scans`, `compare` and `change`, which ask
//! the history and the comparison of scans (ADR 0097). The tools answer; this module adds what a
//! script needs around the answer: the version of the format, whether the scan is out of date,
//! and a failure as JSON with its registered code.

use std::path::Path;
use std::sync::Arc;

use codeskimr_application::cancel::CancelToken;
use codeskimr_application::compare::{ComparisonAnswer, ComparisonRequest, ScanListAnswer};
use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{CodeIndex, FileDiscovery, ProjectRepository};
use codeskimr_cli::{
    CompareQuery, ContextQuery, ImpactQuery, QueryAction, QueryKind, RenameQuery, RulesQuery,
    SymbolsQuery, UnusedQuery, UsagesQuery, change_markdown, query_code_report, query_error_json,
    query_json, query_text,
};
use codeskimr_codes::i18n::COMPARE_DIRECT_ONLY_NOTE;
use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_domain::{Principal, ProjectId, ProjectSource, SnapshotId};
use codeskimr_i18n::Messages;
use codeskimr_mcp::{
    ContextArgs, ImpactArgs, McpError, RenameArgs, RulesArgs, SearchArgs, Tools, UnusedArgs,
    UsagesArgs,
};
use codeskimr_storage::SqliteStore;
use serde_json::Value;

use crate::annotations_cli::Surroundings;
use crate::consts::NEW_VIOLATIONS;
use crate::mcp::project_tools;
use crate::wiring::{build_analysis, open_store};
use crate::{config_files, export, principal, scan_compare};

/// Why a query did not give an answer: the error as the MCP tools report it, which has the
/// registered code and the parameters that explain it.
#[derive(Debug)]
pub struct QueryFailure(pub McpError);

impl QueryFailure {
    /// The registered code.
    pub fn code(&self) -> ErrorCode {
        self.0.code()
    }

    /// The failure as the JSON of the commands (`schema` and `error`).
    pub fn json(&self) -> String {
        let report = serde_json::to_value(self.0.report())
            .unwrap_or_else(|_| query_code_report(self.code().id()));
        query_error_json(&report)
    }
}

impl From<CodedError> for QueryFailure {
    fn from(error: CodedError) -> Self {
        Self(McpError::from(error))
    }
}

impl From<McpError> for QueryFailure {
    fn from(error: McpError) -> Self {
        Self(error)
    }
}

/// What the commands ask over: the tools of the project and the store they read.
struct Session {
    tools: Tools,
    store: Arc<SqliteStore>,
    project: u64,
    config: codeskimr_application::config::Config,
}

fn open(
    action: &QueryAction,
    around: &Surroundings<'_>,
    current_dir: &Path,
) -> Result<Session, QueryFailure> {
    let project = action.target().project.as_deref();
    let folder = project.is_none().then_some(current_dir);
    let config = config_files::load(around.locations, around.global, folder)?;
    // Nothing was ever scanned if there is no database; opening the store would create it.
    if !around.locations.database().exists() {
        return Err(CodedError::new(ErrorCode::ProjectNotFound).into());
    }
    let store = open_store(around.locations, &config)?;
    let named = project.map_or_else(|| current_dir.to_string_lossy().into_owned(), str::to_owned);
    let project = export::resolve(&named, &store)?;
    let messages = Arc::new(Messages::load().map_err(CodedError::caused_by)?);
    let id = ProjectId::new(project);
    let tools = project_tools(
        Arc::clone(&store),
        &config,
        (around.locations, &messages),
        id,
    )?;
    Ok(Session {
        tools,
        store,
        project,
        config,
    })
}

/// The answer of `scans`, `compare` and `change`, in the shape of `scans.list`, `scans.compare` and
/// `scans.impact`; the note about what the comparison does not see is in the language of the user.
fn ask_scans(
    session: &Session,
    around: &Surroundings<'_>,
    action: &QueryAction,
) -> Result<Value, QueryFailure> {
    let project = ProjectId::new(session.project);
    match action {
        QueryAction::Scans(_) => {
            let history = session.store.scan_history(project)?;
            let keep = session.config.snapshots.retention_count;
            as_json(&ScanListAnswer::new(&history, keep))
        }
        QueryAction::Compare(query) => compare_answer(session, around, (project, query), false),
        QueryAction::Change(query) => compare_answer(session, around, (project, query), true),
        _ => Err(CodedError::new(ErrorCode::UnexpectedInternalError).into()),
    }
}

fn compare_answer(
    session: &Session,
    around: &Surroundings<'_>,
    (project, query): (ProjectId, &CompareQuery),
    with_impact: bool,
) -> Result<Value, QueryFailure> {
    let comparison =
        scan_compare::comparer(&session.store, &session.config).compare(&ComparisonRequest {
            project,
            base: query.base.map(SnapshotId::new),
            target: query.newer.map(SnapshotId::new),
            with_impact,
        })?;
    let note = around
        .messages
        .format(around.language, COMPARE_DIRECT_ONLY_NOTE, None)
        .map_err(|error| QueryFailure::from(CodedError::caused_by(error)))?;
    as_json(&ComparisonAnswer::new(&comparison, Some(note)))
}

/// An answer as JSON; serialising these plain types does not fail.
fn as_json<T: serde::Serialize>(answer: &T) -> Result<Value, QueryFailure> {
    serde_json::to_value(answer)
        .map_err(|_| CodedError::new(ErrorCode::UnexpectedInternalError).into())
}

/// Asks the tool of the command.
fn ask(tools: &Tools, principal: Principal, action: &QueryAction) -> Result<Value, McpError> {
    match action {
        QueryAction::Scans(_) | QueryAction::Compare(_) | QueryAction::Change(_) => Err(
            McpError::from(CodedError::new(ErrorCode::UnexpectedInternalError)),
        ),
        QueryAction::Symbols(query) => tools.search_symbols(&principal, &search_args(query)),
        QueryAction::Usages(query) => tools.find_usages(&principal, &usages_args(query)),
        QueryAction::Impact(query) => tools.impact_of_change(&principal, &impact_args(query)),
        QueryAction::Context(query) => tools.get_context(&principal, &context_args(query)),
        QueryAction::Unused(query) => tools.find_unused(&principal, &unused_args(query)),
        QueryAction::Rules(query) => tools.check_rules(&principal, &rules_args(query)),
        QueryAction::Rename(query) => tools.preview_rename(&principal, &rename_args(query)),
    }
}

fn rename_args(query: &RenameQuery) -> RenameArgs {
    RenameArgs {
        symbol: query.symbol.clone(),
        new_name: query.new_name.clone(),
        project: None,
        limit: query.limit,
    }
}

fn rules_args(query: &RulesQuery) -> RulesArgs {
    RulesArgs {
        project: None,
        limit: query.limit,
    }
}

fn unused_args(query: &UnusedQuery) -> UnusedArgs {
    UnusedArgs {
        kind: query.kind.clone(),
        language: query.language.clone(),
        assessment: query.assessment.clone(),
        include_tests: Some(query.include_tests),
        project: None,
        limit: query.limit,
    }
}

fn search_args(query: &SymbolsQuery) -> SearchArgs {
    SearchArgs {
        query: query.query.clone(),
        kind: query.kind.clone(),
        project: None,
        limit: query.limit,
    }
}

fn usages_args(query: &UsagesQuery) -> UsagesArgs {
    UsagesArgs {
        symbol: query.symbol.clone(),
        direction: query.direction.clone(),
        depth: query.depth,
        include_tests: Some(query.include_tests),
        project: None,
        limit: query.limit,
    }
}

fn impact_args(query: &ImpactQuery) -> ImpactArgs {
    ImpactArgs {
        symbol: query.symbol.clone(),
        project: None,
        limit: query.limit,
    }
}

fn context_args(query: &ContextQuery) -> ContextArgs {
    ContextArgs {
        symbol: query.symbol.clone(),
        budget: query.budget.clone(),
        project: None,
    }
}

/// Whether files of the project changed since the newest scan: `None` if that cannot be told.
/// Only a folder can be looked at; a Git source is a copy that only a scan updates. The files
/// are hashed as a scan hashes them, but not parsed, and only the files of the scope of the
/// project count (ADR 0102).
fn stale(session: &Session, around: &Surroundings<'_>) -> Option<bool> {
    let id = ProjectId::new(session.project);
    let ProjectSource::Folder(root) = session.store.project(id).ok()?.source else {
        return None;
    };
    let previous = session.store.latest_files(id).ok()?;
    let scope = session.store.scope(id).ok()?;
    let discovery: Arc<dyn FileDiscovery> = build_analysis(&session.config, around.locations)
        .ok()?
        .discovery;
    let found = discovery
        .discover_scoped(&root, &previous, &scope, &CancelToken::new())
        .ok()?;
    Some(found.len() < previous.len() || found.iter().any(|file| file.changed))
}

const fn kind_of(action: &QueryAction) -> QueryKind {
    match action {
        QueryAction::Symbols(_) => QueryKind::Symbols,
        QueryAction::Usages(_) => QueryKind::Usages,
        QueryAction::Impact(_) => QueryKind::Impact,
        QueryAction::Context(_) => QueryKind::Context,
        QueryAction::Unused(_) => QueryKind::Unused,
        QueryAction::Rules(_) => QueryKind::Rules,
        QueryAction::Rename(_) => QueryKind::Rename,
        QueryAction::Scans(_) => QueryKind::Scans,
        QueryAction::Compare(_) => QueryKind::Compare,
        QueryAction::Change(_) => QueryKind::Change,
    }
}

/// What a query command prints, and whether the answer is one that fails a check.
pub struct Answered {
    /// JSON with `--json`, text otherwise.
    pub text: String,
    /// Whether the answer holds findings that fail a CI run: new violations of `rules`.
    pub failing: bool,
}

/// Whether the answer of the command fails a check: only `rules` can, and only with new violations.
fn failing(action: &QueryAction, answer: &Value) -> bool {
    matches!(action, QueryAction::Rules(_))
        && answer
            .get(NEW_VIOLATIONS)
            .and_then(Value::as_u64)
            .unwrap_or_default()
            > 0
}

/// Whether the command asks the history of scans, not the tools.
const fn is_history(action: &QueryAction) -> bool {
    matches!(
        action,
        QueryAction::Scans(_) | QueryAction::Compare(_) | QueryAction::Change(_)
    )
}

/// Runs the query command and returns what to print: JSON with `--json`, text otherwise.
///
/// # Errors
/// - `PRJ-201` if the project is not registered, `MCP-203` if it has not been scanned.
/// - `MCP-101`, `MCP-202` and the other codes of the tool.
/// - Any coded failure of the configuration, the storage or the message texts.
pub fn run(
    action: &QueryAction,
    around: &Surroundings<'_>,
    current_dir: &Path,
) -> Result<Answered, QueryFailure> {
    let session = open(action, around, current_dir)?;
    let history = is_history(action);
    let answer = if history {
        ask_scans(&session, around, action)?
    } else {
        ask(&session.tools, principal::resolve(), action)?
    };
    let stale = if history {
        None
    } else {
        stale(&session, around)
    };
    let failing = failing(action, &answer);
    let text = if let QueryAction::Change(CompareQuery { markdown: true, .. }) = action {
        change_markdown(around.messages, around.language, &answer)
            .map_err(|error| QueryFailure::from(CodedError::caused_by(error)))?
    } else if action.target().json {
        query_json(answer, stale)
            .map_err(|_| QueryFailure::from(CodedError::new(ErrorCode::UnexpectedInternalError)))?
    } else {
        query_text(
            around.messages,
            around.language,
            kind_of(action),
            (&answer, stale),
        )
        .map_err(|error| QueryFailure::from(CodedError::caused_by(error)))?
    };
    Ok(Answered { text, failing })
}
