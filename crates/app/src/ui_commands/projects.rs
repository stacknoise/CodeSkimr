//! The commands of the project list: list, add and scan again.

use std::path::Path;
use std::sync::Arc;

use codeskimr_application::access;
use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{LanguageShare, ProjectCatalog, ProjectOverview};
use codeskimr_application::registry::{Command, CommandDescriptor, CommandId, Request};
use codeskimr_cli::ScanSummary;
use codeskimr_domain::{Permission, ProjectId, ProjectSource, ScanMode, Snapshot};
use serde::{Deserialize, Serialize};

use super::Context;
use super::consts::{
    ADD_DESCRIPTION, ADD_TITLE, CANCEL_DESCRIPTION, CANCEL_TITLE, KIND_FOLDER, KIND_GIT,
    LIST_DESCRIPTION, LIST_TITLE, PROJECTS_ADD, PROJECTS_CANCEL, PROJECTS_LIST, PROJECTS_RESCAN,
    RESCAN_DESCRIPTION, RESCAN_TITLE, SHORTCUT_NEW, SHORTCUT_RESCAN,
};
use crate::scan::{open_on, reopen};
use crate::source_arg::{SourceError, classify};

/// The newest scan of a project.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ScanRow {
    /// Id of the snapshot.
    pub snapshot: u64,
    /// When it was taken, in seconds since the Unix epoch.
    pub created: i64,
    /// `full` or `incremental`.
    pub mode: &'static str,
    /// Number of files.
    pub files: u32,
    /// Number of symbols.
    pub symbols: u32,
    /// Number of references.
    pub references: u32,
}

/// One language of a project.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LanguageRow {
    /// Id of the language pack.
    pub language: String,
    /// Number of files.
    pub files: u32,
    /// Lines of code.
    pub loc: u32,
    /// Number of symbols declared in its files.
    pub symbols: u32,
    /// `full` or `basic`, the tier of its language pack; `None` if no pack of that id is installed.
    pub tier: Option<&'static str>,
}

/// One row of the project list.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProjectRow {
    /// Id of the project.
    pub id: u64,
    /// Display name.
    pub name: String,
    /// `folder` or `git`.
    pub kind: &'static str,
    /// Folder path or Git address, never with credentials.
    pub location: String,
    /// Branch of a Git source.
    pub branch: Option<String>,
    /// Commit of the newest snapshot, if it has one.
    pub commit: Option<String>,
    /// The newest scan, `None` before the first one.
    pub scan: Option<ScanRow>,
    /// Number of types.
    pub types: u32,
    /// Lines of code of the whole project.
    pub loc: u32,
    /// Languages, the one with the most lines first.
    pub languages: Vec<LanguageRow>,
    /// Whether a scan of the project is running right now.
    pub running: bool,
}

fn scan_row(snapshot: &Snapshot) -> ScanRow {
    ScanRow {
        snapshot: snapshot.id.get(),
        created: snapshot.created.0,
        mode: snapshot.mode.label(),
        files: snapshot.stats.files,
        symbols: snapshot.stats.symbols,
        references: snapshot.stats.references,
    }
}

fn language_row(share: &LanguageShare, context: &Context) -> LanguageRow {
    LanguageRow {
        language: share.language.to_string(),
        files: share.files,
        loc: share.loc,
        symbols: share.symbols,
        tier: context.tier_of(share.language.as_str()),
    }
}

fn project_row(overview: &ProjectOverview, running: bool, context: &Context) -> ProjectRow {
    let (kind, location, branch) = match &overview.project.source {
        ProjectSource::Folder(folder) => (KIND_FOLDER, folder.to_string(), None),
        ProjectSource::Git { url, branch, .. } => {
            (KIND_GIT, url.to_string(), Some(branch.to_string()))
        }
    };
    ProjectRow {
        id: overview.project.id.get(),
        name: overview.project.name.to_string(),
        kind,
        location,
        branch,
        commit: overview
            .latest
            .as_ref()
            .and_then(|snapshot| snapshot.commit.as_ref())
            .map(ToString::to_string),
        scan: overview.latest.as_ref().map(scan_row),
        types: overview.types,
        loc: overview.languages.iter().map(|share| share.loc).sum(),
        languages: overview
            .languages
            .iter()
            .map(|share| language_row(share, context))
            .collect(),
        running,
    }
}

/// Lists the registered projects with their newest scan.
pub struct ListProjects {
    pub(super) catalog: Arc<dyn ProjectCatalog>,
    pub(super) context: Arc<Context>,
}

impl Command for ListProjects {
    type Input = ();
    type Output = Vec<ProjectRow>;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(PROJECTS_LIST),
        title_key: LIST_TITLE,
        description_key: LIST_DESCRIPTION,
        shortcut: None,
        read_only: true,
        permission: Permission::Read,
    };

    fn execute(&self, _request: Request<'_, ()>) -> Result<Vec<ProjectRow>, CodedError> {
        Ok(self
            .catalog
            .overviews()?
            .iter()
            .map(|overview| {
                project_row(
                    overview,
                    self.context.running.is_running(overview.project.id),
                    &self.context,
                )
            })
            .collect())
    }
}

