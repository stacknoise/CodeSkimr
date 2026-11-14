//! The command of the quick search: fuzzy search over the symbol names of the newest scan.

use std::sync::Arc;

use codeskimr_application::access;
use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{CodeIndex, ProjectRepository, SearchHit};
use codeskimr_application::registry::{Command, CommandDescriptor, CommandId, Request};
use codeskimr_domain::{Permission, ProjectId, SymbolId};
use serde::{Deserialize, Serialize};

use super::consts::{SEARCH_DESCRIPTION, SEARCH_LIMIT, SEARCH_QUERY, SEARCH_TITLE};

/// One hit of the quick search.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HitOut {
    /// Id of the symbol.
    pub id: u64,
    /// Simple name.
    pub name: String,
    /// Qualified name.
    pub qualified_name: String,
    /// Kind label.
    pub kind: &'static str,
    /// Path of the declaring file.
    pub path: String,
    /// The enclosing symbol, so a member can be shown within its type.
    pub parent: Option<u64>,
}

/// The answer of `search.query`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SearchOut {
    /// Id of the snapshot searched, `None` before the first scan.
    pub snapshot: Option<u64>,
    /// Number of symbols the snapshot holds, which is what the search runs over; `None` before the
    /// first scan.
    pub entries: Option<u32>,
    /// The best hits first.
    pub hits: Vec<HitOut>,
}

/// The input of `search.query`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SearchInput {
    /// Id of the project.
    pub project: u64,
    /// The text to search for.
    pub query: String,
}

/// What the search reads: the project repository and the index.
pub struct SearchPorts {
    pub(super) projects: Arc<dyn ProjectRepository>,
    pub(super) index: Arc<dyn CodeIndex>,
}

fn hit_out(hit: &SearchHit) -> HitOut {
    HitOut {
        id: hit.symbol.get(),
        name: hit.name.to_string(),
        qualified_name: hit.qualified_name.to_string(),
        kind: hit.kind.label(),
        path: hit.path.to_string(),
        parent: hit.parent.map(SymbolId::get),
    }
}

/// Fuzzy search over the names of the newest scan of a project.
pub struct SearchQuery {
    pub(super) ports: Arc<SearchPorts>,
}

impl Command for SearchQuery {
    type Input = SearchInput;
    type Output = SearchOut;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(SEARCH_QUERY),
        title_key: SEARCH_TITLE,
        description_key: SEARCH_DESCRIPTION,
        shortcut: None,
        read_only: true,
        permission: Permission::Read,
    };

    fn execute(&self, request: Request<'_, SearchInput>) -> Result<SearchOut, CodedError> {
        access::require(request.principal, Permission::Read)?;
        let ports = &self.ports;
        let id = ProjectId::new(request.input.project);
        ports.projects.project(id)?;
        let Some(snapshot) = ports.index.latest_snapshot(id)? else {
            return Ok(SearchOut {
                snapshot: None,
                entries: None,
                hits: Vec::new(),
            });
        };
        let found = ports
            .index
            .search(snapshot.id, &request.input.query, SEARCH_LIMIT)?;
        Ok(SearchOut {
            snapshot: Some(snapshot.id.get()),
            entries: Some(snapshot.stats.symbols),
            hits: found.iter().map(hit_out).collect(),
        })
    }
}
