//! The alert step (ADR 0068, decisions 4 and 5): after a snapshot is stored, the critical symbols
//! are compared with their baselines and an alert is written for each one that changed, is gone
//! or lost its place.
//!
//! The step is a read-model step. It runs after the snapshot transaction; a failure is the caller's
//! to log, and it never undoes the snapshot.

pub mod compare;
pub mod target;

use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;

use codeskimr_domain::{
    Alert, AlertChange, AlertKind, AlertSighting, AlertState, AlertTrigger, Annotation,
    AnnotationId, AnnotationKind, AnnotationTarget, ProjectId, QualifiedName, Snapshot,
    SymbolBaseline, SymbolKind, Timestamp,
};

use crate::error::CodedError;
use crate::events::{AlertsChanged, Event, EventBus};
use crate::ports::{
    AlertFilter, AlertRaise, AnnotationFilter, AnnotationStore, Clock, ContentDigest, SymbolState,
    SymbolStates,
};

/// What one run of the step did.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AlertReport {
    /// Critical annotations that were checked.
    pub checked: u32,
    /// Baselines that were taken because the annotation had none yet.
    pub baselines: u32,
    /// Alerts that were raised or counted once more.
    pub raised: u32,
}

/// What the step found out about one critical annotation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Outcome {
    /// Nothing to tell.
    Quiet,
    /// The annotation had no baseline; it has one now.
    Baselined,
    /// An alert was raised or counted once more.
    Raised,
}

/// The state of the world while the annotations of one snapshot are checked.
struct Look<'a> {
    snapshot: &'a Snapshot,
    trigger: AlertTrigger,
    now: Timestamp,
    /// The symbols that carry a name of a critical target.
    states: Vec<SymbolState>,
    /// The open alerts, by annotation.
    open: HashMap<AnnotationId, Alert>,
}

/// What an alert says before it is dressed as a sighting.
struct Finding {
    kind: AlertKind,
    changes: Vec<AlertChange>,
    references: Option<u32>,
    closest: Option<QualifiedName>,
    observed: Option<SymbolBaseline>,
}

/// Compares the critical symbols of a project with their baselines after each snapshot.
pub struct AlertStep {
    /// Annotations, baselines and alerts.
    pub annotations: Arc<dyn AnnotationStore>,
    /// Looks the targets up in the new snapshot.
    pub states: Arc<dyn SymbolStates>,
    /// The hash function of the content hashes.
    pub digest: Arc<dyn ContentDigest>,
    /// Source of the time of a sighting.
    pub clock: Arc<dyn Clock>,
    /// Where `AlertsChanged` is published.
    pub events: EventBus,
}

impl AlertStep {
    /// Checks the project of `snapshot`, which is the newest one and already stored: marks the
    /// annotations whose target is gone, then writes an alert for every critical symbol that
    /// changed, was removed or lost its place, and a baseline for every critical annotation that
    /// has none. Publishes `AlertsChanged` if an alert was written.
    ///
    /// A project without annotations costs two cheap statements. An annotation that fails does not
    /// keep the others from being checked; the first failure is returned at the end.
    ///
    /// # Errors
    /// A coded failure of the `STO` group or `ANN-103` from the store.
    pub fn run(
        &self,
        snapshot: &Snapshot,
        trigger: AlertTrigger,
    ) -> Result<AlertReport, CodedError> {
        self.annotations
            .refresh_targets(snapshot.project, snapshot.id)?;
        let critical = self.annotations.list_annotations(&AnnotationFilter {
            project: snapshot.project,
            target: None,
            kind: Some(AnnotationKind::Critical),
        })?;
        if critical.is_empty() {
            return Ok(AlertReport::default());
        }
        let look = self.look(snapshot, trigger, &critical)?;
        let mut report = AlertReport::default();
        let mut failure = None;
        for annotation in &critical {
            match self.check(&look, annotation) {
                Ok(outcome) => count(&mut report, outcome),
                Err(error) => {
                    failure.get_or_insert(error);
                }
            }
        }
        report.checked = u32::try_from(critical.len()).unwrap_or(u32::MAX);
        if report.raised > 0 {
            self.announce(snapshot.project)?;
        }
        failure.map_or(Ok(report), Err)
    }

