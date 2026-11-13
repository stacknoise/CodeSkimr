//! Walking a folder: ignore rules, language detection, hashing (blueprint section 7, "Discover").

use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc;

use codeskimr_application::cancel::CancelToken;
use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{DiscoveredFile, FileDiscovery};
use codeskimr_domain::{ContentHash, FolderPath, LanguageId, SourceFile};
use ignore::{DirEntry, WalkBuilder, WalkState};

use crate::consts::GIT_DIR_NAME;
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
///
/// The result does not depend on the machine: only the `.gitignore` files inside the scanned
/// folder count (no global ignore file, no files of parent folders, no `.git/info/exclude`),
/// hidden files are scanned, and the `.git` folder is never entered.
///
/// If the scan is cancelled, `discover` returns the files found so far as `Ok`; the caller must
/// check its cancel token before using the list (the scan pipeline does).
///
/// A file or folder that cannot be read fails the whole run with `SRC-201`: dropping it silently
/// would make an incremental scan treat the file as deleted.
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

/// A file or folder that could not be read.
struct Unreadable;

/// State shared by the worker threads of one walk.
struct Shared<'a> {
    root: &'a Path,
    languages: &'a LanguageMap,
    previous: HashMap<&'a str, (&'a ContentHash, &'a LanguageId)>,
    cancel: &'a CancelToken,
    total_bytes: AtomicU64,
    limit_bytes: u64,
    too_large: AtomicBool,
    unreadable: AtomicBool,
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
                .map(|file| (file.path.as_str(), (&file.content_hash, &file.language)))
                .collect(),
            cancel,
            total_bytes: AtomicU64::new(0),
            limit_bytes: self.config.max_total_bytes,
            too_large: AtomicBool::new(false),
            unreadable: AtomicBool::new(false),
        };
        let mut files = walk(&shared, &self.config.excluded_folders);
        if shared.too_large.load(Ordering::Acquire) {
            return Err(SourceError::TooLarge {
                limit_bytes: self.config.max_total_bytes,
            }
            .into());
        }
        if shared.unreadable.load(Ordering::Acquire) {
            return Err(SourceError::PathNotFound { path: root.clone() }.into());
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
            match entry.ok().map(|entry| inspect(shared, &entry)) {
                Some(Ok(Some(file))) => {
                    // The receiver outlives the walk; a failed send cannot happen before it ends.
                    let _ = sender.send(file);
                }
                Some(Ok(None)) => {}
                Some(Err(Unreadable)) | None => shared.unreadable.store(true, Ordering::Release),
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
        .hidden(false)
        .git_global(false)
        .git_exclude(false)
        .ignore(false)
        .parents(false)
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
            .is_some_and(|name| name == GIT_DIR_NAME || excluded.iter().any(|item| item == name))
}

/// Turns a walked entry into a discovered file, `Ok(None)` if it is not one of ours.
///
/// The size limit is checked on the file size before the content is read.
fn inspect(shared: &Shared<'_>, entry: &DirEntry) -> Result<Option<DiscoveredFile>, Unreadable> {
    if !entry.file_type().is_some_and(|kind| kind.is_file()) {
        return Ok(None);
    }
    let Some(language) = shared.languages.detect(entry.path()) else {
        return Ok(None);
    };
    let Some(path) = relative_path(shared.root, entry.path()) else {
        return Ok(None);
    };
    let size = entry.metadata().map_err(|_| Unreadable)?.len();
    if shared.total_bytes.fetch_add(size, Ordering::AcqRel) + size > shared.limit_bytes {
        shared.too_large.store(true, Ordering::Release);
        return Ok(None);
    }
    let (_, content_hash) = hash_file(entry.path()).map_err(|_| Unreadable)?;
    let changed = shared.previous.get(path.as_str()) != Some(&(&content_hash, language));
    Ok(Some(DiscoveredFile {
        path,
        language: language.clone(),
        content_hash,
        changed,
    }))
}
