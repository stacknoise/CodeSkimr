//! The commands of the annotations (ADR 0068): list, count per symbol, add (generic and with the
//! three shortcuts of blueprint 15.4), change and delete. Reading needs `read`, changing needs
//! `annotate`; the registry checks both before a command runs.

use std::marker::PhantomData;
use std::sync::Arc;

use codeskimr_application::annotations::{
    AnnotationQuery, AnnotationWrites, Change, ContentInput, PageRequest, annotations, summaries,
};
use codeskimr_application::config::Annotations;
use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{AlertFilter, AnnotationStore, ProjectRepository};
use codeskimr_application::registry::{Command, CommandDescriptor, CommandId, Request};
use codeskimr_codes::i18n;
use codeskimr_domain::{
    AlertState, AnnotationId, AnnotationKind, Permission, ProjectId, TaskStatus, Timestamp,
};
use serde::{Deserialize, Serialize};

use super::annotation_wire::{
    AnnotationOut, Nullable, PageOut, SummaryOut, TargetIn, invalid, nullable, optional_label,
};
use super::consts::{
    ANNOTATIONS_ADD_NOTE, ANNOTATIONS_ADD_TASK, ANNOTATIONS_CREATE, ANNOTATIONS_DELETE,
    ANNOTATIONS_LIST, ANNOTATIONS_MARK_CRITICAL, ANNOTATIONS_SUMMARY, ANNOTATIONS_UPDATE,
    SHORTCUT_CRITICAL, SHORTCUT_NOTE, SHORTCUT_TASK,
};

/// What the commands of annotations and alerts read and write through.
pub struct AnnotationPorts {
    pub(super) store: Arc<dyn AnnotationStore>,
    pub(super) projects: Arc<dyn ProjectRepository>,
    pub(super) writes: AnnotationWrites,
    pub(super) config: Annotations,
}

impl AnnotationPorts {
    /// The page a caller asked for, held to the configured limits.
    pub(super) fn page(&self, offset: Option<u32>, limit: Option<u32>) -> PageRequest {
        PageRequest::resolve(offset, limit, &self.config)
    }

    /// How many alerts of the project are open.
    pub(super) fn open_alerts(&self, project: ProjectId) -> Result<u32, CodedError> {
        let open = self.store.list_alerts(&AlertFilter {
            project,
            state: Some(AlertState::Open),
        })?;
        Ok(u32::try_from(open.len()).unwrap_or(u32::MAX))
    }

    /// The project, so that an unknown one is `PRJ-201` and not an empty list.
    pub(super) fn project(&self, id: u64) -> Result<ProjectId, CodedError> {
        let id = ProjectId::new(id);
        self.projects.project(id)?;
        Ok(id)
    }
}

/// The input of `annotations.list`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ListInput {
    /// Id of the project.
    pub project: u64,
    /// Only the annotations of this symbol.
    #[serde(default)]
    pub symbol: Option<TargetIn>,
    /// Only this kind: `critical`, `note` or `task`.
    #[serde(default)]
    pub kind: Option<String>,
    /// Only tasks in this status: `open`, `in_progress` or `done`.
    #[serde(default)]
    pub status: Option<String>,
    /// Only annotations whose symbol is gone (`true`) or still there (`false`).
    #[serde(default)]
    pub without_target: Option<bool>,
    /// Rows to skip.
    #[serde(default)]
    pub offset: Option<u32>,
    /// Rows to return, at most `annotations.max_page_size`.
    #[serde(default)]
    pub limit: Option<u32>,
}

/// Lists the annotations of a project.
pub struct AnnotationsList {
    pub(super) ports: Arc<AnnotationPorts>,
}

impl Command for AnnotationsList {
    type Input = ListInput;
    type Output = PageOut<AnnotationOut>;

