//! Writing annotations and dismissing alerts (ADR 0068): validates the user's text against the
//! configured limits, keeps the baseline of a critical symbol, and tells the event bus when the
//! number of open alerts changed.
//!
//! Nothing here checks a permission: the command registry does, before the command runs. Nothing
//! here is called by a scan.

use std::sync::Arc;

use codeskimr_codes::ErrorCode;
use codeskimr_domain::{
    Alert, AlertId, AlertState, Annotation, AnnotationContent, AnnotationId, AnnotationKind,
    AnnotationTarget, Assignee, Author, NewAnnotation, ProjectId, TaskStatus, Timestamp,
};

use super::read::{AlertQuery, matching_alerts};
use crate::alerts::announce_open;
use crate::alerts::observe::current_baselines;
use crate::config::Annotations;
use crate::error::CodedError;
use crate::events::EventBus;
use crate::ports::{
    AlertDismissal, AnnotationCreation, AnnotationFilter, AnnotationStore, AnnotationUpdate, Clock,
    CodeIndex, ContentDigest, SymbolStates,
};

/// What a caller says about a field that may be kept, cleared or set.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Field<T> {
    /// Leave the stored value as it is.
    #[default]
    Keep,
    /// Remove the stored value.
    Clear,
    /// Replace the stored value.
    Set(T),
}

impl<T> Field<T> {
    fn apply(self, current: Option<T>) -> Option<T> {
        match self {
            Self::Keep => current,
            Self::Clear => None,
            Self::Set(value) => Some(value),
        }
    }
}

/// What a caller says a new annotation is: the raw texts, not yet checked against the limits. A
/// blank text counts as no text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentInput {
    /// Critical mark, note or task.
    pub kind: AnnotationKind,
    /// Title of a task.
    pub title: Option<String>,
    /// The text; the reason of a critical mark.
    pub text: Option<String>,
    /// Status of a task; a task without one starts open.
    pub status: Option<TaskStatus>,
    /// Due date of a task.
    pub due: Option<Timestamp>,
    /// Assignee of a task.
    pub assignee: Option<String>,
}

/// A change to a stored annotation. Its kind never changes; a field that does not fit the kind is
/// `ANN-103`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Change {
    /// A new symbol, to re-assign an annotation whose symbol is gone.
    pub target: Option<AnnotationTarget>,
    /// A new title.
    pub title: Option<String>,
    /// The text.
    pub text: Field<String>,
    /// A new status.
    pub status: Option<TaskStatus>,
    /// The due date.
    pub due: Field<Timestamp>,
    /// The assignee.
    pub assignee: Field<String>,
}

/// The use cases that change annotations and alerts.
pub struct AnnotationWrites {
    /// Annotations, baselines and alerts.
    pub store: Arc<dyn AnnotationStore>,
    /// The newest snapshot of a project.
    pub index: Arc<dyn CodeIndex>,
    /// Looks symbols up in it, for the baseline.
    pub states: Arc<dyn SymbolStates>,
    /// The hash function of the baselines.
    pub digest: Arc<dyn ContentDigest>,
    /// Source of the time of a change.
    pub clock: Arc<dyn Clock>,
    /// Where `AlertsChanged` is published.
    pub events: EventBus,
    /// The limits of the texts.
    pub config: Annotations,
    /// Who writes: the author of every new annotation.
    pub author: Author,
}

/// A blank text is no text.
fn present(text: Option<String>) -> Option<String> {
    text.filter(|text| !text.trim().is_empty())
}

fn field_of(text: Field<String>) -> Field<String> {
    match text {
        Field::Set(text) if text.trim().is_empty() => Field::Clear,
        other => other,
    }
}

