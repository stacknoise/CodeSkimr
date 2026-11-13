//! Git sources: shallow clone of one branch into a cache folder (blueprint section 7, "Fetch").
//!
//! An update clones the branch again into a private work folder and then swaps it in: the old copy
//! is parked, the new one is moved into place, and only then the old one is deleted. If anything
//! fails or the scan is cancelled, the old copy is back in place and the work folder is removed.
//!
//! Secrets never reach the clone: a password in the URL is dropped before Git sees it (Git would
//! write the URL into `.git/config`), and a credential from the secret store is lent to the login
//! callback of the connection for exactly one attempt (blueprint 12.2, 12.3).

use std::fs;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use codeskimr_application::cancel::CancelToken;
use codeskimr_application::ports::{AuthError, Credential, FetchedSource};
use codeskimr_domain::{BranchName, CommitHash, FolderPath, GitUrl};
use gix::credentials::helper::Action;
use gix::credentials::protocol::Outcome;
use gix::sec::identity::Account;
use secrecy::ExposeSecret;

use crate::consts::{
    AUTH_FAILURE_HINTS, BRANCH_REF_PREFIX, CANCEL_POLL_MS, CANCELLED_DETAIL, MISSING_BRANCH_HINT,
    NO_LOGIN_MESSAGE, REJECTED_CREDENTIAL_HINT, SHALLOW_CLONE_DEPTH, WORK_PREFIX, WORK_SEPARATOR,
};
use crate::error::{SourceError, error_chain_text};
use crate::redact::without_password;

/// Lends the credential of a login to a closure, once per call. It reads from the secret store
/// every time and nothing is kept between calls.
type Lender = dyn Fn(&mut dyn FnMut(&Credential)) -> Result<(), AuthError> + Send + Sync;

/// How to log in to a remote that wants credentials.
#[derive(Clone)]
pub struct Login {
    lender: Arc<Lender>,
}

impl Login {
    /// A login that takes its credential from `lend`.
    pub fn new(
        lend: impl Fn(&mut dyn FnMut(&Credential)) -> Result<(), AuthError> + Send + Sync + 'static,
    ) -> Self {
        Self {
            lender: Arc::new(lend),
        }
    }
}

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
    /// Without `login` the system's Git credential helper decides how to log in. A password in
    /// `url` is ignored.
    ///
    /// # Errors
    /// [`SourceError::BranchNotFound`], [`SourceError::AuthRequired`],
    /// [`SourceError::Credential`] or [`SourceError::GitFetchFailed`] (also when `cancel` was
    /// raised).
    pub fn fetch(
        &self,
        url: &GitUrl,
        branch: &BranchName,
        login: Option<&Login>,
        cancel: &CancelToken,
    ) -> Result<FetchedSource, SourceError> {
        if cancel.is_cancelled() {
            return Err(cancelled());
        }
        fs::create_dir_all(&self.cache_root).map_err(|error| SourceError::git_failed(&error))?;
        // Git writes the URL it was given into `.git/config`, so it never gets a password.
        let url = GitUrl::new(without_password(url.as_str()))
            .map_err(|error| SourceError::git_failed(&error))?;
        let target = self.cache_root.join(folder_name(&url, branch));
        let work = WorkFolder::new(&self.cache_root);
        let attempt = Attempt {
            url: &url,
            branch,
            login,
            cancel,
        };
        clone_polling(&attempt, &work.fresh)?;
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

/// What one clone needs to know.
struct Attempt<'a> {
    url: &'a GitUrl,
    branch: &'a BranchName,
    login: Option<&'a Login>,
    cancel: &'a CancelToken,
}

/// Runs the clone while a helper thread turns the cancel token into Git's interrupt flag.
fn clone_polling(attempt: &Attempt<'_>, path: &Path) -> Result<(), SourceError> {
    let interrupt = AtomicBool::new(false);
    let finished = AtomicBool::new(false);
    std::thread::scope(|scope| {
        scope.spawn(|| {
            while !finished.load(Ordering::Acquire) {
                if attempt.cancel.is_cancelled() {
                    interrupt.store(true, Ordering::Release);
                }
                std::thread::sleep(Duration::from_millis(CANCEL_POLL_MS));
            }
        });
        let result = clone_branch(attempt, path, &interrupt);
        finished.store(true, Ordering::Release);
        result
    })
}