    const DESCRIPTOR: CommandDescriptor = read_descriptor(
        ANNOTATIONS_LIST,
        (
            i18n::COMMAND_ANNOTATIONS_LIST_TITLE,
            i18n::COMMAND_ANNOTATIONS_LIST_DESCRIPTION,
        ),
    );

    fn execute(&self, request: Request<'_, Self>) -> Result<Self::Output, CodedError> {
        let ports = &self.ports;
        let input = &request.input;
        let query = AnnotationQuery {
            project: ports.project(input.project)?,
            target: input.symbol.as_ref().map(TargetIn::target).transpose()?,
            kind: optional_label(input.kind.as_deref())?,
            status: optional_label(input.status.as_deref())?,
            without_target: input.without_target,
        };
        let page = annotations(
            ports.store.as_ref(),
            &query,
            ports.page(input.offset, input.limit),
        )?;
        Ok(PageOut::of(&page, |row| AnnotationOut::from(row)))
    }
}

/// The input of `annotations.summary`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct SummaryInput {
    /// Id of the project.
    pub project: u64,
    /// Rows to skip.
    #[serde(default)]
    pub offset: Option<u32>,
    /// Rows to return, at most `annotations.max_page_size`.
    #[serde(default)]
    pub limit: Option<u32>,
}

/// One row per annotated symbol of a project, for the marks and counters of the UI.
pub struct AnnotationsSummary {
    pub(super) ports: Arc<AnnotationPorts>,
}

impl Command for AnnotationsSummary {
    type Input = SummaryInput;
    type Output = PageOut<SummaryOut>;

    const DESCRIPTOR: CommandDescriptor = read_descriptor(
        ANNOTATIONS_SUMMARY,
        (
            i18n::COMMAND_ANNOTATIONS_SUMMARY_TITLE,
            i18n::COMMAND_ANNOTATIONS_SUMMARY_DESCRIPTION,
        ),
    );

    fn execute(&self, request: Request<'_, Self>) -> Result<Self::Output, CodedError> {
        let ports = &self.ports;
        let input = request.input;
        let project = ports.project(input.project)?;
        let page = summaries(
            ports.store.as_ref(),
            project,
            ports.page(input.offset, input.limit),
        )?;
        Ok(PageOut::of(&page, |row| SummaryOut::from(row)))
    }
}

/// The input of the commands that add an annotation. The commands with a fixed kind refuse a
/// different `kind`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CreateInput {
    /// Id of the project.
    pub project: u64,
    /// The symbol.
    pub target: TargetIn,
    /// `critical`, `note` or `task`; required by `annotations.create`.
    #[serde(default)]
    pub kind: Option<String>,
    /// Title of a task, required there.
    #[serde(default)]
    pub title: Option<String>,
    /// The text; required for a note, optional for a task, the reason for a critical mark.
    #[serde(default)]
    pub text: Option<String>,
    /// Status of a task, `open` if missing.
    #[serde(default)]
    pub status: Option<String>,
    /// Due date of a task, in seconds since the Unix epoch.
    #[serde(default)]
    pub due: Option<i64>,
    /// Assignee of a task.
    #[serde(default)]
    pub assignee: Option<String>,
}

/// The variants of the add command: they differ in their descriptor and in the kind they fix.
pub trait CreateVariant: Send + Sync + 'static {
    /// The metadata of the command.
    const DESCRIPTOR: CommandDescriptor;
    /// The kind the command fixes; `None` if the caller says it.
    const KIND: Option<AnnotationKind>;
}

/// `annotations.create`: any kind.
pub struct AnyKind;
/// `annotations.mark_critical`, key `K`.
pub struct CriticalMark;
/// `annotations.add_note`, key `N`.
pub struct NoteKind;
/// `annotations.add_task`, key `T`.
pub struct TaskKind;

