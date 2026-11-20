//! Annotations on symbols, their baselines and the alert list (ADR 0068; adapter: `codeskimr_storage`).
//!
//! They belong to the project, not to a snapshot, so scans and snapshot retention never touch them.

use codeskimr_domain::{
    Alert, AlertId, AlertSighting, AlertState, Annotation, AnnotationId, AnnotationKind,
    AnnotationTarget, NewAnnotation, ProjectId, SnapshotId, SymbolBaseline, Timestamp,
};

use crate::error::CodedError;

/// Which annotations to list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnnotationFilter {
    /// The project.
    pub project: ProjectId,
    /// Only those of this symbol; `None` lists all of the project.
    pub target: Option<AnnotationTarget>,
    /// Only this kind; `None` lists every kind.
    pub kind: Option<AnnotationKind>,
}

/// A change to a stored annotation: `annotation` holds the new values of its target (to re-assign
/// an orphaned one) and of its content. Its id says which one; kind, project, author and creation
/// time never change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnnotationUpdate {
    /// The annotation with its new values.
    pub annotation: Annotation,
    /// The time of the change.
    pub now: Timestamp,
}

/// A new annotation and the time it is written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnnotationCreation {
    /// What to store.
    pub annotation: NewAnnotation,
    /// The creation time.
    pub now: Timestamp,
}

/// A critical symbol was seen changed, removed or orphaned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlertRaise {
    /// The critical annotation.
    pub annotation: AnnotationId,
    /// What the scan found.
    pub sighting: AlertSighting,
    /// The time of the scan.
    pub now: Timestamp,
    /// Whether an open alert counts this sighting. False when the sighting only brings the alert
    /// up to date with what the symbol looks like now, as when a removed symbol is back in the
    /// state the alert saw before; a new alert starts at one either way.
    pub counted: bool,
}

/// Which alerts to list, newest sighting first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AlertFilter {
    /// The project.
    pub project: ProjectId,
    /// Only alerts in this state; `None` lists both.
    pub state: Option<AlertState>,
}

/// An alert the user removes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlertDismissal {
    /// The alert.
    pub alert: AlertId,
    /// What the symbol is now; it becomes the baseline, so only a later change raises a new alert.
    /// `None` for a symbol that is gone: the baseline stays as it was.
    pub baseline: Option<SymbolBaseline>,
    /// The time of the dismissal.
    pub now: Timestamp,
}

/// Dismissed alerts older than a number of days.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AlertPurge {
    /// The time now.
    pub now: Timestamp,
    /// Dismissed alerts older than this many days are deleted (`annotations.dismissed_retention_days`).
    pub retention_days: u32,
}

/// Stores annotations, baselines and alerts.
pub trait AnnotationStore: Send + Sync {
    /// Stores a new annotation.
    ///
    /// # Errors
    /// `ANN-103` if the fields do not fit the kind, `ANN-101` if the symbol is not in the newest
    /// snapshot, `ANN-301` if it is critical already and the new one is critical too, `PRJ-201`
    /// if the project does not exist.
    fn create_annotation(&self, creation: &AnnotationCreation) -> Result<Annotation, CodedError>;

    /// Replaces target and content of an annotation.
    ///
    /// # Errors
    /// `ANN-201` if it does not exist, otherwise as [`AnnotationStore::create_annotation`].
    fn update_annotation(&self, update: &AnnotationUpdate) -> Result<Annotation, CodedError>;

    /// Deletes an annotation with its baseline and alerts.
    ///
    /// # Errors
    /// `ANN-201` if it does not exist.
    fn delete_annotation(&self, id: AnnotationId) -> Result<(), CodedError>;

    /// The annotations that match the filter, oldest first.
    ///
    /// # Errors
    /// A coded failure of the `STO` group.
    fn list_annotations(&self, filter: &AnnotationFilter) -> Result<Vec<Annotation>, CodedError>;

    /// Sets `without_target` of every annotation of the project: true if no symbol of `snapshot`
    /// has the kind and qualified name of its target, false if one has. Returns how many flags
    /// changed. A project without annotations costs one read.
    ///
    /// # Errors
    /// A coded failure of the `STO` group.
    fn refresh_targets(&self, project: ProjectId, snapshot: SnapshotId) -> Result<u64, CodedError>;

    /// The baseline of a critical annotation, `None` if none was taken yet.
    ///
    /// # Errors
    /// `ANN-201` if the annotation does not exist.
    fn baseline(&self, annotation: AnnotationId) -> Result<Option<SymbolBaseline>, CodedError>;

    /// Stores the baseline of a critical annotation, replacing the old one.
    ///
    /// # Errors
    /// `ANN-201` if the annotation does not exist, `ANN-103` if it is not critical.
    fn store_baseline(
        &self,
        annotation: AnnotationId,
        baseline: &SymbolBaseline,
    ) -> Result<(), CodedError>;

    /// Raises an alert for a critical annotation. If it has an open alert, that one counts one
    /// more and takes the sighting and the time as its latest; otherwise a new alert with the
    /// counter one is stored. So there is at most one open alert per annotation.
    ///
    /// # Errors
    /// `ANN-201` if the annotation does not exist, `ANN-103` if it is not critical.
    fn raise_alert(&self, raise: &AlertRaise) -> Result<Alert, CodedError>;

    /// The alerts of a project that match the filter.
    ///
    /// # Errors
    /// A coded failure of the `STO` group.
    fn list_alerts(&self, filter: &AlertFilter) -> Result<Vec<Alert>, CodedError>;

    /// Sets an alert to dismissed and the baseline of its annotation to the current state.
    ///
    /// # Errors
    /// `ANN-202` if the alert does not exist.
    fn dismiss_alert(&self, dismissal: &AlertDismissal) -> Result<Alert, CodedError>;

    /// Deletes the dismissed alerts that are older than the retention and returns how many.
    ///
    /// # Errors
    /// A coded failure of the `STO` group.
    fn purge_dismissed(&self, purge: &AlertPurge) -> Result<u64, CodedError>;
}
