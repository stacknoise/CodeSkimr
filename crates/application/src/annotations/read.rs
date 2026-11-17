//! Reading annotations and alerts as lists (ADR 0068): filtered, paged and cheap. Every list is one
//! query of the store, filtered in memory: the rows are what a user wrote by hand, so there are few.

use std::collections::{BTreeMap, HashMap};

use codeskimr_domain::{
    Alert, AlertKind, AlertState, Annotation, AnnotationId, AnnotationKind, AnnotationTarget,
    ProjectId, TaskStatus,
};

use super::page::{Page, PageRequest};
use crate::error::CodedError;
use crate::ports::{AlertFilter, AnnotationFilter, AnnotationStore};

/// Which annotations to list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnnotationQuery {
    /// The project.
    pub project: ProjectId,
    /// Only those of this symbol. Without parameter types it means every overload of the name.
    pub target: Option<AnnotationTarget>,
    /// Only this kind.
    pub kind: Option<AnnotationKind>,
    /// Only tasks in this status.
    pub status: Option<TaskStatus>,
    /// Only those whose symbol is gone (`true`) or still there (`false`).
    pub without_target: Option<bool>,
}

impl AnnotationQuery {
    /// Every annotation of the project.
    #[must_use]
    pub const fn of_project(project: ProjectId) -> Self {
        Self {
            project,
            target: None,
            kind: None,
            status: None,
            without_target: None,
        }
    }

    /// The target to hand to the store: only one that names parameter types, because the store
    /// matches those exactly. A target without them means every overload of the name, which
    /// `matches` selects.
    fn exact_target(&self) -> Option<AnnotationTarget> {
        self.target
            .clone()
            .filter(|target| !target.parameters.is_empty())
    }

    fn matches(&self, annotation: &Annotation) -> bool {
        self.target.as_ref().is_none_or(|target| {
            annotation.target.kind == target.kind && annotation.target.name == target.name
        }) && self
            .status
            .is_none_or(|status| annotation.content.status == Some(status))
            && self
                .without_target
                .is_none_or(|wanted| annotation.without_target == wanted)
    }
}

/// The annotations that match `query`, oldest first, one page of them.
///
/// # Errors
/// A coded failure of the `STO` group.
pub fn annotations(
    store: &dyn AnnotationStore,
    query: &AnnotationQuery,
    request: PageRequest,
) -> Result<Page<Annotation>, CodedError> {
    let all = store.list_annotations(&AnnotationFilter {
        project: query.project,
        target: query.exact_target(),
        kind: query.kind,
    })?;
    Ok(Page::of(
        all.into_iter()
            .filter(|annotation| query.matches(annotation))
            .collect(),
        request,
    ))
}

/// Which alerts to list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlertQuery {
    /// The project.
    pub project: ProjectId,
    /// Only alerts in this state.
    pub state: Option<AlertState>,
    /// Only alerts of these kinds; empty means every kind.
    pub kinds: Vec<AlertKind>,
}

impl AlertQuery {
    fn matches(&self, alert: &Alert) -> bool {
        self.state.is_none_or(|state| alert.state == state)
            && (self.kinds.is_empty() || self.kinds.contains(&alert.sighting.kind))
    }
}

/// An alert with the critical annotation it is about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlertEntry {
    /// The alert.
    pub alert: Alert,
    /// The critical annotation: it names the symbol and holds the reason.
    pub annotation: Annotation,
}

/// A page of alerts and how many are open in the whole project, whatever the filter says (the
/// number on the bell).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlertPage {
    /// The page.
    pub page: Page<AlertEntry>,
    /// Open alerts of the project.
    pub open: u32,
}