/// The descriptor of a command that only reads.
pub(super) const fn read_descriptor(
    id: &'static str,
    keys: (&'static str, &'static str),
) -> CommandDescriptor {
    CommandDescriptor {
        id: CommandId::new(id),
        title_key: keys.0,
        description_key: keys.1,
        shortcut: None,
        read_only: true,
        permission: Permission::Read,
    }
}

const fn annotate_descriptor(
    id: &'static str,
    keys: (&'static str, &'static str),
    shortcut: Option<&'static str>,
) -> CommandDescriptor {
    CommandDescriptor {
        id: CommandId::new(id),
        title_key: keys.0,
        description_key: keys.1,
        shortcut,
        read_only: false,
        permission: Permission::Annotate,
    }
}

impl CreateVariant for AnyKind {
    const DESCRIPTOR: CommandDescriptor = annotate_descriptor(
        ANNOTATIONS_CREATE,
        (
            i18n::COMMAND_ANNOTATIONS_CREATE_TITLE,
            i18n::COMMAND_ANNOTATIONS_CREATE_DESCRIPTION,
        ),
        None,
    );
    const KIND: Option<AnnotationKind> = None;
}

impl CreateVariant for CriticalMark {
    const DESCRIPTOR: CommandDescriptor = annotate_descriptor(
        ANNOTATIONS_MARK_CRITICAL,
        (
            i18n::COMMAND_ANNOTATIONS_MARK_CRITICAL_TITLE,
            i18n::COMMAND_ANNOTATIONS_MARK_CRITICAL_DESCRIPTION,
        ),
        Some(SHORTCUT_CRITICAL),
    );
    const KIND: Option<AnnotationKind> = Some(AnnotationKind::Critical);
}

impl CreateVariant for NoteKind {
    const DESCRIPTOR: CommandDescriptor = annotate_descriptor(
        ANNOTATIONS_ADD_NOTE,
        (
            i18n::COMMAND_ANNOTATIONS_ADD_NOTE_TITLE,
            i18n::COMMAND_ANNOTATIONS_ADD_NOTE_DESCRIPTION,
        ),
        Some(SHORTCUT_NOTE),
    );
    const KIND: Option<AnnotationKind> = Some(AnnotationKind::Note);
}

impl CreateVariant for TaskKind {
    const DESCRIPTOR: CommandDescriptor = annotate_descriptor(
        ANNOTATIONS_ADD_TASK,
        (
            i18n::COMMAND_ANNOTATIONS_ADD_TASK_TITLE,
            i18n::COMMAND_ANNOTATIONS_ADD_TASK_DESCRIPTION,
        ),
        Some(SHORTCUT_TASK),
    );
    const KIND: Option<AnnotationKind> = Some(AnnotationKind::Task);
}

/// Adds an annotation. One implementation serves the generic command and the three that carry the
/// shortcuts, so no action is implemented twice.
pub struct Create<V: CreateVariant> {
    pub(super) ports: Arc<AnnotationPorts>,
    pub(super) variant: PhantomData<fn() -> V>,
}

impl<V: CreateVariant> Create<V> {
    pub(super) const fn new(ports: Arc<AnnotationPorts>) -> Self {
        Self {
            ports,
            variant: PhantomData,
        }
    }
}

/// The kind of a new annotation: the one the command fixes, which the caller may repeat but not
/// contradict, or else the one the caller names.
fn kind_of(
    fixed: Option<AnnotationKind>,
    said: Option<&str>,
) -> Result<AnnotationKind, CodedError> {
    let said: Option<AnnotationKind> = optional_label(said)?;
    match (fixed, said) {
        (Some(fixed), Some(said)) if fixed != said => Err(invalid()),
        (Some(kind), _) | (None, Some(kind)) => Ok(kind),
        (None, None) => Err(invalid()),
    }
}

impl<V: CreateVariant> Command for Create<V> {
    type Input = CreateInput;
    type Output = AnnotationOut;

    const DESCRIPTOR: CommandDescriptor = V::DESCRIPTOR;

