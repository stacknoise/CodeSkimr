//! Fakes of the alert step: annotations, the symbols of a snapshot, the digest and the clock.
#![allow(dead_code, missing_docs)]

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use codeskimr_application::alerts::AlertStep;
use codeskimr_application::error::CodedError;
use codeskimr_application::events::EventBus;
use codeskimr_application::ports::{
    AlertDismissal, AlertFilter, AlertPurge, AlertRaise, AnnotationCreation, AnnotationFilter,
    AnnotationStore, AnnotationUpdate, Clock, ContentDigest, SameName, SymbolState, SymbolStates,
};
use codeskimr_codes::ErrorCode;
use codeskimr_domain::{
    Alert, AlertId, AlertState, Annotation, AnnotationContent, AnnotationId, AnnotationKind,
    AnnotationTarget, Author, ContentHash, Loc, ProjectId, QualifiedName, SnapshotId,
    SymbolBaseline, SymbolKind, SymbolName, Timestamp,
};

pub const PROJECT: ProjectId = ProjectId::new(7);

/// What the fake store holds.
#[derive(Default)]
pub struct Held {
    pub annotations: Vec<Annotation>,
    pub baselines: Vec<(AnnotationId, SymbolBaseline)>,
    pub alerts: Vec<Alert>,
}

/// Annotations in memory with the rules of the real store that the step relies on: at most one
/// open alert per annotation, a counter that grows with each raise.
#[derive(Default)]
pub struct MemoryAnnotations {
    held: Mutex<Held>,
    /// Statements that were asked for, by name.
    pub calls: Mutex<Vec<&'static str>>,
    /// Makes every call fail with this code.
    pub fail: Mutex<Option<ErrorCode>>,
    /// Makes `store_baseline` fail for this annotation only.
    pub fail_baseline_of: Mutex<Option<AnnotationId>>,
}

fn locked<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

impl MemoryAnnotations {
    pub fn held(&self) -> MutexGuard<'_, Held> {
        locked(&self.held)
    }

    fn enter(&self, call: &'static str) -> Result<(), CodedError> {
        locked(&self.calls).push(call);
        match *locked(&self.fail) {
            Some(code) => Err(CodedError::new(code)),
            None => Ok(()),
        }
    }

    pub fn calls_named(&self, call: &str) -> usize {
        locked(&self.calls)
            .iter()
            .filter(|seen| **seen == call)
            .count()
    }

    /// Adds a critical annotation on `name` and returns it.
    pub fn critical(&self, kind: SymbolKind, name: &str, parameters: &[&str]) -> Annotation {
        let mut held = self.held();
        let annotation = Annotation {
            id: AnnotationId::new(held.annotations.len() as u64 + 1),
            project: PROJECT,
            target: AnnotationTarget {
                kind,
                name: QualifiedName::new(name).unwrap(),
                parameters: parameters.iter().map(|text| (*text).to_owned()).collect(),
            },
            author: Author::new("anton").unwrap(),
            content: AnnotationContent {
                kind: AnnotationKind::Critical,
                title: None,
                text: None,
                status: None,
                due: None,
                assignee: None,
            },
            created: Timestamp(1),
            updated: Timestamp(1),
            without_target: false,
        };
        held.annotations.push(annotation.clone());
        annotation
    }

    pub fn open_alerts(&self) -> Vec<Alert> {
        self.held()
            .alerts
            .iter()
            .filter(|alert| alert.state == AlertState::Open)
            .cloned()
            .collect()
    }
}

impl AnnotationStore for MemoryAnnotations {
    fn create_annotation(&self, _: &AnnotationCreation) -> Result<Annotation, CodedError> {
        Err(CodedError::new(ErrorCode::UnexpectedInternalError))
    }

    fn update_annotation(&self, _: &AnnotationUpdate) -> Result<Annotation, CodedError> {
        Err(CodedError::new(ErrorCode::UnexpectedInternalError))
    }

    fn delete_annotation(&self, _: AnnotationId) -> Result<(), CodedError> {
        Err(CodedError::new(ErrorCode::UnexpectedInternalError))
    }

    fn list_annotations(&self, filter: &AnnotationFilter) -> Result<Vec<Annotation>, CodedError> {
        self.enter("list_annotations")?;
        Ok(self
            .held()
            .annotations
            .iter()
            .filter(|annotation| {
                filter
                    .kind
                    .is_none_or(|kind| annotation.content.kind == kind)
            })
            .cloned()
            .collect())
    }

    fn refresh_targets(&self, _: ProjectId, _: SnapshotId) -> Result<u64, CodedError> {
        self.enter("refresh_targets")?;
        Ok(0)
    }

    fn baseline(&self, annotation: AnnotationId) -> Result<Option<SymbolBaseline>, CodedError> {
        self.enter("baseline")?;
        Ok(self
            .held()
            .baselines
            .iter()
            .find(|(id, _)| *id == annotation)
            .map(|(_, baseline)| baseline.clone()))
    }

    fn store_baseline(
        &self,
        annotation: AnnotationId,
        baseline: &SymbolBaseline,
    ) -> Result<(), CodedError> {
        self.enter("store_baseline")?;
        if *locked(&self.fail_baseline_of) == Some(annotation) {
            return Err(CodedError::new(ErrorCode::UnexpectedInternalError));
        }
        let mut held = self.held();
        held.baselines.retain(|(id, _)| *id != annotation);
        held.baselines.push((annotation, baseline.clone()));
        Ok(())
    }

