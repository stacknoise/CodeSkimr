//! Git sources: shallow clone of one branch into a cache folder (blueprint section 7, "Fetch").
//!
//! An update clones the branch again into a private work folder and then swaps it in: the old copy
//! is parked, the new one is moved into place, and only then the old one is deleted. If anything
//! fails or the scan is cancelled, the old copy is back in place and the work folder is removed.
//!
//! Secrets never reach the clone: `GitUrl` carries no credentials (Git would write the URL into
//! `.git/config`), and a credential from the secret store is lent to the login
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
    AUTH_FAILURE_HINTS, BRANCH_REF_PREFIX, CANCEL_POLL_MS, CANCELLED_DETAIL,
    CLONE_CONFIG_OVERRIDES, HTTP_NOT_FOUND_HINT, MISSING_BRANCH_HINT, NO_LOGIN_MESSAGE,
    SHALLOW_CLONE_DEPTH, URL_SCHEME_MARKER, WORK_PREFIX, WORK_SEPARATOR,
};
use crate::error::{SourceError, error_chain_text};

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
    /// Without `login` the system's Git credential helper decides how to log in.
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
        let target = self.cache_root.join(folder_name(url, branch));
        let work = WorkFolder::new(&self.cache_root);
        let attempt = Attempt {
            url,
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
    let classify = |error: &gix::Error| classify_failure(error, branch, used_login);
    let mut prepare = gix::prepare_clone(attempt.url.as_str(), path)
        .map_err(|error| classify(&error))?
        .with_in_memory_config_overrides(CLONE_CONFIG_OVERRIDES)
        .with_shallow(gix::remote::fetch::Shallow::DepthAtRemote(depth))
        .with_ref_name(Some(&format!("{BRANCH_REF_PREFIX}{branch}")))
        .map_err(|error| SourceError::git_failed(&error))?;
    let refusal = LoginRefusal::default();
    if let Some(login) = attempt.login {
        let (login, refusal) = (login.clone(), refusal.clone());
        let origin = attempt.url.origin().to_owned();
        // Every request of the connection asks the secret store again; nothing is kept in between.
        prepare = prepare.configure_connection(move |connection| {
            let (login, refusal, origin) = (login.clone(), refusal.clone(), origin.clone());
            connection.set_credentials(move |action| answer(&login, &refusal, &origin, action));
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
///
/// The credential is only given to the server the source points at. After a redirect to another
/// host Git asks again with the new address; that request gets no credential.
fn answer(
    login: &Login,
    refusal: &LoginRefusal,
    origin: &str,
    action: Action,
) -> gix::Result<Option<Outcome>> {
    let Action::Get(context) = action else {
        return Ok(None);
    };
    if !is_origin(&context, origin) {
        return Ok(None);
    }
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

/// Whether the request is for `origin` (`scheme://host[:port]`).
fn is_origin(context: &gix::credentials::protocol::Context, origin: &str) -> bool {
    let mut parts = context.clone();
    if parts.destructure_url_in_place(false).is_err() {
        return false;
    }
    let (Some(protocol), Some(host)) = (&parts.protocol, &parts.host) else {
        return false;
    };
    format!("{protocol}{URL_SCHEME_MARKER}{host}").eq_ignore_ascii_case(origin)
}

/// Sorts a Git failure into the codes a user can act on.
///
/// The failure class comes from the error itself (`gix` marks a refused login as unauthenticated
/// and a missing object as not found), not from its wording. Hosts such as GitHub answer an
/// invalid token for a repository they hide with 403 or 404, so a failed login with a credential
/// is reported as a rejected credential. The missing branch and a plain 404 are still recognised
/// by text, because `gix` has no variant for them; `tests/login.rs` and `tests/fetch.rs` pin the wording.
fn classify_failure(error: &gix::Error, branch: &BranchName, used_login: bool) -> SourceError {
    let text = error_chain_text(error).to_lowercase();
    let refused = error.is_unauthenticated() || error.is_permission_denied();
    if text.contains(MISSING_BRANCH_HINT) {
        SourceError::BranchNotFound {
            branch: branch.clone(),
        }
    } else if used_login && (refused || error.is_not_found() || text.contains(HTTP_NOT_FOUND_HINT))
    {
        SourceError::Credential(AuthError::Rejected)
    } else if refused || AUTH_FAILURE_HINTS.iter().any(|hint| text.contains(hint)) {
        SourceError::AuthRequired
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
