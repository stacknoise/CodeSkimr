//! The commands behind the scope of a project: which languages and folders its scans cover
//! (ADR 0102). The code stays one project; nothing is loaded twice.

use std::io::Read;
use std::path::Path;
use std::sync::Arc;

use codeskimr_application::abapgit::{
    AbapGitConfig, AbapGitSummary, CONFIG_FILE, parse_config, summarize,
};
use codeskimr_application::access;
use codeskimr_application::cancel::CancelToken;
use codeskimr_application::config::Config;
use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{ListedFile, ProjectRepository, ScopeStore};
use codeskimr_application::registry::{Command, CommandDescriptor, CommandId, Request};
use codeskimr_application::scope::{
    FolderEntry, LanguageEntry, OverviewLimits, ScopeView, checked_scope, overview, scope_view,
};
use codeskimr_domain::{Permission, ProjectId, ProjectSource, Scope};
use codeskimr_storage::SqliteStore;
use serde::{Deserialize, Serialize};

use super::consts::{
    SCOPE_GET, SCOPE_GET_DESCRIPTION, SCOPE_GET_TITLE, SCOPE_INSPECT, SCOPE_INSPECT_DESCRIPTION,
    SCOPE_INSPECT_TITLE, SCOPE_SET, SCOPE_SET_DESCRIPTION, SCOPE_SET_TITLE,
};
use super::folder_languages;
use crate::consts::ABAPGIT_CONFIG_MAX_BYTES;
use crate::locations::Locations;
use crate::source_arg::classify;
use crate::wiring::{ScopeTools, scope_tools};

/// The input of `scope.get`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct GetInput {
    /// Id of the project.
    pub project: u64,
}

/// A language with its number of files.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LanguageCount {
    /// Id of the language pack.
    pub language: String,
    /// Number of files.
    pub files: u64,
}

/// A language on the scope page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LanguageOut {
    /// Id of the language pack.
    pub language: String,
    /// Its files, `null` if they were not counted.
    pub files: Option<u64>,
    /// Whether scans cover it.
    pub covered: bool,
}

/// A folder on the scope page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FolderOut {
    /// The folder, relative to the project folder, with `/`.
    pub path: String,
    /// Its files of known languages, `null` if they were not counted.
    pub files: Option<u64>,
    /// Whether scans cover it.
    pub covered: bool,
    /// Whether the scope names this very folder.
    pub left_out: bool,
    /// Its files by language, the most files first.
    pub languages: Vec<LanguageCount>,
    /// The files in the folder itself that an indexer needs for the exact check (level 2).
    pub project_files: Vec<String>,
}

/// What a project leaves out.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScopeOut {
    /// The languages that are left out.
    pub languages: Vec<String>,
    /// The folders that are left out, with everything below them.
    pub folders: Vec<String>,
}

/// The answer of `scope.get`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScopeAnswer {
    /// Id of the project.
    pub project: u64,
    /// Whether the counts come from the folder as it is now (a folder project) or from the files
    /// of the newest scan (a Git project, which counts what is covered only).
    pub live: bool,
    /// What the project leaves out now.
    pub scope: ScopeOut,
    /// The languages, the ones with the most files first, then those that are only left out.
    pub languages: Vec<LanguageOut>,
    /// The folders in path order.
    pub folders: Vec<FolderOut>,
    /// Folders that exist but are not listed (`scan.overview_max_folders`).
    pub folders_more: u64,
    /// The files in the project folder itself that an indexer needs.
    pub root_project_files: Vec<String>,
    /// What the project holds if it is an abapGit repository (ADR 0113), else `null`.
    pub abapgit: Option<AbapGitOut>,
    /// Whether the project has files of a language whose pack takes the folders as namespaces
    /// (ABAP) but no abapGit layout was recognised: no `.abapgit.xml` and no objects below `src/`
    /// (issue #845). The files are read, without packages.
    pub abapgit_unrecognised: bool,
}

/// The number of objects of one type in an abapGit repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ObjectCount {
    /// The object type in upper case: `CLAS`, `INTF`, `PROG`, `FUGR`, `TABL` …
    pub kind: String,
    /// Number of objects of the type.
    pub objects: u64,
}

/// What an abapGit repository holds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AbapGitOut {
    /// The folder below which the objects lie, relative to the project; empty for the project
    /// folder itself.
    pub starting_folder: String,
    /// How packages are turned into folders: `prefix` or `full`.
    pub folder_logic: String,
    /// Whether `.abapgit.xml` was read; if not, the defaults were used.
    pub configured: bool,
    /// Objects by type, the type with the most objects first.
    pub kinds: Vec<ObjectCount>,
    /// Number of packages (folders that hold objects).
    pub packages: u64,
    /// Number of files that hold source.
    pub source_files: u64,
}