/// Every alert of the project that matches `query`, with its annotation, and the number of open
/// alerts of the project. Two reads: the alerts and the critical annotations.
///
/// # Errors
/// A coded failure of the `STO` group.
pub fn matching_alerts(
    store: &dyn AnnotationStore,
    query: &AlertQuery,
) -> Result<(Vec<AlertEntry>, u32), CodedError> {
    let all = store.list_alerts(&AlertFilter {
        project: query.project,
        state: None,
    })?;
    let open = all
        .iter()
        .filter(|alert| alert.state == AlertState::Open)
        .count();
    let critical = store.list_annotations(&AnnotationFilter {
        project: query.project,
        target: None,
        kind: Some(AnnotationKind::Critical),
    })?;
    let by_id: HashMap<AnnotationId, Annotation> = critical
        .into_iter()
        .map(|annotation| (annotation.id, annotation))
        .collect();
    let entries = all
        .into_iter()
        .filter(|alert| query.matches(alert))
        .filter_map(|alert| {
            let annotation = by_id.get(&alert.annotation)?.clone();
            Some(AlertEntry { alert, annotation })
        })
        .collect();
    Ok((entries, u32::try_from(open).unwrap_or(u32::MAX)))
}

/// The alerts that match `query`, newest sighting first, one page of them.
///
/// # Errors
/// A coded failure of the `STO` group.
pub fn alerts(
    store: &dyn AnnotationStore,
    query: &AlertQuery,
    request: PageRequest,
) -> Result<AlertPage, CodedError> {
    let (entries, open) = matching_alerts(store, query)?;
    Ok(AlertPage {
        page: Page::of(entries, request),
        open,
    })
}

/// What a symbol carries, for the marks and counters of the tree and the member table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetSummary {
    /// The symbol, by kind, qualified name and parameter types.
    pub target: AnnotationTarget,
    /// Whether it is marked as system critical.
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

/// The key a symbol is grouped by: qualified name first, so the page order is the order of names.
type GroupKey = (String, &'static str, Vec<String>);

fn group_key(target: &AnnotationTarget) -> GroupKey {
    (
        target.name.as_str().to_owned(),
        target.kind.label(),
        target.parameters.clone(),
    )
}

fn count_into(summary: &mut TargetSummary, annotation: &Annotation) {
    summary.without_target |= annotation.without_target;
    match annotation.content.kind {
        AnnotationKind::Critical => summary.critical = true,
        AnnotationKind::Note => summary.notes += 1,
        AnnotationKind::Task => {
            summary.tasks_total += 1;
            if annotation.content.status == Some(TaskStatus::Done) {
                summary.tasks_done += 1;
            }
        }
    }
}

/// One row per annotated symbol of the project, ordered by qualified name: one read, whatever the
/// number of symbols, so the UI marks a whole tree without a request per row.
///
/// # Errors
/// A coded failure of the `STO` group.
pub fn summaries(
    store: &dyn AnnotationStore,
    project: ProjectId,
    request: PageRequest,
) -> Result<Page<TargetSummary>, CodedError> {
    let all = store.list_annotations(&AnnotationFilter {
        project,
        target: None,
        kind: None,
    })?;
    let mut groups: BTreeMap<GroupKey, TargetSummary> = BTreeMap::new();
    for annotation in &all {
        let summary = groups
            .entry(group_key(&annotation.target))
            .or_insert_with(|| TargetSummary {
                target: annotation.target.clone(),
                critical: false,
                notes: 0,
                tasks_done: 0,
                tasks_total: 0,
                without_target: false,
            });
        count_into(summary, annotation);
    }
    Ok(Page::of(groups.into_values().collect(), request))
}

/// The parts of an annotation as plain values, for the answers of the REST routes and the MCP
/// tools.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentTexts {
    /// Title of a task.
    pub title: Option<String>,
    /// The text.
    pub text: Option<String>,
    /// Label of the status of a task.
    pub status: Option<&'static str>,
    /// Due date in seconds since the Unix epoch.
    pub due: Option<i64>,
    /// Assignee of a task.
    pub assignee: Option<String>,
}

impl From<&codeskimr_domain::AnnotationContent> for ContentTexts {
    fn from(content: &codeskimr_domain::AnnotationContent) -> Self {
        Self {
            title: content.title.as_ref().map(ToString::to_string),
            text: content.text.as_ref().map(ToString::to_string),
            status: content.status.map(TaskStatus::label),
            due: content.due.map(|due| due.0),
            assignee: content.assignee.as_ref().map(ToString::to_string),
        }
    }
}
