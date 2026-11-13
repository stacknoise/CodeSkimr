//! Git sources: shallow clone of one branch into a cache folder (blueprint section 7, "Fetch").
//!
//! An update clones the branch again into a private work folder and then swaps it in: the old copy
//! is parked, the new one is moved into place, and only then the old one is deleted. If anything
//! fails or the scan is cancelled, the old copy is back in place and the work folder is removed.

use std::fs;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use codeskimr_application::cancel::CancelToken;
use codeskimr_application::ports::FetchedSource;
use codeskimr_domain::{BranchName, CommitHash, FolderPath, GitUrl};

use crate::consts::{
    AUTH_FAILURE_HINTS, BRANCH_REF_PREFIX, CANCEL_POLL_MS, CANCELLED_DETAIL, MISSING_BRANCH_HINT,
    SHALLOW_CLONE_DEPTH, WORK_PREFIX, WORK_SEPARATOR,
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
    /// [`SourceError::GitFetchFailed`] (also when `cancel` was raised).
    pub fn fetch(
        &self,
        url: &GitUrl,
        branch: &BranchName,
        cancel: &CancelToken,
    ) -> Result<FetchedSource, SourceError> {
        if cancel.is_cancelled() {
            return Err(cancelled());
        }
        fs::create_dir_all(&self.cache_root).map_err(|error| SourceError::git_failed(&error))?;
        let target = self.cache_root.join(folder_name(url, branch));
        let work = WorkFolder::new(&self.cache_root);
        clone_polling(url, branch, &work.fresh, cancel)?;
        let commit = head_commit(&work.fresh)?;
        work.swap_into(&target)?;
        let root = FolderPath::new(target.to_string_lossy().into_owned())
            .map_err(|error| SourceError::git_failed(&error))?;
        Ok(FetchedSource {
            root,
            commit: Some(commit),
        })
    }
}

fn cancelled() -> SourceError {
    SourceError::GitFetchFailed {
        detail: CANCELLED_DETAIL.to_owned(),
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

/// The private folders of one fetch. Dropping it removes whatever is left in them.
struct WorkFolder {
    /// Where the new clone is prepared.
    fresh: PathBuf,
    /// Where the old copy is parked during the swap.
    parked: PathBuf,
}

impl WorkFolder {
    fn new(cache_root: &Path) -> Self {
        // Process, thread and time tell concurrent fetches apart without shared state.
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_nanos());
        let id = format!(
            "{WORK_PREFIX}{}{WORK_SEPARATOR}{:?}{WORK_SEPARATOR}{nanos}",
            std::process::id(),
            std::thread::current().id()
        );
        Self {
            fresh: cache_root.join(&id),
            parked: cache_root.join(format!("{id}{WORK_SEPARATOR}{WORK_PREFIX}")),
        }
    }

    /// Replaces `target` by the new clone; on failure the old copy is restored.
    fn swap_into(&self, target: &Path) -> Result<(), SourceError> {
        let had_old = target.exists();
        if had_old {
            fs::rename(target, &self.parked).map_err(|error| SourceError::git_failed(&error))?;
        }
        if let Err(error) = fs::rename(&self.fresh, target) {
            if had_old {
                // Best effort: the old copy is the only valid state left.
                let _ = fs::rename(&self.parked, target);
            }
            return Err(SourceError::git_failed(&error));
        }
        Ok(())
    }
}

impl Drop for WorkFolder {
    fn drop(&mut self) {
        // Leftovers of a failed fetch; nothing useful can be done if the removal fails.
        let _ = fs::remove_dir_all(&self.fresh);
        let _ = fs::remove_dir_all(&self.parked);
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
    } else if text.contains(MISSING_BRANCH_HINT) {
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
