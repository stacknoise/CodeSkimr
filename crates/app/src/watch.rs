//! `codeskimr scan --watch`: scans once, then scans again whenever files of the folder change.
//!
//! The file watcher (`codeskimr_source::publish_changes`) debounces the operating system's events
//! and publishes `FilesChanged` on the event bus. This module is the one that reacts: it turns
//! the events that piled up into one incremental scan, or into a full scan if the operating system
//! lost events. Scans never overlap; changes that arrive during a scan wait for the next one.

use codeskimr_application::error::CodedError;
use codeskimr_application::events::{Event, FilesChanged, Subscription};
use codeskimr_cli::ScanSummary;
use codeskimr_codes::ErrorCode;
use codeskimr_domain::{FolderPath, ProjectId, ProjectSource, ScanMode};
use codeskimr_source::{WatchConfig, publish_changes};
use std::time::{Duration, Instant};

use crate::scan::Session;
use crate::source_arg::SourceSpec;

/// What watch mode tells the caller.
#[derive(Debug)]
pub enum Report {
    /// The first, full scan is done and watching has begun.
    Initial(ScanSummary),
    /// Changed files were scanned again.
    Updated {
        /// The outcome of the scan.
        summary: ScanSummary,
        /// Number of files the watcher reported since the previous scan.
        changed: u32,
        /// How long the scan took, in milliseconds.
        millis: u64,
    },
    /// A scan failed; watching goes on, the next change tries again.
    Failed(CodedError),
}

/// The changes that piled up while nothing was scanning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Pending {
    files: u32,
    rescan: bool,
}

impl Pending {
    fn from_event(change: &FilesChanged) -> Self {
        Self {
            files: change.files,
            rescan: change.rescan,
        }
    }

    fn merge(self, change: &FilesChanged) -> Self {
        Self {
            files: self.files.saturating_add(change.files),
            rescan: self.rescan || change.rescan,
        }
    }

    /// Lost events cannot be trusted, so they ask for a full scan.
    const fn mode(self) -> ScanMode {
        if self.rescan {
            ScanMode::Full
        } else {
            ScanMode::Incremental
        }
    }
}

/// Scans the project, then keeps scanning when its folder changes. Returns only on an error that
/// ends watching; the user ends it with Ctrl+C.
///
/// The watcher starts before the first scan, so a file that changes during it is scanned again
/// right afterwards.
///
/// # Errors
/// - `CLI-102` if the project is a Git repository.
/// - `SRC-602` if the operating system refuses to watch the folder.
/// - Any coded failure of the first scan. Later failures are reported and watching continues.
pub fn run(session: &Session, report: &mut impl FnMut(Report)) -> Result<(), CodedError> {
    let folder = watched_folder(session)?;
    let changes = session.events().subscribe();
    let _watch = publish_changes(
        session.project().id,
        &folder,
        watch_config(session),
        session.events().clone(),
    )
    .map_err(CodedError::caused_by)?;
    report(Report::Initial(session.scan(ScanMode::Full)?));
    follow(
        &changes,
        session.project().id,
        |mode| session.scan(mode),
        report,
    );
    Ok(())
}

/// Fails with `CLI-102` for a source that is not a folder, before anything is registered.
///
/// # Errors
/// `CLI-102` for a Git source.
pub fn require_folder(spec: &SourceSpec) -> Result<(), CodedError> {
    match spec.source {
        ProjectSource::Folder(_) => Ok(()),
        ProjectSource::Git { .. } => Err(CodedError::new(ErrorCode::WatchNeedsFolder)),
    }
}

fn watched_folder(session: &Session) -> Result<FolderPath, CodedError> {
    match &session.project().source {
        ProjectSource::Folder(folder) => Ok(folder.clone()),
        ProjectSource::Git { .. } => Err(CodedError::new(ErrorCode::WatchNeedsFolder)),
    }
}

fn watch_config(session: &Session) -> WatchConfig {
    let config = session.config();
    WatchConfig {
        debounce: Duration::from_millis(u64::from(config.watcher.debounce_ms)),
        max_wait: Duration::from_millis(u64::from(config.watcher.max_wait_ms)),
        excluded_folders: config.scan.excluded_folders.clone(),
    }
}

/// Scans again for every group of changes until the bus is gone.
fn follow(
    changes: &Subscription,
    project: ProjectId,
    mut scan: impl FnMut(ScanMode) -> Result<ScanSummary, CodedError>,
    report: &mut impl FnMut(Report),
) {
    while let Some(pending) = next_pending(changes, project) {
        let started = Instant::now();
        report(match scan(pending.mode()) {
            Ok(summary) => Report::Updated {
                summary,
                changed: pending.files,
                millis: u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
            },
            Err(error) => Report::Failed(error),
        });
    }
}

/// Waits for the next change of `project` and takes everything that already waits behind it.
/// `None` once nobody can publish any more.
fn next_pending(changes: &Subscription, project: ProjectId) -> Option<Pending> {
    let mut pending = loop {
        if let Event::FilesChanged(change) = changes.recv().ok()?
            && change.project == project
        {
            break Pending::from_event(&change);
        }
    };
    for event in changes.try_iter() {
        if let Event::FilesChanged(change) = event
            && change.project == project
        {
            pending = pending.merge(&change);
        }
    }
    Some(pending)
}

#[cfg(test)]
mod tests {
    use codeskimr_application::events::EventBus;

    use super::*;

    const OURS: u64 = 7;
    const OTHER: u64 = 8;

    fn change(project: u64, files: u32, rescan: bool) -> Event {
        Event::FilesChanged(FilesChanged {
            project: ProjectId::new(project),
            files,
            rescan,
        })
    }

    fn summary() -> ScanSummary {
        ScanSummary {
            project: String::new(),
            snapshot: 1,
            files: 0,
            symbols: 0,
            references: 0,
            seconds: 0.0,
        }
    }

    /// Feeds `events`, closes the bus and returns the modes that were scanned and the reports.
    fn replay(events: &[Event], fail: bool) -> (Vec<ScanMode>, Vec<Report>) {
        let bus = EventBus::new();
        let changes = bus.subscribe();
        for event in events {
            bus.publish(event);
        }
        drop(bus);
        let mut modes = Vec::new();
        let mut reports = Vec::new();
        follow(
            &changes,
            ProjectId::new(OURS),
            |mode| {
                modes.push(mode);
                if fail {
                    Err(CodedError::new(ErrorCode::UnexpectedInternalError))
                } else {
                    Ok(summary())
                }
            },
            &mut |report| reports.push(report),
        );
        (modes, reports)
    }

    #[test]
    fn changes_that_piled_up_become_one_incremental_scan() {
        let (modes, reports) = replay(&[change(OURS, 2, false), change(OURS, 3, false)], false);
        assert_eq!(modes, [ScanMode::Incremental]);
        assert!(matches!(reports[..], [Report::Updated { changed: 5, .. }]));
    }

    #[test]
    fn lost_events_ask_for_a_full_scan() {
        let (modes, _) = replay(&[change(OURS, 1, false), change(OURS, 0, true)], false);
        assert_eq!(modes, [ScanMode::Full]);
    }

    #[test]
    fn other_projects_and_other_events_are_ignored() {
        let (modes, _) = replay(&[change(OTHER, 4, false)], false);
        assert_eq!(modes, Vec::<ScanMode>::new());
    }

    #[test]
    fn a_failed_scan_is_reported_and_watching_goes_on() {
        let (modes, reports) = replay(&[change(OURS, 1, false)], true);
        assert_eq!(modes.len(), 1);
        assert!(matches!(reports[..], [Report::Failed(_)]));
    }
}
