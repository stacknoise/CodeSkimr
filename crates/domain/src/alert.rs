//! Alerts: a system-critical symbol changed, was removed or lost its target (ADR 0068).

use crate::annotation::AnnotationId;
use crate::consts::{
    ALERT_CHANGED, ALERT_ORPHANED, ALERT_REMOVED, ALERT_STATE_DISMISSED, ALERT_STATE_OPEN,
    CHANGE_CC, CHANGE_CONTENT, CHANGE_LOC, CHANGE_SIGNATURE, TRIGGER_FETCH, TRIGGER_SCAN,
    TRIGGER_WATCHER,
};
use crate::file::{ContentHash, Loc};
use crate::qualified_name::QualifiedName;
use crate::snapshot::{CommitHash, SnapshotId, Timestamp};
use crate::vocabulary::{id_type, labeled_enum};

id_type!(
    /// Identifies an alert.
    AlertId
);

labeled_enum!(
    /// What happened to the critical symbol.
    AlertKind {
        /// The symbol still exists and differs from its baseline.
        Changed => ALERT_CHANGED,
        /// The symbol is gone.
        Removed => ALERT_REMOVED,
        /// The target key is not found any more, so the annotation has no symbol.
        Orphaned => ALERT_ORPHANED,
    }
);

labeled_enum!(
    /// Whether the alert still needs attention.
    AlertState {
        /// Not dealt with.
        Open => ALERT_STATE_OPEN,
        /// Removed by the user; kept for `annotations.dismissed_retention_days`.
        Dismissed => ALERT_STATE_DISMISSED,
    }
);

labeled_enum!(
    /// What made the scan that found the change.
    AlertTrigger {
        /// A scan that the user or the CLI started.
        Scan => TRIGGER_SCAN,
        /// The file watcher.
        Watcher => TRIGGER_WATCHER,
        /// A Git fetch.
        Fetch => TRIGGER_FETCH,
    }
);

labeled_enum!(
    /// Which part of a symbol differs from its baseline.
    AlertChange {
        /// The token hash of the symbol.
        Content => CHANGE_CONTENT,
        /// The signature.
        Signature => CHANGE_SIGNATURE,
        /// The cyclomatic complexity.
        Cc => CHANGE_CC,
        /// The lines of code.
        Loc => CHANGE_LOC,
    }
);

/// What a critical symbol looked like when the user last accepted it: the state that later scans
/// are compared with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolBaseline {
    /// Hash over the token stream of the symbol (whitespace and comments excluded).
    pub content_hash: ContentHash,
    /// Hash of the signature.
    pub signature_hash: ContentHash,
    /// Lines of code.
    pub loc: Loc,
    /// Cyclomatic complexity.
    pub cc: u32,
}

/// What a scan found out about a critical symbol.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlertSighting {
    /// What happened.
    pub kind: AlertKind,
    /// The snapshot that showed it.
    pub snapshot: SnapshotId,
    /// The commit, if the source is Git.
    pub commit: Option<CommitHash>,
    /// What started the scan.
    pub trigger: AlertTrigger,
    /// For `changed`: which parts differ.
    pub changes: Vec<AlertChange>,
    /// For `removed`: how many references to the symbol remain (provisional at stage 1).
    pub references: Option<u32>,
    /// For `orphaned`: the closest symbol with the same name, if any.
    pub closest: Option<QualifiedName>,
    /// For `changed`: what the symbol looked like at this sighting. A later scan raises the alert's
    /// counter only if the symbol looks different again, so a scan that did not touch it counts for
    /// nothing. `None` for `removed` and `orphaned`.
    pub observed: Option<SymbolBaseline>,
}

/// A stored alert. There is at most one open alert per annotation; a further sighting raises its
/// counter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Alert {
    /// Identity.
    pub id: AlertId,
    /// The critical annotation it is about.
    pub annotation: AnnotationId,
    /// Open or dismissed.
    pub state: AlertState,
    /// How often the symbol was seen changed while the alert was open, at least one.
    pub counter: u32,
    /// When it was raised first.
    pub first_seen: Timestamp,
    /// When it was seen last.
    pub last_seen: Timestamp,
    /// When it was dismissed.
    pub dismissed: Option<Timestamp>,
    /// The latest sighting.
    pub sighting: AlertSighting,
}
