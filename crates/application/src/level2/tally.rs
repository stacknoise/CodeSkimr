//! Putting the runs of several workspaces of one indexer together (ADR 0100).

use codeskimr_domain::Snapshot;

use super::report::{Level2Outcome, Skipped};

/// The runs of one indexer over the workspaces of a project, added up.
#[derive(Debug, Default)]
pub(super) struct Tally {
    files: u64,
    stale: u64,
    mapped: u64,
    dropped: u64,
    outside_scan: u64,
    failed_workspaces: u64,
    /// The snapshot after the last run that replaced references.
    fresh: Option<Snapshot>,
    /// Why the first workspace that did not run through gave up.
    first_failure: Option<Skipped>,
    cancelled: bool,
}

impl Tally {
    /// A workspace that ran through. Anything but a checked outcome is not added.
    pub(super) fn checked(&mut self, outcome: &Level2Outcome, fresh: Snapshot) {
        if let Level2Outcome::Checked {
            files,
            stale,
            mapped,
            dropped,
            outside_scan,
            ..
        } = outcome
        {
            self.files = self.files.saturating_add(*files);
            self.stale = self.stale.saturating_add(*stale);
            self.mapped = self.mapped.saturating_add(*mapped);
            self.dropped = self.dropped.saturating_add(*dropped);
            self.outside_scan = self.outside_scan.saturating_add(*outside_scan);
            self.fresh = Some(fresh);
        }
    }

    /// A workspace that did not run through; its files keep level 1 and the report counts it.
    pub(super) fn failed(&mut self, reason: Skipped) {
        self.failed_workspaces = self.failed_workspaces.saturating_add(1);
        self.first_failure.get_or_insert(reason);
    }

    /// The scan was cancelled: the workspaces after this one are not run.
    pub(super) fn cancel(&mut self) {
        self.cancelled = true;
    }

    /// The outcome of the indexer: checked if at least one workspace was, else the first reason
    /// why none was.
    pub(super) fn outcome(self) -> Result<(Level2Outcome, Snapshot), Skipped> {
        let Some(fresh) = self.fresh else {
            return Err(if self.cancelled {
                Skipped::Cancelled
            } else {
                self.first_failure.unwrap_or(Skipped::NoProjectFile)
            });
        };
        let checked = Level2Outcome::Checked {
            files: self.files,
            stale: self.stale,
            mapped: self.mapped,
            dropped: self.dropped,
            outside_scan: self.outside_scan,
            failed_workspaces: self.failed_workspaces,
        };
        Ok((checked, fresh))
    }
}
