//! Watching a folder for changes with debouncing (blueprint section 7, "Watch mode").
//!
//! Events are collected until the folder was quiet for the debounce window, but never longer than
//! the maximum wait: a build that writes all the time cannot starve the listener.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use codeskimr_application::events::{Event as AppEvent, EventBus, FilesChanged};
use codeskimr_domain::{FolderPath, ProjectId, RelativePath};
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use crate::error::SourceError;
use crate::paths::relative_path;

/// A running watch. Dropping it stops the watcher, delivers what is pending and waits for the
/// watcher thread. Do not drop it from inside the `on_change` callback: that would wait for itself.
#[derive(Debug)]
pub struct WatchHandle {
    watcher: Option<RecommendedWatcher>,
    worker: Option<JoinHandle<()>>,
}

impl Drop for WatchHandle {
    fn drop(&mut self) {
        // Dropping the watcher closes the event channel, which ends the worker.
        self.watcher.take();
        if let Some(worker) = self.worker.take() {
            // A panic in the callback was already reported by the thread itself.
            let _ = worker.join();
        }
    }
}

/// Settings of a watch.
#[derive(Debug, Clone)]
pub struct WatchConfig {
    /// Events within this quiet window become one batch (`watcher.debounce_ms`).
    pub debounce: Duration,
    /// A batch is delivered at the latest this long after its first event.
    pub max_wait: Duration,
    /// Folder names whose changes are ignored (`scan.excluded_folders`).
    pub excluded_folders: Vec<String>,
}

/// What the watcher reports.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WatchBatch {
    /// Changed paths, sorted and without duplicates.
    pub paths: Vec<RelativePath>,
    /// The operating system lost events (queue overflow or a watch error); the listener should
    /// scan the whole folder instead of trusting `paths`.
    pub rescan: bool,
}

/// Starts watching `root`. `on_change` receives each batch on the watcher's own thread.
///
/// # Errors
/// [`SourceError::WatcherFailed`] if the operating system refuses to watch the folder.
pub fn watch(
    root: &FolderPath,
    config: WatchConfig,
    on_change: impl Fn(WatchBatch) + Send + 'static,
) -> Result<WatchHandle, SourceError> {
    let root_path = PathBuf::from(root.as_str());
    let (sender, receiver) = mpsc::channel();
    let mut watcher =
        notify::recommended_watcher(sender).map_err(|error| SourceError::watcher_failed(&error))?;
    watcher
        .watch(&root_path, RecursiveMode::Recursive)
        .map_err(|error| SourceError::watcher_failed(&error))?;
    let worker = std::thread::spawn(move || {
        collect_batches(&root_path, &config, &receiver, &on_change);
    });
    Ok(WatchHandle {
        watcher: Some(watcher),
        worker: Some(worker),
    })
}

/// Watches the folder of `project` and publishes `FilesChanged` on `events` for every debounced
/// batch. Whoever reacts to it (watch mode) starts an incremental scan; the budget of blueprint 16
/// starts when the event is published.
///
/// # Errors
/// [`SourceError::WatcherFailed`] if the operating system refuses to watch the folder.
pub fn publish_changes(
    project: ProjectId,
    root: &FolderPath,
    config: WatchConfig,
    events: EventBus,
) -> Result<WatchHandle, SourceError> {
    watch(root, config, move |batch| {
        events.publish(&AppEvent::FilesChanged(FilesChanged {
            project,
            files: u32::try_from(batch.paths.len()).unwrap_or(u32::MAX),
            rescan: batch.rescan,
        }));
    })
}

type Event = notify::Result<notify::Event>;

/// The events collected since the last report.
#[derive(Default)]
struct Collected {
    batch: WatchBatch,
    paths: BTreeSet<RelativePath>,
    /// When the first and the latest relevant event arrived.
    first_last: Option<(Instant, Instant)>,
}

impl Collected {
    /// How long to wait before the batch is due: the rest of the quiet window, cut short by the
    /// maximum wait. Without events the full window.
    fn wait(&self, config: &WatchConfig) -> Duration {
        self.first_last.map_or(config.debounce, |(first, last)| {
            config
                .debounce
                .saturating_sub(last.elapsed())
                .min(config.max_wait.saturating_sub(first.elapsed()))
        })
    }