impl AnnotationWrites {
    /// Stores a new annotation of `author`. A critical mark also takes the baseline of the symbol
    /// as it is now, so the next change raises an alert.
    ///
    /// # Errors
    /// `ANN-102` for a text over the limit, `ANN-103` for fields that do not fit the kind,
    /// `ANN-101` if the symbol is not in the newest snapshot, `ANN-301` for a second critical
    /// mark, `PRJ-201` for an unknown project.
    pub fn create(
        &self,
        project: ProjectId,
        target: AnnotationTarget,
        input: ContentInput,
    ) -> Result<Annotation, CodedError> {
        self.check_target(&target)?;
        let content = self.content(input)?;
        let created = self.store.create_annotation(&AnnotationCreation {
            annotation: NewAnnotation {
                project,
                target,
                author: self.author.clone(),
                content,
            },
            now: self.clock.now(),
        })?;
        self.keep_baseline(&created)?;
        Ok(created)
    }

    /// Changes an annotation of the project.
    ///
    /// # Errors
    /// `ANN-201` if the project has no such annotation, otherwise as [`AnnotationWrites::create`].
    pub fn update(
        &self,
        project: ProjectId,
        id: AnnotationId,
        change: Change,
    ) -> Result<Annotation, CodedError> {
        let current = self.find(project, id)?;
        let next = self.merged(current, change)?;
        let updated = self.store.update_annotation(&AnnotationUpdate {
            annotation: next,
            now: self.clock.now(),
        })?;
        self.keep_baseline(&updated)?;
        Ok(updated)
    }

    /// Deletes an annotation of the project with its baseline and alerts.
    ///
    /// # Errors
    /// `ANN-201` if the project has no such annotation.
    pub fn delete(&self, project: ProjectId, id: AnnotationId) -> Result<(), CodedError> {
        let current = self.find(project, id)?;
        self.store.delete_annotation(id)?;
        if current.content.kind == AnnotationKind::Critical {
            self.announce(project)?;
        }
        Ok(())
    }

    /// Dismisses one open alert of the project: it moves to "done" and the symbol as it is now
    /// becomes the baseline.
    ///
    /// # Errors
    /// `ANN-202` if the project has no such open alert.
    pub fn dismiss(&self, project: ProjectId, alert: AlertId) -> Result<Alert, CodedError> {
        let query = AlertQuery {
            project,
            state: Some(AlertState::Open),
            kinds: Vec::new(),
        };
        let (entries, _) = matching_alerts(self.store.as_ref(), &query)?;
        let entry = entries
            .into_iter()
            .find(|entry| entry.alert.id == alert)
            .ok_or_else(|| CodedError::new(ErrorCode::AlertNotFound))?;
        let baseline = self
            .current(project, &[&entry.annotation.target])?
            .into_iter()
            .next()
            .flatten();
        let dismissed = self.store.dismiss_alert(&AlertDismissal {
            alert,
            baseline,
            now: self.clock.now(),
        })?;
        self.announce(project)?;
        Ok(dismissed)
    }

    /// Dismisses every open alert of the project that matches `query` (its state is ignored: only
    /// open alerts can be dismissed) and returns how many.
    ///
    /// # Errors
    /// A coded failure of the `STO` group.
    pub fn dismiss_all(&self, query: &AlertQuery) -> Result<u32, CodedError> {
        let open = AlertQuery {
            state: Some(AlertState::Open),
            ..query.clone()
        };
        let (entries, _) = matching_alerts(self.store.as_ref(), &open)?;
        let targets: Vec<&AnnotationTarget> = entries
            .iter()
            .map(|entry| &entry.annotation.target)
            .collect();
        let baselines = self.current(query.project, &targets)?;
        let now = self.clock.now();
        for (entry, baseline) in entries.iter().zip(baselines) {
            self.store.dismiss_alert(&AlertDismissal {
                alert: entry.alert.id,
                baseline,
                now,
            })?;
        }
        let count = u32::try_from(entries.len()).unwrap_or(u32::MAX);
        if count > 0 {
            self.announce(query.project)?;
        }
        Ok(count)
    }

    fn find(&self, project: ProjectId, id: AnnotationId) -> Result<Annotation, CodedError> {
        self.store
            .list_annotations(&AnnotationFilter {
                project,
                target: None,
                kind: None,
            })?
            .into_iter()
            .find(|annotation| annotation.id == id)
            .ok_or_else(|| CodedError::new(ErrorCode::AnnotationNotFound))
    }

