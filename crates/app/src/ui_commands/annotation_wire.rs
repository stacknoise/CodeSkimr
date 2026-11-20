//! The JSON forms of annotations and alerts that the commands take and return (ADR 0068). Kinds,
//! states and statuses travel as their labels, times as seconds since the Unix epoch.

use std::str::FromStr;

use codeskimr_application::annotations::{AlertEntry, ContentTexts, Field, Page, TargetSummary};
use codeskimr_application::error::CodedError;
use codeskimr_codes::ErrorCode;
use codeskimr_domain::{Alert, Annotation, AnnotationTarget, QualifiedName, SymbolKind};
use serde::{Deserialize, Deserializer, Serialize};

/// The error of an input that does not fit: `API-101`.
pub(super) fn invalid() -> CodedError {
    CodedError::new(ErrorCode::RequestInvalid)
}

/// The value a label names.
pub(super) fn label<T: FromStr>(text: &str) -> Result<T, CodedError> {
    text.parse().map_err(|_| invalid())
}

/// The value an optional label names.
pub(super) fn optional_label<T: FromStr>(text: Option<&str>) -> Result<Option<T>, CodedError> {
    text.map(label).transpose()
}

/// A field that may be left out (keep), `null` (clear) or given (set).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Nullable<T> {
    /// The key is missing: the stored value stays.
    #[default]
    Keep,
    /// The value is `null`: the stored value is removed.
    Clear,
    /// A new value.
    Set(T),
}

impl<T> Nullable<T> {
    /// The change of the application layer this asks for, with the value converted.
    pub(super) fn into_field<U>(self, convert: impl FnOnce(T) -> U) -> Field<U> {
        match self {
            Self::Keep => Field::Keep,
            Self::Clear => Field::Clear,
            Self::Set(value) => Field::Set(convert(value)),
        }
    }
}

/// Reads a key that is present as `Clear` for `null` and `Set` for a value; a key that is missing
/// takes the default, `Keep`.
pub(super) fn nullable<'de, D, T>(deserializer: D) -> Result<Nullable<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Ok(Option::<T>::deserialize(deserializer)?.map_or(Nullable::Clear, Nullable::Set))
}

/// A symbol as a caller names it: its kind, its qualified name and, to tell overloads apart, the
/// types of its parameters as written in the signature.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct TargetIn {
    /// Kind label of the symbol, such as `class`, `method` or `namespace`.
    pub kind: String,
    /// Fully qualified name.
    pub name: String,
    /// Parameter types in order; empty for what has none.
    #[serde(default)]
    pub parameters: Vec<String>,
}

impl TargetIn {
    /// The target this names.
    ///
    /// # Errors
    /// `API-101` for an unknown kind or an empty or invalid name.
    pub(super) fn target(&self) -> Result<AnnotationTarget, CodedError> {
        Ok(AnnotationTarget {
            kind: label::<SymbolKind>(&self.kind)?,
            name: QualifiedName::new(self.name.as_str()).map_err(|_| invalid())?,
            parameters: self.parameters.clone(),
        })
    }
}

/// A symbol in an answer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TargetOut {
    /// Kind label.
    pub kind: &'static str,
    /// Qualified name.
    pub name: String,
    /// Parameter types, empty for what has none.
    pub parameters: Vec<String>,
}

impl From<&AnnotationTarget> for TargetOut {
    fn from(target: &AnnotationTarget) -> Self {
        Self {
            kind: target.kind.label(),
            name: target.name.to_string(),
            parameters: target.parameters.clone(),
        }
    }
}

/// One annotation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AnnotationOut {
    /// Id of the annotation.
    pub id: u64,
    /// Id of the project.
    pub project: u64,
    /// The symbol it is about.
    pub target: TargetOut,
    /// `critical`, `note` or `task`.
    pub kind: &'static str,
    /// Title of a task.
    pub title: Option<String>,
    /// The text; the reason of a critical mark.
    pub text: Option<String>,
    /// `open`, `in_progress` or `done`, for a task.
    pub status: Option<&'static str>,
    /// Due date of a task, in seconds since the Unix epoch.
    pub due: Option<i64>,
    /// Assignee of a task.
    pub assignee: Option<String>,
    /// Who wrote it.
    pub author: String,
    /// Creation time, in seconds since the Unix epoch.
    pub created: i64,
    /// Time of the last change, in seconds since the Unix epoch.
    pub updated: i64,
    /// Whether the newest scan does not find the symbol.
    pub without_target: bool,
}

