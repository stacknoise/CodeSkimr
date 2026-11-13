//! Watching a folder for changes with debouncing (blueprint section 7, "Watch mode").

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::thread::JoinHandle;
use std::time::Duration;

use codeskimr_domain::{FolderPath, RelativePath};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};

use crate::error::SourceError;
use crate::paths::relative_path;

/// A running watch. Dropping it stops the watcher and its thread.
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
            let _ = worker.join();
        }
    }
}

/// Settings of a watch.
#[derive(Debug, Clone)]
pub struct WatchConfig {
    /// Events within this window become one batch (`watcher.debounce_ms`).
    pub debounce: Duration,
    /// Folder names whose changes are ignored (`scan.excluded_folders`).
    pub excluded_folders: Vec<String>,
}

/// Starts watching `root`. `on_change` receives each debounced batch of changed paths, sorted
/// and without duplicates, on the watcher's own thread.
///
/// # Errors
/// [`SourceError::WatcherFailed`] if the operating system refuses to watch the folder.
pub fn watch(
    root: &FolderPath,
    config: WatchConfig,
    on_change: impl Fn(Vec<RelativePath>) + Send + 'static,
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

type Event = notify::Result<notify::Event>;

/// Gathers events until the channel is quiet for one debounce window, then reports them.
fn collect_batches(
    root: &Path,
    config: &WatchConfig,
    receiver: &mpsc::Receiver<Event>,
    on_change: &impl Fn(Vec<RelativePath>),
) {
    let mut pending = BTreeSet::new();
    loop {
        match receiver.recv_timeout(config.debounce) {
            Ok(event) => add_event(root, config, event, &mut pending),
            Err(RecvTimeoutError::Timeout) => flush(&mut pending, on_change),
            Err(RecvTimeoutError::Disconnected) => {
                flush(&mut pending, on_change);
                return;
            }
        }
    }
}

fn flush(pending: &mut BTreeSet<RelativePath>, on_change: &impl Fn(Vec<RelativePath>)) {
    if !pending.is_empty() {
        on_change(std::mem::take(pending).into_iter().collect());
    }
}

/// Adds the paths of one event that are not below an excluded folder.
fn add_event(
    root: &Path,
    config: &WatchConfig,
    event: Event,
    pending: &mut BTreeSet<RelativePath>,
) {
    let Ok(event) = event else { return };
    for path in &event.paths {
        let Some(relative) = relative_path(root, path) else {
            continue;
        };
        if !is_below_excluded(path, root, &config.excluded_folders) {
            pending.insert(relative);
        }
    }
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
