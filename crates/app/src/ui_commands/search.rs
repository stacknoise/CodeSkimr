//! The command of the quick search: fuzzy search over the symbol names of the newest scan.

use std::sync::Arc;

use codeskimr_application::access;
use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{
    CodeIndex, FileHit, ProjectCatalog, ProjectRepository, SearchHit,
};
use codeskimr_application::registry::{Command, CommandDescriptor, CommandId, Request};
use codeskimr_domain::{Permission, ProjectId, SymbolId};
use serde::{Deserialize, Serialize};

use super::consts::{
    SEARCH_DESCRIPTION, SEARCH_FILE_LIMIT, SEARCH_LIMIT, SEARCH_MIN_SHARE, SEARCH_QUERY,
    SEARCH_TITLE,
};

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
    /// Id of the project the symbol belongs to; a search over all projects mixes them.
    pub project: u64,
    /// Name of that project.
    pub project_name: String,
}

/// One file of the quick search: a file that declares at least one type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FileOut {
    /// Path of the file.
    pub path: String,
    /// Lines of code of the file.
    pub loc: u32,
    /// The first type of the file, which is where the explorer opens it.
    pub first_type: u64,
    /// Id of the project the file belongs to.
    pub project: u64,
    /// Name of that project.
    pub project_name: String,
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
    /// The files whose path matches, best first.
    pub files: Vec<FileOut>,
}

/// The input of `search.query`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SearchInput {
    /// Id of the project.
    pub project: u64,
    /// The text to search for.
    pub query: String,
    /// Search the newest scan of every project, not only `project` (`@` in the quick search).
    #[serde(default)]
    pub all_projects: bool,
}

/// What the search reads: the project repository and the index.
pub struct SearchPorts {
    pub(super) projects: Arc<dyn ProjectRepository>,
    pub(super) index: Arc<dyn CodeIndex>,
    pub(super) catalog: Arc<dyn ProjectCatalog>,
}

/// The project a hit belongs to, for the mixed list of a search over all projects.
struct Owner {
    id: u64,
    name: String,
}

fn hit_out(hit: &SearchHit, owner: &Owner) -> HitOut {
    HitOut {
        id: hit.symbol.get(),
        name: hit.name.to_string(),
        qualified_name: hit.qualified_name.to_string(),
        kind: hit.kind.label(),
        path: hit.path.to_string(),
        parent: hit.parent.map(SymbolId::get),
        project: owner.id,
        project_name: owner.name.clone(),
    }
}

fn file_out(file: &FileHit, owner: &Owner) -> FileOut {
    FileOut {
        path: file.path.to_string(),
        loc: file.loc,
        first_type: file.first_type.get(),
        project: owner.id,
        project_name: owner.name.clone(),
    }
}

impl SearchPorts {
    /// The projects a search looks at: the one asked for, or every registered one.
    fn scope(&self, input: &SearchInput) -> Result<Vec<Owner>, CodedError> {
        if input.all_projects {
            return Ok(self
                .catalog
                .overviews()?
                .into_iter()
                .map(|overview| Owner {
                    id: overview.project.id.get(),
                    name: overview.project.name.to_string(),
                })
                .collect());
        }
        let project = self.projects.project(ProjectId::new(input.project))?;
        Ok(vec![Owner {
            id: input.project,
            name: project.name.to_string(),
        }])
    }

    /// Searches the newest scan of one project and adds what it finds to `out`.
    fn search_one(
        &self,
        owner: &Owner,
        query: &str,
        share: u32,
        out: &mut SearchOut,
    ) -> Result<(), CodedError> {
        let Some(snapshot) = self.index.latest_snapshot(ProjectId::new(owner.id))? else {
            return Ok(());
        };
        out.snapshot = out.snapshot.or(Some(snapshot.id.get()));
        out.entries = Some(out.entries.unwrap_or_default() + snapshot.stats.symbols);
        let hits = self.index.search(snapshot.id, query, share)?;
        out.hits.extend(hits.iter().map(|hit| hit_out(hit, owner)));
        let files = self
            .index
            .search_files(snapshot.id, query, SEARCH_FILE_LIMIT)?;
        out.files
            .extend(files.iter().map(|file| file_out(file, owner)));
        Ok(())
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
        let mut out = SearchOut {
            snapshot: None,
            entries: None,
            hits: Vec::new(),
            files: Vec::new(),
        };
        let owners = ports.scope(&request.input)?;
        let count = u32::try_from(owners.len()).unwrap_or(u32::MAX).max(1);
        let share = (SEARCH_LIMIT / count).max(SEARCH_MIN_SHARE);
        for owner in &owners {
            ports.search_one(owner, &request.input.query, share, &mut out)?;
        }
        out.hits
            .truncate(usize::try_from(SEARCH_LIMIT).unwrap_or(usize::MAX));
        out.files
            .truncate(usize::try_from(SEARCH_FILE_LIMIT).unwrap_or(usize::MAX));
        Ok(out)
    }
}