    fn execute(&self, request: Request<'_, Self>) -> Result<AnnotationOut, CodedError> {
        let ports = &self.ports;
        let input = request.input;
        let kind = kind_of(V::KIND, input.kind.as_deref())?;
        let target = input.target.target()?;
        let content = ContentInput {
            kind,
            title: input.title,
            text: input.text,
            status: optional_label::<TaskStatus>(input.status.as_deref())?,
            due: input.due.map(Timestamp),
            assignee: input.assignee,
        };
        let created = ports
            .writes
            .create(ProjectId::new(input.project), target, content)?;
        Ok(AnnotationOut::from(&created))
    }
}

/// The input of `annotations.update`: a field that is missing stays as it is; `null` removes
/// `text`, `due` and `assignee`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct UpdateInput {
    /// Id of the project.
    pub project: u64,
    /// Id of the annotation.
    pub id: u64,
    /// Another symbol, to assign an annotation whose symbol is gone.
    #[serde(default)]
    pub target: Option<TargetIn>,
    /// A new title.
    #[serde(default)]
    pub title: Option<String>,
    /// The text.
    #[serde(default, deserialize_with = "nullable")]
    pub text: Nullable<String>,
    /// A new status.
    #[serde(default)]
    pub status: Option<String>,
    /// The due date, in seconds since the Unix epoch.
    #[serde(default, deserialize_with = "nullable")]
    pub due: Nullable<i64>,
    /// The assignee.
    #[serde(default, deserialize_with = "nullable")]
    pub assignee: Nullable<String>,
}

/// Changes an annotation.
pub struct AnnotationsUpdate {
    pub(super) ports: Arc<AnnotationPorts>,
}

impl Command for AnnotationsUpdate {
    type Input = UpdateInput;
    type Output = AnnotationOut;

    const DESCRIPTOR: CommandDescriptor = annotate_descriptor(
        ANNOTATIONS_UPDATE,
        (
            i18n::COMMAND_ANNOTATIONS_UPDATE_TITLE,
            i18n::COMMAND_ANNOTATIONS_UPDATE_DESCRIPTION,
        ),
        None,
    );

    fn execute(&self, request: Request<'_, Self>) -> Result<AnnotationOut, CodedError> {
        let input = request.input;
        let change = Change {
            target: input.target.as_ref().map(TargetIn::target).transpose()?,
            title: input.title,
            text: input.text.into_field(|text| text),
            status: optional_label(input.status.as_deref())?,
            due: input.due.into_field(Timestamp),
            assignee: input.assignee.into_field(|who| who),
        };
        let updated = self.ports.writes.update(
            ProjectId::new(input.project),
            AnnotationId::new(input.id),
            change,
        )?;
        Ok(AnnotationOut::from(&updated))
    }
}

/// The input of `annotations.delete`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct DeleteInput {
    /// Id of the project.
    pub project: u64,
    /// Id of the annotation.
    pub id: u64,
}

/// What `annotations.delete` answers: the id that is gone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Deleted {
    /// Id of the deleted annotation.
    pub id: u64,
}

/// Deletes an annotation.
pub struct AnnotationsDelete {
    pub(super) ports: Arc<AnnotationPorts>,
}

impl Command for AnnotationsDelete {
    type Input = DeleteInput;
    type Output = Deleted;

    const DESCRIPTOR: CommandDescriptor = annotate_descriptor(
        ANNOTATIONS_DELETE,
        (
            i18n::COMMAND_ANNOTATIONS_DELETE_TITLE,
            i18n::COMMAND_ANNOTATIONS_DELETE_DESCRIPTION,
        ),
        None,
    );

    fn execute(&self, request: Request<'_, Self>) -> Result<Deleted, CodedError> {
        let input = request.input;
        self.ports
            .writes
            .delete(ProjectId::new(input.project), AnnotationId::new(input.id))?;
        Ok(Deleted { id: input.id })
    }
}
