//! `codeskimr symbols`, `usages`, `impact`, `context`, `unused` and `preview-rename`: the
//! questions of the MCP tools
//! of the same names, asked from the command line. The tools answer; this module adds what a
//! script needs around the answer: the version of the format, whether the scan is out of date,
//! and a failure as JSON with its registered code.

use std::path::Path;
use std::sync::Arc;

use codeskimr_application::access;
use codeskimr_application::cancel::CancelToken;
use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{Clock, CodeIndex, FileDiscovery, ProjectRepository};
use codeskimr_cli::{
    ContextQuery, ImpactQuery, QueryAction, QueryKind, RenameQuery, RulesQuery, SymbolsQuery,
    UnusedQuery, UsagesQuery, query_code_report, query_error_json, query_json, query_text,
};
use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_domain::{Permission, Principal, ProjectId, ProjectSource};
use codeskimr_i18n::Messages;
use codeskimr_mcp::{
    ContextArgs, ImpactArgs, McpError, RenameArgs, RulesArgs, SearchArgs, Tools, UnusedArgs,
    UsagesArgs,
};
use codeskimr_storage::SqliteStore;
use serde_json::Value;

use crate::annotations_cli::Surroundings;
use crate::clock::SystemClock;
use crate::consts::NEW_VIOLATIONS;
use crate::mcp::project_tools;
use crate::ui_commands::hold;
use crate::wiring::{build_analysis, open_store};
use crate::{config_files, export, principal};

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

/// Asks the tool of the command.
fn ask(tools: &Tools, principal: Principal, action: &QueryAction) -> Result<Value, McpError> {
    match action {
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

/// Holds the state of the rules instead of judging them (`codeskimr rules --hold`): the answer
/// is the day and the number of violations the file now has.
fn held(session: &Session, principal: Principal) -> Result<Value, QueryFailure> {
    access::require(&principal, Permission::Manage)?;
    let id = ProjectId::new(session.project);
    let project = session.store.project(id)?;
    let snapshot = session
        .store
        .latest_snapshot(id)?
        .ok_or_else(|| CodedError::new(ErrorCode::ProjectNotScanned))?;
    let written = hold(
        session.store.as_ref(),
        &project,
        snapshot.id,
        SystemClock.now(),
    )?;
    serde_json::to_value(written)
        .map_err(|_| QueryFailure::from(CodedError::new(ErrorCode::UnexpectedInternalError)))
}

/// Whether the command holds the state of the rules instead of asking a tool.
const fn holds(action: &QueryAction) -> bool {
    matches!(action, QueryAction::Rules(RulesQuery { hold: true, .. }))
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
    let holding = holds(action);
    let answer = if holding {
        held(&session, principal::resolve())?
    } else {
        ask(&session.tools, principal::resolve(), action)?
    };
    let stale = if holding {
        None
    } else {
        stale(&session, around)
    };
    let failing = failing(action, &answer);
    let kind = if holding {
        QueryKind::RulesHeld
    } else {
        kind_of(action)
    };
    let text = if action.target().json {
        query_json(answer, stale)
            .map_err(|_| QueryFailure::from(CodedError::new(ErrorCode::UnexpectedInternalError)))?
    } else {
        query_text(around.messages, around.language, kind, (&answer, stale))
            .map_err(|error| QueryFailure::from(CodedError::caused_by(error)))?
    };
    Ok(Answered { text, failing })
}
