//! Notes, tasks and critical marks that the user puts on symbols (ADR 0068). They are not part of
//! a snapshot: a symbol is found again by its target key, never by a line or a snapshot id.

use crate::consts::{
    ANNOTATION_CRITICAL, ANNOTATION_NOTE, ANNOTATION_TASK, TASK_DONE, TASK_IN_PROGRESS, TASK_OPEN,
};
use crate::error::ModelError;
use crate::project::ProjectId;
use crate::qualified_name::QualifiedName;
use crate::snapshot::Timestamp;
use crate::symbol::SymbolKind;
use crate::vocabulary::{bounded_text_type, id_type, labeled_enum, text_type};

id_type!(
    /// Identifies an annotation.
    AnnotationId
);
text_type!(
    /// Who wrote an annotation: a user name, free text until the team edition has accounts.
    Author
);
text_type!(
    /// Who a task is assigned to, free text.
    Assignee
);
bounded_text_type!(
    /// Text of a note or a task, or the reason of a critical mark (limit: `annotations.max_text_chars`).
    AnnotationText
);
bounded_text_type!(
    /// Title of a task (limit: `annotations.max_title_chars`).
    AnnotationTitle
);

labeled_enum!(
    /// What an annotation says about its symbol.
    AnnotationKind {
        /// The symbol is system critical; a change raises an alert.
        Critical => ANNOTATION_CRITICAL,
        /// A note.
        Note => ANNOTATION_NOTE,
        /// A task with a status.
        Task => ANNOTATION_TASK,
    }
);

labeled_enum!(
    /// How far a task is.
    TaskStatus {
        /// Not started.
        Open => TASK_OPEN,
        /// Somebody works on it.
        InProgress => TASK_IN_PROGRESS,
        /// Finished.
        Done => TASK_DONE,
    }
);

/// The limits for texts of user input. They come from the configuration (`annotations.*`) and are
/// passed in, so the model holds no number of its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnnotationLimits {
    /// Most characters of an [`AnnotationText`].
    pub max_text_chars: u32,
    /// Most characters of an [`AnnotationTitle`].
    pub max_title_chars: u32,
}

impl AnnotationLimits {
    /// Validates a text.
    ///
    /// # Errors
    /// [`ModelError::EmptyText`] or [`ModelError::TooLong`] (`ANN-102`).
    pub fn text(&self, value: impl Into<String>) -> Result<AnnotationText, ModelError> {
        AnnotationText::new(value, self.max_text_chars)
    }

    /// Validates a title.
    ///
    /// # Errors
    /// [`ModelError::EmptyText`] or [`ModelError::TooLong`] (`ANN-102`).
    pub fn title(&self, value: impl Into<String>) -> Result<AnnotationTitle, ModelError> {
        AnnotationTitle::new(value, self.max_title_chars)
    }
}

/// The stable key of a symbol: its kind, its qualified name and the types of its parameters (empty
/// for what has none). It names no line and no snapshot, so it survives every scan.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AnnotationTarget {
    /// Kind of the symbol; a namespace, a type or a member.
    pub kind: SymbolKind,
    /// Fully qualified name.
    pub name: QualifiedName,
    /// Types of the parameters as written in the signature, in order; tells overloads apart.
    pub parameters: Vec<String>,
}

/// What an annotation says, without its identity. Which fields may be set depends on the kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnnotationContent {
    /// Critical mark, note or task.
    pub kind: AnnotationKind,
    /// Title of a task.
    pub title: Option<AnnotationTitle>,
    /// The text; the reason of a critical mark (optional there), required for a note.
    pub text: Option<AnnotationText>,
    /// Status of a task.
    pub status: Option<TaskStatus>,
    /// Due date of a task.
    pub due: Option<Timestamp>,
    /// Assignee of a task.
    pub assignee: Option<Assignee>,
}

impl AnnotationContent {
    /// Checks that the fields fit the kind: a critical mark has at most a reason, a note has a
    /// text and nothing else, a task has a title and a status.
    ///
    /// # Errors
    /// [`ModelError::Inconsistent`] (`ANN-103`) if a field is missing or does not belong to the kind.
    pub fn check(&self) -> Result<(), ModelError> {
        let task_fields = self.title.is_some()
            || self.status.is_some()
            || self.due.is_some()
            || self.assignee.is_some();
        let fits = match self.kind {
            AnnotationKind::Critical => !task_fields,
            AnnotationKind::Note => !task_fields && self.text.is_some(),
            AnnotationKind::Task => self.title.is_some() && self.status.is_some(),
        };
        if fits {
            Ok(())
        } else {
            Err(ModelError::Inconsistent)
        }
    }
}

/// An annotation that is not stored yet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewAnnotation {
    /// Owning project.
    pub project: ProjectId,
    /// The symbol it is about.
    pub target: AnnotationTarget,
    /// Who writes it.
    pub author: Author,
    /// What it says.
    pub content: AnnotationContent,
}

/// A stored annotation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Annotation {
    /// Identity.
    pub id: AnnotationId,
    /// Owning project.
    pub project: ProjectId,
    /// The symbol it is about.
    pub target: AnnotationTarget,
    /// Who wrote it.
    pub author: Author,
    /// What it says.
    pub content: AnnotationContent,
    /// Creation time.
    pub created: Timestamp,
    /// Time of the last change.
    pub updated: Timestamp,
    /// Whether the newest scan did not find the symbol. It is kept for every kind; a note or a
    /// task only shows it (the side panel and the task board), a critical annotation raises an
    /// alert as well (ADR 0068, decision 3).
    pub without_target: bool,
}