/// Shallow clone of one branch with its working tree.
fn clone_branch(
    attempt: &Attempt<'_>,
    path: &Path,
    interrupt: &AtomicBool,
) -> Result<(), SourceError> {
    let branch = attempt.branch;
    let depth = NonZeroU32::new(SHALLOW_CLONE_DEPTH).unwrap_or(NonZeroU32::MIN);
    let used_login = attempt.login.is_some();
    let classify = |error: &dyn std::error::Error| classify_failure(error, branch, used_login);
    let mut prepare = gix::prepare_clone(attempt.url.as_str(), path)
        .map_err(|error| classify(&error))?
        .with_shallow(gix::remote::fetch::Shallow::DepthAtRemote(depth))
        .with_ref_name(Some(&format!("{BRANCH_REF_PREFIX}{branch}")))
        .map_err(|error| SourceError::git_failed(&error))?;
    let refusal = LoginRefusal::default();
    if let Some(login) = attempt.login {
        let (login, refusal) = (login.clone(), refusal.clone());
        // Every request of the connection asks the secret store again; nothing is kept in between.
        prepare = prepare.configure_connection(move |connection| {
            let (login, refusal) = (login.clone(), refusal.clone());
            connection.set_credentials(move |action| answer(&login, &refusal, action));
            Ok(())
        });
    }
    let outcome = prepare.fetch_then_checkout(gix::progress::Discard, interrupt);
    let (mut checkout, _) = match outcome {
        Ok(done) => done,
        Err(error) => {
            return Err(refusal
                .take()
                .map_or_else(|| classify(&error), SourceError::from));
        }
    };
    checkout
        .main_worktree(gix::progress::Discard, interrupt)
        .map_err(|error| classify(&error))?;
    Ok(())
}

/// Why the secret store could not provide the credential, if that was the reason a login failed.
#[derive(Clone, Default)]
struct LoginRefusal(Arc<Mutex<Option<AuthError>>>);

impl LoginRefusal {
    fn set(&self, error: AuthError) {
        if let Ok(mut slot) = self.0.lock() {
            *slot = Some(error);
        }
    }

    fn take(&self) -> Option<AuthError> {
        self.0.lock().ok().and_then(|mut slot| slot.take())
    }
}

/// Answers one request of the connection. Only `Get` needs a secret; Git is not told to store or
/// erase anything, because the secret store is the only place that keeps credentials.
fn answer(login: &Login, refusal: &LoginRefusal, action: Action) -> gix::Result<Option<Outcome>> {
    let Action::Get(context) = action else {
        return Ok(None);
    };
    let mut outcome = None;
    let lent = (login.lender)(&mut |credential| {
        outcome = Some(Outcome {
            identity: Account {
                username: credential.user().expose_secret().to_owned(),
                password: credential.secret().expose_secret().to_owned(),
                oauth_refresh_token: None,
            },
            next: context.clone().into(),
        });
    });
    match lent {
        Ok(()) => Ok(outcome),
        Err(error) => {
            refusal.set(error);
            Err(gix::error::Exn::from(gix::error::message(NO_LOGIN_MESSAGE)).into())
        }
    }
}

/// Sorts a Git failure into the codes a user can act on.
fn classify_failure(
    error: &dyn std::error::Error,
    branch: &BranchName,
    used_login: bool,
) -> SourceError {
    let text = error_chain_text(error).to_lowercase();
    if used_login && text.contains(REJECTED_CREDENTIAL_HINT) {
        SourceError::Credential(AuthError::Rejected)
    } else if AUTH_FAILURE_HINTS.iter().any(|hint| text.contains(hint)) {
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