fn abapgit_out(summary: AbapGitSummary) -> AbapGitOut {
    AbapGitOut {
        starting_folder: summary.starting_folder,
        folder_logic: summary.folder_logic.label().to_owned(),
        configured: summary.configured,
        kinds: summary
            .objects
            .into_iter()
            .map(|(kind, objects)| ObjectCount { kind, objects })
            .collect(),
        packages: summary.packages,
        source_files: summary.source_files,
    }
}

/// The configuration in `.abapgit.xml` of a project folder, if the file is there and is a regular
/// file; it is read through `fsguard` and not further than `ABAPGIT_CONFIG_MAX_BYTES`.
pub(super) fn read_config(root: &Path) -> Option<AbapGitConfig> {
    let file = codeskimr_fsguard::open_regular(&root.join(CONFIG_FILE)).ok()?;
    let mut text = String::new();
    file.take(ABAPGIT_CONFIG_MAX_BYTES)
        .read_to_string(&mut text)
        .ok()?;
    Some(parse_config(&text))
}

fn scope_out(scope: &Scope) -> ScopeOut {
    ScopeOut {
        languages: scope.languages().iter().map(ToString::to_string).collect(),
        folders: scope.folders().iter().map(ToString::to_string).collect(),
    }
}

fn language_out(entry: &LanguageEntry) -> LanguageOut {
    LanguageOut {
        language: entry.language.to_string(),
        files: entry.files,
        covered: entry.covered,
    }
}

fn folder_out(entry: FolderEntry) -> FolderOut {
    FolderOut {
        path: entry.path.to_string(),
        files: entry.files,
        covered: entry.covered,
        left_out: entry.left_out,
        languages: entry
            .languages
            .into_iter()
            .map(|(language, files)| LanguageCount {
                language: language.to_string(),
                files,
            })
            .collect(),
        project_files: entry.project_files,
    }
}

fn answer(
    project: ProjectId,
    (scope, live): (&Scope, bool),
    (view, abapgit, unrecognised): (ScopeView, Option<AbapGitSummary>, bool),
) -> ScopeAnswer {
    ScopeAnswer {
        project: project.get(),
        live,
        scope: scope_out(scope),
        languages: view.languages.iter().map(language_out).collect(),
        folders: view.folders.into_iter().map(folder_out).collect(),
        folders_more: view.folders_more,
        root_project_files: view.root_project_files,
        abapgit: abapgit.map(abapgit_out),
        abapgit_unrecognised: unrecognised,
    }
}

/// Shows what a project covers and what it leaves out.
pub struct ScopeGet {
    pub(super) store: Arc<SqliteStore>,
    pub(super) config: Config,
    pub(super) locations: Locations,
}

impl ScopeGet {
    /// The files to count: the folder as it is now for a folder project, the files of the newest
    /// scan for a Git project. The second element tells which.
    fn files(
        &self,
        id: ProjectId,
        source: &ProjectSource,
        tools: &ScopeTools,
        cancel: &CancelToken,
    ) -> Result<(Vec<ListedFile>, bool), CodedError> {
        match source {
            ProjectSource::Folder(root) => Ok((tools.lister.list(root, cancel)?, true)),
            ProjectSource::Git { .. } => {
                let files = self.store.latest_files(id)?;
                let listed = files.into_iter().map(|file| ListedFile {
                    path: file.path,
                    language: Some(file.language),
                });
                Ok((listed.collect(), false))
            }
        }
    }
}

impl ScopeGet {
    /// Whether any of `files` is of a language whose pack takes the folders as namespaces.
    fn has_folder_language(&self, files: &[ListedFile]) -> bool {
        let languages = folder_languages(&self.locations.languages_dir);
        files
            .iter()
            .filter_map(|file| file.language.as_ref())
            .any(|language| languages.contains(language))
    }
}