impl From<&Annotation> for AnnotationOut {
    fn from(annotation: &Annotation) -> Self {
        let texts = ContentTexts::from(&annotation.content);
        Self {
            id: annotation.id.get(),
            project: annotation.project.get(),
            target: TargetOut::from(&annotation.target),
            kind: annotation.content.kind.label(),
            title: texts.title,
            text: texts.text,
            status: texts.status,
            due: texts.due,
            assignee: texts.assignee,
            author: annotation.author.to_string(),
            created: annotation.created.0,
            updated: annotation.updated.0,
            without_target: annotation.without_target,
        }
    }
}

/// One alert with what it is about.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AlertOut {
    /// Id of the alert.
    pub id: u64,
    /// Id of the critical annotation.
    pub annotation: u64,
    /// The symbol.
    pub target: TargetOut,
    /// The reason of the critical mark, if it has one.
    pub reason: Option<String>,
    /// `changed`, `removed` or `orphaned`.
    pub kind: &'static str,
    /// `open` or `dismissed`.
    pub state: &'static str,
    /// How often the symbol was seen changed while the alert was open.
    pub counter: u32,
    /// When it was raised first, in seconds since the Unix epoch.
    pub first_seen: i64,
    /// When it was seen last.
    pub last_seen: i64,
    /// When it was dismissed.
    pub dismissed: Option<i64>,
    /// The snapshot of the latest sighting.
    pub snapshot: u64,
    /// The commit of the latest sighting, for a Git project.
    pub commit: Option<String>,
    /// `scan`, `watcher` or `fetch`.
    pub trigger: &'static str,
    /// For `changed`: `content`, `signature`, `cc` or `loc`.
    pub changes: Vec<&'static str>,
    /// For `removed`: how many references remain (provisional).
    pub references: Option<u32>,
    /// For `orphaned`: the closest symbol of the same name.
    pub closest: Option<String>,
}

impl From<&AlertEntry> for AlertOut {
    fn from(entry: &AlertEntry) -> Self {
        let alert: &Alert = &entry.alert;
        let sighting = &alert.sighting;
        Self {
            id: alert.id.get(),
            annotation: alert.annotation.get(),
            target: TargetOut::from(&entry.annotation.target),
            reason: entry
                .annotation
                .content
                .text
                .as_ref()
                .map(ToString::to_string),
            kind: sighting.kind.label(),
            state: alert.state.label(),
            counter: alert.counter,
            first_seen: alert.first_seen.0,
            last_seen: alert.last_seen.0,
            dismissed: alert.dismissed.map(|time| time.0),
            snapshot: sighting.snapshot.get(),
            commit: sighting.commit.as_ref().map(ToString::to_string),
            trigger: sighting.trigger.label(),
            changes: sighting
                .changes
                .iter()
                .map(|change| change.label())
                .collect(),
            references: sighting.references,
            closest: sighting.closest.as_ref().map(ToString::to_string),
        }
    }
}

/// What one symbol carries: the mark and the counters for the tree and the member table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SummaryOut {
    /// The symbol.
    pub target: TargetOut,
    /// Whether it is marked as critical.
    pub critical: bool,
    /// Number of notes.
    pub notes: u32,
    /// Number of tasks that are done.
    pub tasks_done: u32,
    /// Number of tasks.
    pub tasks_total: u32,
    /// Whether the newest scan does not find the symbol.
    pub without_target: bool,
}

impl From<&TargetSummary> for SummaryOut {
    fn from(summary: &TargetSummary) -> Self {
        Self {
            target: TargetOut::from(&summary.target),
            critical: summary.critical,
            notes: summary.notes,
            tasks_done: summary.tasks_done,
            tasks_total: summary.tasks_total,
            without_target: summary.without_target,
        }
    }
}

/// One page of a list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PageOut<T> {
    /// The rows.
    pub items: Vec<T>,
    /// Rows of the whole list.
    pub total: u32,
    /// Rows skipped.
    pub offset: u32,
    /// The limit that was applied.
    pub limit: u32,
    /// The offset of the next page, `null` on the last one.
    pub next_offset: Option<u32>,
}

impl<T> PageOut<T> {
    /// The page with each row turned into its wire form.
    pub(super) fn of<U>(page: &Page<U>, wire: impl FnMut(&U) -> T) -> Self {
        Self {
            items: page.items.iter().map(wire).collect(),
            total: page.total,
            offset: page.offset,
            limit: page.limit,
            next_offset: page.next_offset(),
        }
    }
}