/// What a scan reports back.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Scanned {
    /// Id of the project.
    pub project: u64,
    /// Id of the snapshot that was written.
    pub snapshot: u64,
    /// Number of scanned files.
    pub files: u32,
    /// Number of symbols found.
    pub symbols: u32,
    /// Number of references found.
    pub references: u32,
    /// Duration in seconds.
    pub seconds: f64,
}

impl Scanned {
    fn of(project: ProjectId, summary: &ScanSummary) -> Self {
        Self {
            project: project.get(),
            snapshot: summary.snapshot,
            files: summary.files,
            symbols: summary.symbols,
            references: summary.references,
            seconds: summary.seconds,
        }
    }
}

/// The input of `projects.add`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct AddInput {
    /// An absolute folder path or a Git address.
    pub source: String,
    /// Branch of a Git source.
    pub branch: Option<String>,
    /// Name of the stored credential a Git source logs in with (`credentials.set`).
    #[serde(default)]
    pub credential: Option<String>,
}

/// Registers a folder or a Git repository as a project and scans it.
pub struct AddProject {
    pub(super) context: Arc<Context>,
}

impl Command for AddProject {
    type Input = AddInput;
    type Output = Scanned;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(PROJECTS_ADD),
        title_key: ADD_TITLE,
        description_key: ADD_DESCRIPTION,
        shortcut: Some(SHORTCUT_NEW),
        read_only: false,
        permission: Permission::RegisterProject,
    };

    fn execute(&self, request: Request<'_, AddInput>) -> Result<Scanned, CodedError> {
        access::require(request.principal, Permission::Scan)?;
        let text = request.input.source.trim();
        let spec = classify(text, request.input.branch.as_deref())?;
        let spec = match request.input.credential.as_deref() {
            Some(name) => spec.with_credential(name)?,
            None => spec,
        };
        if matches!(spec.source, ProjectSource::Folder(_)) && !Path::new(text).is_absolute() {
            return Err(SourceError::NotASource.into());
        }
        let context = &self.context;
        let registering = context.registering();
        let session = open_on(
            *request.principal,
            spec,
            &context.global,
            &context.locations,
            (context.events.clone(), &context.secrets),
        )?;
        drop(registering);
        let id = session.project().id;
        let _scanning = context.running.start(id)?;
        let summary =
            session.scan_with(ScanMode::Full, |token| context.running.attach(id, token))?;
        Ok(Scanned::of(id, &summary))
    }
}

/// The input of `projects.rescan`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct RescanInput {
    /// Id of the project.
    pub project: u64,
}

/// Scans a registered project again, incrementally if it has been scanned before.
pub struct RescanProject {
    pub(super) catalog: Arc<dyn ProjectCatalog>,
    pub(super) context: Arc<Context>,
}

impl Command for RescanProject {
    type Input = RescanInput;
    type Output = Scanned;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(PROJECTS_RESCAN),
        title_key: RESCAN_TITLE,
        description_key: RESCAN_DESCRIPTION,
        shortcut: Some(SHORTCUT_RESCAN),
        read_only: false,
        permission: Permission::Scan,
    };

    fn execute(&self, request: Request<'_, RescanInput>) -> Result<Scanned, CodedError> {
        let id = ProjectId::new(request.input.project);
        let context = &self.context;
        let scanned_before = self
            .catalog
            .overviews()?
            .iter()
            .any(|overview| overview.project.id == id && overview.latest.is_some());
        let session = reopen(
            *request.principal,
            id,
            &context.global,
            &context.locations,
            (context.events.clone(), &context.secrets),
        )?;
        let _scanning = context.running.start(id)?;
        let mode = if scanned_before {
            ScanMode::Incremental
        } else {
            ScanMode::Full
        };
        let summary = session.scan_with(mode, |token| context.running.attach(id, token))?;
        Ok(Scanned::of(id, &summary))
    }
}

/// The input of `projects.cancel`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct CancelInput {
    /// Id of the project.
    pub project: u64,
}

/// The output of `projects.cancel`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Cancelled {
    /// Whether a running scan was asked to stop; `false` if none was running.
    pub cancelled: bool,
}

/// Asks the running scan of a project to stop at the next stage boundary.
pub struct CancelScan {
    pub(super) context: Arc<Context>,
}

impl Command for CancelScan {
    type Input = CancelInput;
    type Output = Cancelled;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(PROJECTS_CANCEL),
        title_key: CANCEL_TITLE,
        description_key: CANCEL_DESCRIPTION,
        shortcut: None,
        read_only: false,
        permission: Permission::Scan,
    };

    fn execute(&self, request: Request<'_, CancelInput>) -> Result<Cancelled, CodedError> {
        access::require(request.principal, Permission::Scan)?;
        let id = ProjectId::new(request.input.project);
        Ok(Cancelled {
            cancelled: self.context.running.cancel(id),
        })
    }
}
