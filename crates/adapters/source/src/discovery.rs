//! Walking a folder: ignore rules, language detection, hashing (blueprint section 7, "Discover").

use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc;

use codeskimr_application::cancel::CancelToken;
use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{DiscoveredFile, FileDiscovery};
use codeskimr_domain::{ContentHash, FolderPath, SourceFile};
use ignore::{DirEntry, WalkBuilder, WalkState};

use crate::error::SourceError;
use crate::folder::open_folder;
use crate::hash::hash_file;
use crate::languages::LanguageMap;
use crate::paths::relative_path;

/// Settings of a discovery run that come from the configuration.
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    /// Folder names that are never entered, at any depth (`scan.excluded_folders`).
    pub excluded_folders: Vec<String>,
    /// Upper bound for the summed size of all discovered files, in bytes.
    pub max_total_bytes: u64,
}

/// Finds the files of a project with the `ignore` crate.
#[derive(Debug, Clone)]
pub struct WalkDiscovery {
    config: DiscoveryConfig,
    languages: LanguageMap,
}

impl WalkDiscovery {
    /// Creates the adapter.
    #[must_use]
    pub fn new(config: DiscoveryConfig, languages: LanguageMap) -> Self {
        Self { config, languages }
    }
}

/// State shared by the worker threads of one walk.
struct Shared<'a> {
    root: &'a Path,
    languages: &'a LanguageMap,
    previous: HashMap<&'a str, &'a ContentHash>,
    cancel: &'a CancelToken,
    total_bytes: AtomicU64,
    limit_bytes: u64,
    too_large: AtomicBool,
}

impl FileDiscovery for WalkDiscovery {
    fn discover(
        &self,
        root: &FolderPath,
        previous: &[SourceFile],
        cancel: &CancelToken,
    ) -> Result<Vec<DiscoveredFile>, CodedError> {
        open_folder(root)?;
        let shared = Shared {
            root: Path::new(root.as_str()),
            languages: &self.languages,
            previous: previous
                .iter()
                .map(|file| (file.path.as_str(), &file.content_hash))
                .collect(),
            cancel,
            total_bytes: AtomicU64::new(0),
            limit_bytes: self.config.max_total_bytes,
            too_large: AtomicBool::new(false),
        };
        let mut files = walk(&shared, &self.config.excluded_folders);
        if shared.too_large.load(Ordering::Acquire) {
            return Err(SourceError::TooLarge {
                limit_bytes: self.config.max_total_bytes,
            }
            .into());
        }
        files.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(files)
    }
}

/// Runs the parallel walk and collects what the workers found.
fn walk(shared: &Shared<'_>, excluded: &[String]) -> Vec<DiscoveredFile> {
    let (sender, receiver) = mpsc::channel();
    builder(shared.root, excluded).build_parallel().run(|| {
        let sender = sender.clone();
        Box::new(move |entry| {
            if shared.cancel.is_cancelled() || shared.too_large.load(Ordering::Acquire) {
                return WalkState::Quit;
            }
            if let Some(file) = entry.ok().and_then(|entry| inspect(shared, &entry)) {
                // The receiver outlives the walk; a failed send cannot happen before it ends.
                let _ = sender.send(file);
            }
            WalkState::Continue
        })
    });
    drop(sender);
    receiver.into_iter().collect()
}

/// A walker that honours `.gitignore` also outside Git repositories and skips excluded folders.
fn builder(root: &Path, excluded: &[String]) -> WalkBuilder {
    let excluded: Vec<String> = excluded.to_vec();
    let mut builder = WalkBuilder::new(root);
    builder
        .require_git(false)
        .follow_links(false)
        .filter_entry(move |entry| !is_excluded(entry, &excluded));
    builder
}

/// Whether the entry is a folder whose name is on the exclusion list.
fn is_excluded(entry: &DirEntry, excluded: &[String]) -> bool {
    let is_dir = entry.file_type().is_some_and(|kind| kind.is_dir());
    is_dir
        && entry
            .file_name()
            .to_str()
            .is_some_and(|name| excluded.iter().any(|item| item == name))
}

/// Turns a walked entry into a discovered file, or `None` if it is not one of ours.
fn inspect(shared: &Shared<'_>, entry: &DirEntry) -> Option<DiscoveredFile> {
    if !entry.file_type().is_some_and(|kind| kind.is_file()) {
        return None;
    }
    let language = shared.languages.detect(entry.path())?.clone();
    let path = relative_path(shared.root, entry.path())?;
    let (size, content_hash) = hash_file(entry.path()).ok()?;
    let total = shared.total_bytes.fetch_add(size, Ordering::AcqRel) + size;
    if total > shared.limit_bytes {
        shared.too_large.store(true, Ordering::Release);
        return None;
    }
    let changed = shared.previous.get(path.as_str()) != Some(&&content_hash);
    Some(DiscoveredFile {
        path,
        language,
        content_hash,
        changed,
    })
}