    fn raise_alert(&self, raise: &AlertRaise) -> Result<Alert, CodedError> {
        self.enter("raise_alert")?;
        let mut held = self.held();
        let open = held
            .alerts
            .iter_mut()
            .find(|alert| alert.annotation == raise.annotation && alert.state == AlertState::Open);
        if let Some(alert) = open {
            alert.counter += 1;
            alert.last_seen = raise.now;
            alert.sighting = raise.sighting.clone();
            return Ok(alert.clone());
        }
        let alert = Alert {
            id: AlertId::new(held.alerts.len() as u64 + 1),
            annotation: raise.annotation,
            state: AlertState::Open,
            counter: 1,
            first_seen: raise.now,
            last_seen: raise.now,
            dismissed: None,
            sighting: raise.sighting.clone(),
        };
        held.alerts.push(alert.clone());
        Ok(alert)
    }

    fn list_alerts(&self, filter: &AlertFilter) -> Result<Vec<Alert>, CodedError> {
        self.enter("list_alerts")?;
        Ok(self
            .held()
            .alerts
            .iter()
            .filter(|alert| filter.state.is_none_or(|state| alert.state == state))
            .cloned()
            .collect())
    }

    fn dismiss_alert(&self, dismissal: &AlertDismissal) -> Result<Alert, CodedError> {
        self.enter("dismiss_alert")?;
        let mut held = self.held();
        let Some(alert) = held
            .alerts
            .iter_mut()
            .find(|alert| alert.id == dismissal.alert)
        else {
            return Err(CodedError::new(ErrorCode::UnexpectedInternalError));
        };
        alert.state = AlertState::Dismissed;
        alert.dismissed = Some(dismissal.now);
        let (annotation, done) = (alert.annotation, alert.clone());
        if let Some(baseline) = &dismissal.baseline {
            held.baselines.retain(|(id, _)| *id != annotation);
            held.baselines.push((annotation, baseline.clone()));
        }
        Ok(done)
    }

    fn purge_dismissed(&self, _: &AlertPurge) -> Result<u64, CodedError> {
        Ok(0)
    }
}

/// The symbols of the newest snapshot, set by the test.
#[derive(Default)]
pub struct MemoryStates {
    pub symbols: Mutex<Vec<SymbolState>>,
    pub children: Mutex<Vec<ContentHash>>,
    pub references: AtomicU32,
    pub same_name: Mutex<Vec<SameName>>,
    pub reads: AtomicU32,
}

pub fn hash(text: &str) -> ContentHash {
    ContentHash::new(text).unwrap()
}

pub fn symbol(kind: SymbolKind, name: &str, content: &str) -> SymbolState {
    SymbolState {
        kind,
        name: QualifiedName::new(name).unwrap(),
        signature: Some(format!("{name}()")),
        content_hash: Some(hash(content)),
        loc: Loc(10),
        cc: 2,
    }
}

impl MemoryStates {
    pub fn set(&self, symbols: Vec<SymbolState>) {
        *locked(&self.symbols) = symbols;
    }

    pub fn reads(&self) -> u32 {
        self.reads.load(Ordering::SeqCst)
    }
}

impl SymbolStates for MemoryStates {
    fn states(
        &self,
        _: SnapshotId,
        names: &[QualifiedName],
    ) -> Result<Vec<SymbolState>, CodedError> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        Ok(locked(&self.symbols)
            .iter()
            .filter(|state| names.contains(&state.name))
            .cloned()
            .collect())
    }

    fn child_hashes(
        &self,
        _: SnapshotId,
        _: &QualifiedName,
    ) -> Result<Vec<ContentHash>, CodedError> {
        Ok(locked(&self.children).clone())
    }

    fn remaining_references(
        &self,
        _: SnapshotId,
        _: &QualifiedName,
        _: &SymbolName,
    ) -> Result<u32, CodedError> {
        Ok(self.references.load(Ordering::SeqCst))
    }

    fn same_simple_name(&self, _: SnapshotId, _: &SymbolName) -> Result<Vec<SameName>, CodedError> {
        Ok(locked(&self.same_name).clone())
    }
}

/// A digest that shows its parts, so that a test reads the hash.
pub struct ShowingDigest;

impl ContentDigest for ShowingDigest {
    fn digest(&self, parts: &[&str]) -> Result<ContentHash, CodedError> {
        ContentHash::new(format!("digest({})", parts.join("|"))).map_err(CodedError::caused_by)
    }
}

pub struct FixedClock;

impl Clock for FixedClock {
    fn now(&self) -> Timestamp {
        Timestamp(1_000)
    }
}

/// The step over the fakes, with the bus its events go to.
pub fn step(
    annotations: &Arc<MemoryAnnotations>,
    states: &Arc<MemoryStates>,
    events: &EventBus,
) -> AlertStep {
    AlertStep {
        annotations: annotations.clone(),
        states: states.clone(),
        digest: Arc::new(ShowingDigest),
        clock: Arc::new(FixedClock),
        events: events.clone(),
    }
}
