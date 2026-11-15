//! Updates an earlier clone instead of cloning again (issue #36): the Git data of the old copy is
//! duplicated into the work folder, the new commit is fetched into it (only what is new travels
//! over the connection) and the working tree is written from it. The old copy is not touched, so
//! the swap and its rollback work as for a fresh clone.

use std::fs;
use std::io;
use std::num::NonZeroU32;
use std::path::Path;
use std::sync::atomic::AtomicBool;

use gix::progress::Discard;
use gix::remote::Direction;
use gix::remote::fetch::{Shallow, Tags};

use super::{Attempt, LoginRefusal, answer};
use crate::consts::{
    BRANCH_REF_PREFIX, CLONE_CONFIG_OVERRIDES, GIT_DIR_NAME, HEAD_MISMATCH_DETAIL,
    MAX_PACKS_BEFORE_RECLONE, NO_WORKTREE_DETAIL, OBJECTS_DIR_NAME, PACK_DIR_NAME, PACK_EXTENSION,
    REFSPEC_FORCE, REFSPEC_SEPARATOR, REMOTE_REF_PREFIX, SHALLOW_CLONE_DEPTH,
    TOO_MANY_PACKS_DETAIL, UPDATE_REFLOG_MESSAGE,
};
use crate::error::SourceError;

fn failed(error: &dyn std::error::Error) -> SourceError {
    SourceError::git_failed(error)
}

/// Whether `old` holds a clone that can be updated.
pub(super) fn can_update(old: &Path) -> bool {
    old.join(GIT_DIR_NAME).is_dir()
}

/// Refuses an update when the old clone holds [`MAX_PACKS_BEFORE_RECLONE`] packs or more: the
/// branch is cloned again then, which drops the objects of earlier commits (issue #363).
pub(super) fn check_pack_count(old: &Path) -> Result<(), SourceError> {
    let dir = old
        .join(GIT_DIR_NAME)
        .join(OBJECTS_DIR_NAME)
        .join(PACK_DIR_NAME);
    let Ok(entries) = fs::read_dir(dir) else {
        return Ok(());
    };
    let packs = entries
        .flatten()
        .filter(|entry| {
            entry
                .path()
                .extension()
                .is_some_and(|ext| ext == PACK_EXTENSION)
        })
        .count();
    if packs >= MAX_PACKS_BEFORE_RECLONE {
        return Err(failed(&io::Error::other(TOO_MANY_PACKS_DETAIL)));
    }
    Ok(())
}

/// Prepares the working tree of the newest commit of the branch in `path`, starting from the clone
/// in `old`. Any error means "clone again"; the caller decides.
pub(super) fn update_branch(
    attempt: &Attempt<'_>,
    old: &Path,
    path: &Path,
    interrupt: &AtomicBool,
) -> Result<(), SourceError> {
    fs::create_dir_all(path).map_err(|error| failed(&error))?;
    duplicate(&old.join(GIT_DIR_NAME), &path.join(GIT_DIR_NAME), false)
        .map_err(|error| failed(&error))?;
    let options = gix::open::Options::default().config_overrides(CLONE_CONFIG_OVERRIDES);
    let mut repo = gix::open_opts(path, options).map_err(|error| failed(&error))?;
    repo.committer_or_set_generic_fallback()
        .map_err(|error| failed(&error))?;
    let tip = fetch_tip(attempt, &repo, interrupt)?;
    let branch = attempt.branch;
    repo.reference(
        format!("{BRANCH_REF_PREFIX}{branch}"),
        tip,
        gix::refs::transaction::PreviousValue::Any,
        UPDATE_REFLOG_MESSAGE,
    )
    .map_err(|error| failed(&error))?;
    // HEAD must lead to the new commit, or the copy is no good as a clone.
    let head = repo.head_id().map_err(|error| failed(&error))?;
    if head != tip {
        return Err(failed(&io::Error::other(HEAD_MISMATCH_DETAIL)));
    }
    write_tree(&repo, tip, interrupt)
}

/// Fetches the branch (shallow, nothing else) and returns the commit it now points to.
fn fetch_tip(
    attempt: &Attempt<'_>,
    repo: &gix::Repository,
    interrupt: &AtomicBool,
) -> Result<gix::ObjectId, SourceError> {
    let branch = attempt.branch;
    let tracking = format!("{REMOTE_REF_PREFIX}{branch}");
    let spec = format!("{REFSPEC_FORCE}{BRANCH_REF_PREFIX}{branch}{REFSPEC_SEPARATOR}{tracking}");
    let remote = repo
        .remote_at(attempt.url.as_str())
        .map_err(|error| failed(&error))?
        .with_fetch_tags(Tags::None)
        .with_refspecs([spec.as_str()], Direction::Fetch)
        .map_err(|error| failed(&error))?;
    let mut connection = remote
        .connect(Direction::Fetch)
        .map_err(|error| failed(&error))?;
    if let Some(login) = attempt.login {
        let (login, refusal) = (login.clone(), LoginRefusal::default());
        let origin = attempt.url.origin().to_owned();
        connection.set_credentials(move |action| answer(&login, &refusal, &origin, action));
    }
    let depth = NonZeroU32::new(SHALLOW_CLONE_DEPTH).unwrap_or(NonZeroU32::MIN);
    connection
        .prepare_fetch(Discard, gix::remote::ref_map::Options::default())
        .map_err(|error| failed(&error))?
        .with_shallow(Shallow::DepthAtRemote(depth))
        .receive(Discard, interrupt)
        .map_err(|error| failed(&error))?;
    let id = repo
        .find_reference(tracking.as_str())
        .map_err(|error| failed(&error))?
        .id()
        .detach();
    Ok(id)
}

/// Writes the files of the commit into the work folder, which holds nothing but the Git data, and
/// the index that belongs to them.
fn write_tree(
    repo: &gix::Repository,
    commit: gix::ObjectId,
    interrupt: &AtomicBool,
) -> Result<(), SourceError> {
    let workdir = repo
        .workdir()
        .ok_or_else(|| failed(&io::Error::other(NO_WORKTREE_DETAIL)))?;
    let tree = repo
        .find_commit(commit)
        .and_then(|found| found.tree_id())
        .map_err(|error| failed(&error))?;
    let mut index = repo
        .index_from_tree(&tree)
        .map_err(|error| failed(&error))?;
    let mut options = repo
        .checkout_options(gix::worktree::stack::state::attributes::Source::IdMapping)
        .map_err(|error| failed(&error))?;
    options.destination_is_initially_empty = true;
    let objects = repo
        .objects
        .clone()
        .into_arc()
        .map_err(|error| failed(&error))?;
    gix::worktree::state::checkout(
        &mut index, workdir, objects, &Discard, &Discard, interrupt, options,
    )
    .map_err(|error| failed(&error))?;
    index
        .write(gix::index::write::Options::default())
        .map_err(|error| failed(&error))
}

/// Copies the Git data folder `from` to `to`. Files below `objects` are never changed once they
/// are written, so they are linked instead of copied where the file system allows it.
fn duplicate(from: &Path, to: &Path, in_objects: bool) -> io::Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let (source, target) = (entry.path(), to.join(entry.file_name()));
        let objects = in_objects || entry.file_name() == OBJECTS_DIR_NAME;
        if entry.file_type()?.is_dir() {
            duplicate(&source, &target, objects)?;
        } else if !(objects && fs::hard_link(&source, &target).is_ok()) {
            fs::copy(&source, &target)?;
        }
    }
    Ok(())
}