    fn note_activity(&mut self) {
        let now = Instant::now();
        let first = self.first_last.map_or(now, |(first, _)| first);
        self.first_last = Some((first, now));
    }
}

/// Gathers events and reports them when the folder is quiet or the maximum wait is over. Events
/// that are ignored never restart the quiet window.
fn collect_batches(
    root: &Path,
    config: &WatchConfig,
    receiver: &mpsc::Receiver<Event>,
    on_change: &impl Fn(WatchBatch),
) {
    let mut collected = Collected::default();
    loop {
        match receiver.recv_timeout(collected.wait(config)) {
            Ok(event) => {
                if add_event(root, config, event, &mut collected) {
                    collected.note_activity();
                }
                if collected.wait(config).is_zero() {
                    flush(&mut collected, on_change);
                }
            }
            Err(RecvTimeoutError::Timeout) => flush(&mut collected, on_change),
            Err(RecvTimeoutError::Disconnected) => {
                flush(&mut collected, on_change);
                return;
            }
        }
    }
}

fn flush(collected: &mut Collected, on_change: &impl Fn(WatchBatch)) {
    let finished = std::mem::take(collected);
    if finished.paths.is_empty() && !finished.batch.rescan {
        return;
    }
    on_change(WatchBatch {
        paths: finished.paths.into_iter().collect(),
        rescan: finished.batch.rescan,
    });
}

/// Records one event. Returns whether it is relevant, so that ignored events do not count as
/// activity.
fn add_event(root: &Path, config: &WatchConfig, event: Event, collected: &mut Collected) -> bool {
    let Ok(event) = event else {
        collected.batch.rescan = true;
        return true;
    };
    if matches!(event.kind, EventKind::Access(_)) {
        return false;
    }
    let rescan = event.need_rescan();
    collected.batch.rescan |= rescan;
    let relevant: Vec<RelativePath> = event
        .paths
        .iter()
        .filter(|path| !is_below_excluded(path, root, &config.excluded_folders))
        .filter_map(|path| relative_path(root, path))
        .collect();
    let any = rescan || !relevant.is_empty();
    collected.paths.extend(relevant);
    any
}

fn is_below_excluded(path: &Path, root: &Path, excluded: &[String]) -> bool {
    path.strip_prefix(root).is_ok_and(|relative| {
        relative.components().any(|part| {
            part.as_os_str()
                .to_str()
                .is_some_and(|name| excluded.iter().any(|item| item == name))
        })
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::Duration;

    use notify::event::{AccessKind, CreateKind};

    use super::*;

    fn config() -> WatchConfig {
        WatchConfig {
            debounce: Duration::from_millis(10),
            max_wait: Duration::from_millis(100),
            excluded_folders: vec!["target".to_owned()],
        }
    }

    fn event(kind: EventKind, path: &str) -> notify::Event {
        notify::Event::new(kind).add_path(PathBuf::from(path))
    }

    #[test]
    fn access_events_are_not_changes() {
        let mut collected = Collected::default();
        let access = Ok(event(EventKind::Access(AccessKind::Any), "/r/a.rs"));
        assert!(!add_event(
            Path::new("/r"),
            &config(),
            access,
            &mut collected
        ));
        assert!(collected.paths.is_empty());
    }

    #[test]
    fn excluded_paths_are_not_relevant() {
        let mut collected = Collected::default();
        let skipped = Ok(event(EventKind::Create(CreateKind::File), "/r/target/x.rs"));
        assert!(!add_event(
            Path::new("/r"),
            &config(),
            skipped,
            &mut collected
        ));
        let kept = Ok(event(EventKind::Create(CreateKind::File), "/r/src/x.rs"));
        assert!(add_event(Path::new("/r"), &config(), kept, &mut collected));
        assert_eq!(collected.paths.len(), 1);
    }

    #[test]
    fn a_watch_error_asks_for_a_rescan() {
        let mut collected = Collected::default();
        let failure = Err(notify::Error::generic("overflow"));
        assert!(add_event(
            Path::new("/r"),
            &config(),
            failure,
            &mut collected
        ));
        assert!(collected.batch.rescan);
    }
}
