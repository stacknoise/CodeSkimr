//! Git sources: shallow clone of one branch into a cache folder (blueprint section 7, "Fetch").
//!
//! An update clones the branch again next to the old copy and swaps the two, so a failed or
//! cancelled update never damages the copy that exists.

use std::fs;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use codeskimr_application::cancel::CancelToken;
use codeskimr_application::ports::FetchedSource;
use codeskimr_domain::{BranchName, CommitHash, FolderPath, GitUrl};

use crate::consts::{
    AUTH_FAILURE_HINTS, BRANCH_REF_PREFIX, CANCEL_POLL_MS, MISSING_BRANCH_HINTS,
    SHALLOW_CLONE_DEPTH, STAGING_SUFFIX,
};
use crate::error::{SourceError, error_chain_text};

/// Clones Git repositories below `cache_root`, one folder per URL and branch.
#[derive(Debug, Clone)]
pub struct GitCache {
    cache_root: PathBuf,
}

impl GitCache {
    /// Creates the cache; the folder is created on first use.
    #[must_use]
    pub fn new(cache_root: PathBuf) -> Self {
        Self { cache_root }
    }

    /// Makes the newest commit of `branch` available and returns where it lies.
    ///
    /// # Errors
    /// [`SourceError::BranchNotFound`], [`SourceError::AuthRequired`] or
    /// [`SourceError::GitFetchFailed`].
    pub fn fetch(
        &self,
        url: &GitUrl,
        branch: &BranchName,
        cancel: &CancelToken,
    ) -> Result<FetchedSource, SourceError> {
        let target = self.cache_root.join(folder_name(url, branch));
        let staging = staging_path(&target);
        fs::create_dir_all(&self.cache_root).map_err(|error| SourceError::git_failed(&error))?;
        remove_if_present(&staging)?;
        clone_polling(url, branch, &staging, cancel)?;
        let commit = head_commit(&staging)?;
        remove_if_present(&target)?;
        fs::rename(&staging, &target).map_err(|error| SourceError::git_failed(&error))?;
        let root = FolderPath::new(target.to_string_lossy().into_owned())
            .map_err(|error| SourceError::git_failed(&error))?;
        Ok(FetchedSource {
            root,
            commit: Some(commit),
        })
    }
}

/// A stable folder name for a URL and branch; it contains no part of the URL itself.
fn folder_name(url: &GitUrl, branch: &BranchName) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(url.as_str().as_bytes());
    hasher.update(&[0]);
    hasher.update(branch.as_str().as_bytes());
    hasher.finalize().to_hex().to_string()
}

fn staging_path(target: &Path) -> PathBuf {
    let mut name = target.as_os_str().to_owned();
    name.push(STAGING_SUFFIX);
    PathBuf::from(name)
}

fn remove_if_present(path: &Path) -> Result<(), SourceError> {
    match fs::remove_dir_all(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(SourceError::git_failed(&error)),
    }
}

/// Runs the clone while a helper thread turns the cancel token into Git's interrupt flag.
fn clone_polling(
    url: &GitUrl,
    branch: &BranchName,
    path: &Path,
    cancel: &CancelToken,
) -> Result<(), SourceError> {
    let interrupt = AtomicBool::new(false);
    let finished = AtomicBool::new(false);
    std::thread::scope(|scope| {
        scope.spawn(|| {
            while !finished.load(Ordering::Acquire) {
                if cancel.is_cancelled() {
                    interrupt.store(true, Ordering::Release);
                }
                std::thread::sleep(Duration::from_millis(CANCEL_POLL_MS));
            }
        });
        let result = clone_branch(url, branch, path, &interrupt);
        finished.store(true, Ordering::Release);
        result
    })
}

/// Shallow clone of one branch with its working tree.
fn clone_branch(
    url: &GitUrl,
    branch: &BranchName,
    path: &Path,
    interrupt: &AtomicBool,
) -> Result<(), SourceError> {
    let depth = NonZeroU32::new(SHALLOW_CLONE_DEPTH).unwrap_or(NonZeroU32::MIN);
    let classify = |error: &dyn std::error::Error| classify_failure(error, branch);
    let mut prepare = gix::prepare_clone(url.as_str(), path)
        .map_err(|error| classify(&error))?
        .with_shallow(gix::remote::fetch::Shallow::DepthAtRemote(depth))
        .with_ref_name(Some(&format!("{BRANCH_REF_PREFIX}{branch}")))
        .map_err(|error| SourceError::git_failed(&error))?;
    let (mut checkout, _) = prepare
        .fetch_then_checkout(gix::progress::Discard, interrupt)
        .map_err(|error| classify(&error))?;
    checkout
        .main_worktree(gix::progress::Discard, interrupt)
        .map_err(|error| classify(&error))?;
    Ok(())
}

/// Sorts a Git failure into the three codes a user can act on.
fn classify_failure(error: &dyn std::error::Error, branch: &BranchName) -> SourceError {
    let text = error_chain_text(error).to_lowercase();
    if AUTH_FAILURE_HINTS.iter().any(|hint| text.contains(hint)) {
        SourceError::AuthRequired
    } else if MISSING_BRANCH_HINTS.iter().any(|hint| text.contains(hint)) {
        SourceError::BranchNotFound {
            branch: branch.clone(),
        }
    } else {
        SourceError::git_failed(error)
    }
}

/// The commit the clone's `HEAD` points to.
fn head_commit(path: &Path) -> Result<CommitHash, SourceError> {
    let repository = gix::open(path).map_err(|error| SourceError::git_failed(&error))?;
    let id = repository
        .head_id()
        .map_err(|error| SourceError::git_failed(&error))?;
    CommitHash::new(id.to_hex().to_string()).map_err(|error| SourceError::git_failed(&error))
}