    fn look<'a>(
        &self,
        snapshot: &'a Snapshot,
        trigger: AlertTrigger,
        critical: &[Annotation],
    ) -> Result<Look<'a>, CodedError> {
        let names: BTreeSet<&QualifiedName> = critical
            .iter()
            .map(|annotation| &annotation.target.name)
            .collect();
        let names: Vec<QualifiedName> = names.into_iter().cloned().collect();
        let open = self.annotations.list_alerts(&AlertFilter {
            project: snapshot.project,
            state: Some(AlertState::Open),
        })?;
        Ok(Look {
            snapshot,
            trigger,
            now: self.clock.now(),
            states: self.states.states(snapshot.id, &names)?,
            open: open
                .into_iter()
                .map(|alert| (alert.annotation, alert))
                .collect(),
        })
    }

    fn check(&self, look: &Look<'_>, annotation: &Annotation) -> Result<Outcome, CodedError> {
        match target::pick(&look.states, &annotation.target) {
            Some(state) => self.compare(look, annotation, state),
            None => self.vanished(look, annotation),
        }
    }

    /// The symbol is there: take its baseline if it has none, otherwise raise an alert if it
    /// differs from the baseline and from what the open alert saw last.
    fn compare(
        &self,
        look: &Look<'_>,
        annotation: &Annotation,
        state: &SymbolState,
    ) -> Result<Outcome, CodedError> {
        let now = self.observe(look.snapshot, state)?;
        let Some(baseline) = self.annotations.baseline(annotation.id)? else {
            self.annotations.store_baseline(annotation.id, &now)?;
            return Ok(Outcome::Baselined);
        };
        let changes = compare::differences(&baseline, &now);
        let seen = look.open.get(&annotation.id).is_some_and(|alert| {
            alert
                .sighting
                .observed
                .as_ref()
                .is_some_and(|before| compare::differences(before, &now).is_empty())
        });
        if changes.is_empty() || seen {
            return Ok(Outcome::Quiet);
        }
        self.raise(
            look,
            annotation,
            Finding {
                kind: AlertKind::Changed,
                changes,
                references: None,
                closest: None,
                observed: Some(now),
            },
        )
    }

    /// The symbol is not in the snapshot: `orphaned` if a symbol of the same simple name exists
    /// (it may have been renamed or moved, and the user can re-assign the annotation), otherwise
    /// `removed` with the names that still point at it. An open alert of either kind already
    /// says so.
    fn vanished(&self, look: &Look<'_>, annotation: &Annotation) -> Result<Outcome, CodedError> {
        let said = look.open.get(&annotation.id).is_some_and(|alert| {
            matches!(
                alert.sighting.kind,
                AlertKind::Removed | AlertKind::Orphaned
            )
        });
        if said {
            return Ok(Outcome::Quiet);
        }
        let finding = self.loss(look, &annotation.target)?;
        self.raise(look, annotation, finding)
    }

    /// What the scan says about a target that is not in the snapshot.
    fn loss(&self, look: &Look<'_>, target: &AnnotationTarget) -> Result<Finding, CodedError> {
        let simple = target::simple_name(&target.name);
        let same = match &simple {
            Some(simple) => self.states.same_simple_name(look.snapshot.id, simple)?,
            None => Vec::new(),
        };
        let closest = target::closest(target, &same);
        let (kind, references) = match (&closest, &simple) {
            (Some(_), _) => (AlertKind::Orphaned, None),
            (None, Some(simple)) => {
                let names =
                    self.states
                        .remaining_references(look.snapshot.id, &target.name, simple)?;
                (AlertKind::Removed, Some(names))
            }
            (None, None) => (AlertKind::Removed, Some(0)),
        };
        Ok(Finding {
            kind,
            changes: Vec::new(),
            references,
            closest,
            observed: None,
        })
    }

    fn raise(
        &self,
        look: &Look<'_>,
        annotation: &Annotation,
        finding: Finding,
    ) -> Result<Outcome, CodedError> {
        let sighting = AlertSighting {
            kind: finding.kind,
            snapshot: look.snapshot.id,
            commit: look.snapshot.commit.clone(),
            trigger: look.trigger,
            changes: finding.changes,
            references: finding.references,
            closest: finding.closest,
            observed: finding.observed,
        };
        self.annotations.raise_alert(&AlertRaise {
            annotation: annotation.id,
            sighting,
            now: look.now,
        })?;
        Ok(Outcome::Raised)
    }

    /// What the symbol looks like now. A namespace has no hash of its own: its content is the hash
    /// over the sorted hashes of the types and members below it, so it changes when one of them
    /// does, is added or is removed. A symbol without a hash (an old row, a kind that has none)
    /// hashes as empty, so only its signature, complexity and lines can tell it apart.
    fn observe(
        &self,
        snapshot: &Snapshot,
        state: &SymbolState,
    ) -> Result<SymbolBaseline, CodedError> {
        let content_hash = if state.kind == SymbolKind::Namespace {
            let mut hashes = self.states.child_hashes(snapshot.id, &state.name)?;
            hashes.sort();
            let parts: Vec<&str> = hashes
                .iter()
                .map(codeskimr_domain::ContentHash::as_str)
                .collect();
            self.digest.digest(&parts)?
        } else {
            match &state.content_hash {
                Some(hash) => hash.clone(),
                None => self.digest.digest(&[])?,
            }
        };
        let words: Vec<&str> = state
            .signature
            .as_deref()
            .unwrap_or_default()
            .split_whitespace()
            .collect();
        Ok(SymbolBaseline {
            content_hash,
            signature_hash: self.digest.digest(&words)?,
            loc: state.loc,
            cc: state.cc,
        })
    }

    fn announce(&self, project: ProjectId) -> Result<(), CodedError> {
        let open = self.annotations.list_alerts(&AlertFilter {
            project,
            state: Some(AlertState::Open),
        })?;
        self.events.publish(&Event::AlertsChanged(AlertsChanged {
            project,
            open: u32::try_from(open.len()).unwrap_or(u32::MAX),
        }));
        Ok(())
    }
}

fn count(report: &mut AlertReport, outcome: Outcome) {
    match outcome {
        Outcome::Quiet => {}
        Outcome::Baselined => report.baselines += 1,
        Outcome::Raised => report.raised += 1,
    }
}