impl Command for ScopeGet {
    type Input = GetInput;
    type Output = ScopeAnswer;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(SCOPE_GET),
        title_key: SCOPE_GET_TITLE,
        description_key: SCOPE_GET_DESCRIPTION,
        shortcut: None,
        read_only: true,
        permission: Permission::Read,
    };

    /// # Errors
    /// `AUTH-401` without the right to read, `PRJ-201` for an unknown project, `SRC-201` if the
    /// folder cannot be read, `PRS` failures if the language packs cannot be loaded.
    fn execute(&self, request: Request<'_, Self>) -> Result<ScopeAnswer, CodedError> {
        access::require(request.principal, Permission::Read)?;
        let id = ProjectId::new(request.input.project);
        let project = self.store.project(id)?;
        let scope = self.store.scope(id)?;
        let tools = scope_tools(&self.config, &self.locations)?;
        let (files, live) = self.files(id, &project.source, &tools, &CancelToken::new())?;
        let limits = OverviewLimits {
            depth: self.config.scan.overview_depth,
            max_folders: self.config.scan.overview_max_folders,
        };
        let counted = overview(&files, &tools.project_files, limits);
        let config = match &project.source {
            ProjectSource::Folder(root) => read_config(Path::new(root.as_str())),
            ProjectSource::Git { .. } => None,
        };
        let abapgit = summarize(&files, config.as_ref());
        let unrecognised = abapgit.is_none() && self.has_folder_language(&files);
        Ok(answer(
            id,
            (&scope, live),
            (scope_view(&scope, counted), abapgit, unrecognised),
        ))
    }
}

/// The input of `scope.inspect`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct InspectInput {
    /// The folder or Git address that the user typed in the window "New project".
    pub source: String,
}

/// The answer of `scope.inspect`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Inspected {
    /// What the folder holds if it is an abapGit repository (it has a `.abapgit.xml`), else `null`.
    pub abapgit: Option<AbapGitOut>,
}

/// Looks into a folder before it becomes a project, so that the window "New project" can say what
/// it is (an abapGit repository, ADR 0113). Only a folder with a `.abapgit.xml` is looked into;
/// every other text, a Git address and a missing folder included, gets an empty answer.
pub struct ScopeInspect {
    pub(super) config: Config,
    pub(super) locations: Locations,
}

impl Command for ScopeInspect {
    type Input = InspectInput;
    type Output = Inspected;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(SCOPE_INSPECT),
        title_key: SCOPE_INSPECT_TITLE,
        description_key: SCOPE_INSPECT_DESCRIPTION,
        shortcut: None,
        // Not "read only": it looks the folder up on disk, like `projects.suggest_name`.
        read_only: false,
        permission: Permission::RegisterProject,
    };

    /// # Errors
    /// `AUTH-401` without the right to register projects, `SRC-201` if the folder cannot be
    /// listed, `PRS` failures if the language packs cannot be loaded.
    fn execute(&self, request: Request<'_, Self>) -> Result<Inspected, CodedError> {
        access::require(request.principal, Permission::RegisterProject)?;
        let none = Inspected { abapgit: None };
        let Ok(spec) = classify(request.input.source.trim(), None) else {
            return Ok(none);
        };
        let ProjectSource::Folder(root) = &spec.source else {
            return Ok(none);
        };
        let Some(config) = read_config(Path::new(root.as_str())) else {
            return Ok(none);
        };
        let tools = scope_tools(&self.config, &self.locations)?;
        let files = tools.lister.list(root, &CancelToken::new())?;
        Ok(Inspected {
            abapgit: summarize(&files, Some(&config)).map(abapgit_out),
        })
    }
}

/// The input of `scope.set`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SetInput {
    /// Id of the project.
    pub project: u64,
    /// The languages the scans leave out; they replace the ones that were left out.
    pub languages: Vec<String>,
    /// The folders the scans leave out, relative to the project folder; they replace the ones that
    /// were left out.
    pub folders: Vec<String>,
}

/// The answer of `scope.set`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScopeSaved {
    /// Id of the project.
    pub project: u64,
    /// What the project leaves out now.
    pub scope: ScopeOut,
    /// Whether the scope differs from the one before; if so, the next scan reads another set of
    /// files.
    pub changed: bool,
}

/// Stores what a project leaves out.
pub struct ScopeSet {
    pub(super) store: Arc<SqliteStore>,
    pub(super) max_entries: u32,
}

impl Command for ScopeSet {
    type Input = SetInput;
    type Output = ScopeSaved;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(SCOPE_SET),
        title_key: SCOPE_SET_TITLE,
        description_key: SCOPE_SET_DESCRIPTION,
        shortcut: None,
        read_only: false,
        permission: Permission::RegisterProject,
    };

    /// # Errors
    /// `AUTH-401` without the right to register projects, `PRJ-103` for an invalid scope,
    /// `PRJ-201` for an unknown project.
    fn execute(&self, request: Request<'_, Self>) -> Result<ScopeSaved, CodedError> {
        access::require(request.principal, Permission::RegisterProject)?;
        let id = ProjectId::new(request.input.project);
        let wanted = checked_scope(
            &request.input.languages,
            &request.input.folders,
            self.max_entries,
        )?;
        let before = self.store.scope(id)?;
        self.store.store_scope(id, &wanted)?;
        Ok(ScopeSaved {
            project: id.get(),
            scope: scope_out(&wanted),
            changed: before != wanted,
        })
    }
}