    fn content(&self, input: ContentInput) -> Result<AnnotationContent, CodedError> {
        let limits = self.config.limits();
        let status = match (input.kind, input.status) {
            (AnnotationKind::Task, None) => Some(TaskStatus::Open),
            (_, status) => status,
        };
        let content = AnnotationContent {
            kind: input.kind,
            title: present(input.title)
                .map(|title| limits.title(title))
                .transpose()
                .map_err(CodedError::caused_by)?,
            text: present(input.text)
                .map(|text| limits.text(text))
                .transpose()
                .map_err(CodedError::caused_by)?,
            status,
            due: input.due,
            assignee: self.assignee(input.assignee)?,
        };
        content.check().map_err(CodedError::caused_by)?;
        Ok(content)
    }

    /// The assignee, if there is one, within `annotations.max_assignee_chars`.
    fn assignee(&self, input: Option<String>) -> Result<Option<Assignee>, CodedError> {
        let Some(who) = present(input) else {
            return Ok(None);
        };
        if who.chars().count() > self.config.max_assignee_chars as usize {
            return Err(CodedError::new(ErrorCode::AnnotationAssigneeTooLong));
        }
        Assignee::new(who).map(Some).map_err(CodedError::caused_by)
    }

    /// The symbol an annotation is put on, within the limits of its name and its parameter types.
    fn check_target(&self, target: &AnnotationTarget) -> Result<(), CodedError> {
        let limits = &self.config;
        let name_fits = target.name.as_str().chars().count() <= limits.max_name_chars as usize;
        let parameters_fit = target.parameters.len() <= limits.max_parameters as usize
            && target
                .parameters
                .iter()
                .all(|text| text.chars().count() <= limits.max_parameter_chars as usize);
        if name_fits && parameters_fit {
            Ok(())
        } else {
            Err(CodedError::new(ErrorCode::AnnotationTargetTooLong))
        }
    }

    /// The stored annotation with `change` applied, checked like a new one.
    fn merged(&self, current: Annotation, change: Change) -> Result<Annotation, CodedError> {
        let old = current.content;
        let input = ContentInput {
            kind: old.kind,
            title: change
                .title
                .or_else(|| old.title.map(|title| title.as_str().to_owned())),
            text: field_of(change.text).apply(old.text.map(|text| text.as_str().to_owned())),
            status: change.status.or(old.status),
            due: change.due.apply(old.due),
            assignee: field_of(change.assignee)
                .apply(old.assignee.map(|who| who.as_str().to_owned())),
        };
        if let Some(target) = &change.target {
            self.check_target(target)?;
        }
        Ok(Annotation {
            target: change.target.unwrap_or(current.target),
            content: self.content(input)?,
            ..current
        })
    }

    /// The baselines of `targets` in the newest snapshot, `None` for a symbol it lacks or without a
    /// snapshot.
    fn current(
        &self,
        project: ProjectId,
        targets: &[&AnnotationTarget],
    ) -> Result<Vec<Option<codeskimr_domain::SymbolBaseline>>, CodedError> {
        let Some(snapshot) = self.index.latest_snapshot(project)? else {
            return Ok(targets.iter().map(|_| None).collect());
        };
        current_baselines(
            (self.states.as_ref(), self.digest.as_ref()),
            snapshot.id,
            targets,
        )
    }

    /// A critical annotation without a baseline (new, or re-assigned to another symbol) takes the
    /// symbol as it is now.
    fn keep_baseline(&self, annotation: &Annotation) -> Result<(), CodedError> {
        if annotation.content.kind != AnnotationKind::Critical
            || self.store.baseline(annotation.id)?.is_some()
        {
            return Ok(());
        }
        let found = self.current(annotation.project, &[&annotation.target])?;
        if let Some(baseline) = found.into_iter().next().flatten() {
            self.store.store_baseline(annotation.id, &baseline)?;
        }
        Ok(())
    }

    fn announce(&self, project: ProjectId) -> Result<(), CodedError> {
        announce_open(self.store.as_ref(), &self.events, project)
    }
}
