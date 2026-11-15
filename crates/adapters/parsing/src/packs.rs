//! The set of installed language packs.

use std::collections::HashMap;
use std::fs;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, MutexGuard, OnceLock, PoisonError};
use std::thread;
use std::time::{Duration, Instant};

use codeskimr_application::ports::IndexerSpec;
use codeskimr_domain::LanguageId;

use crate::consts::{
    ID_MISMATCH, LOAD_PANICKED, MANIFEST_FILE, MIN_LOAD_THREADS, NAMING_FILE_IN_NAMES,
    NAMING_NAMESPACED, NAMING_REVISION, NAMING_RULE_SEPARATOR, NAMING_RULES_SEPARATOR,
};
use crate::error::ParsingError;
use crate::indexer::IndexerManifest;
use crate::manifest::{PackManifest, Tier};
use crate::pack::LanguagePack;

/// Runs `work` for every index below `count` on at most one thread per core, so that the number of
/// threads does not grow with the number of installed packs. The result keeps the order of the
/// items; if a worker panicked and took its results with it, the last entry is `lost`.
fn run_parallel<I: Sync, T: Send>(
    items: &[I],
    work: impl Fn(&I) -> T + Sync,
    lost: impl FnOnce() -> T,
) -> Vec<T> {
    let count = items.len();
    let next = AtomicUsize::new(0);
    let take = || {
        let mut done = Vec::new();
        loop {
            let index = next.fetch_add(1, Ordering::Relaxed);
            let Some(item) = items.get(index) else {
                return done;
            };
            done.push((index, work(item)));
        }
    };
    let mut done: Vec<(usize, T)> = thread::scope(|scope| {
        let handles: Vec<_> = (0..load_workers(count))
            .map(|_| scope.spawn(take))
            .collect();
        handles
            .into_iter()
            .flat_map(|handle| handle.join().unwrap_or_default())
            .collect()
    });
    done.sort_by_key(|(index, _)| *index);
    let mut results: Vec<T> = done.into_iter().map(|(_, result)| result).collect();
    if results.len() < count {
        results.push(lost());
    }
    results
}

/// The error for a worker that panicked while loading packs.
fn panicked() -> ParsingError {
    ParsingError::invalid(String::new(), LOAD_PANICKED.to_owned())
}

/// Threads that load `packs` packs: one per core at most, and no more than there are packs.
fn load_workers(packs: usize) -> usize {
    thread::available_parallelism()
        .map_or(MIN_LOAD_THREADS, NonZeroUsize::get)
        .min(packs.max(MIN_LOAD_THREADS))
}

/// The time during which at least one pack was being loaded (ADR 0059): packs that load at the same
/// time count once, so the total never exceeds the time that passed.
#[derive(Default)]
struct LoadClock {
    state: Mutex<LoadState>,
}

#[derive(Default)]
struct LoadState {
    running: usize,
    since: Option<Instant>,
    total: Duration,
}

/// Counts one running load; the time is added when it is dropped.
struct LoadTimer<'a>(&'a LoadClock);

impl LoadClock {
    fn state(&self) -> MutexGuard<'_, LoadState> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn running(&self) -> LoadTimer<'_> {
        let mut state = self.state();
        if state.running == 0 {
            state.since = Some(Instant::now());
        }
        state.running += 1;
        drop(state);
        LoadTimer(self)
    }

    /// The loading time so far, including a load that is still running.
    fn total(&self) -> Duration {
        let state = self.state();
        state.total + state.since.map_or(Duration::ZERO, |since| since.elapsed())
    }
}

impl Drop for LoadTimer<'_> {
    fn drop(&mut self) {
        let mut state = self.0.state();
        state.running -= 1;
        if state.running == 0
            && let Some(since) = state.since.take()
        {
            state.total += since.elapsed();
        }
    }
}

/// What is known about a pack without loading its grammar and queries: enough to route files to
/// languages and to show the tier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackSummary {
    id: LanguageId,
    tier: Tier,
    extensions: Vec<String>,
    file_in_names: bool,
    indexer: Option<IndexerSpec>,
}

impl PackSummary {
    /// Id of the language.
    #[must_use]
    pub const fn id(&self) -> &LanguageId {
        &self.id
    }

    /// Tier of the pack.
    #[must_use]
    pub const fn tier(&self) -> Tier {
        self.tier
    }

    /// How level 2 runs the indexer of this language, if the pack names one.
    #[must_use]
    pub const fn indexer(&self) -> Option<&IndexerSpec> {
        self.indexer.as_ref()
    }

    /// Whether the path of the file starts every qualified name of the pack (ADR 0036).
    #[must_use]
    pub const fn file_in_names(&self) -> bool {
        self.file_in_names
    }

    /// File extensions the pack claims, without the dot.
    #[must_use]
    pub fn extensions(&self) -> &[String] {
        &self.extensions
    }
}

/// One installed pack. A pack is loaded when a file of its language is first analyzed (ADR 0059):
/// reading and compiling the queries of a full-tier pack takes long and most scans use one or two
/// languages.
struct Slot {
    summary: PackSummary,
    dir: PathBuf,
    /// The pack, or why it could not be loaded (folder, reason), kept so that the other files of the
    /// language fail at once instead of building the pack again (issue #66). The first thread that
    /// needs the pack builds it; the others wait for the result instead of building it again.
    loaded: OnceLock<Result<LanguagePack, (String, String)>>,
}

impl Slot {
    fn read(dir: PathBuf) -> Result<Self, ParsingError> {
        let manifest = PackManifest::read(&dir)?;
        if manifest.id != folder(&dir) {
            let reason = [ID_MISMATCH, manifest.id.as_str()].concat();
            return Err(ParsingError::invalid(folder(&dir), reason));
        }
        let indexer = manifest
            .indexer
            .as_ref()
            .map(IndexerManifest::spec)
            .transpose()
            .map_err(|reason| ParsingError::invalid(folder(&dir), reason))?;
        let summary = PackSummary {
            id: LanguageId::new(manifest.id)
                .map_err(|e| ParsingError::invalid(folder(&dir), e.to_string()))?,
            tier: manifest.tier,
            extensions: manifest.extensions,
            file_in_names: manifest.file_in_names,
            indexer,
        };
        Ok(Self {
            summary,
            dir,
            loaded: OnceLock::new(),
        })
    }

    fn pack(&self, clock: &LoadClock) -> Result<&LanguagePack, ParsingError> {
        let loaded = self.loaded.get_or_init(|| {
            let _timer = clock.running();
            // `LanguagePack::load` reports only `PackInvalid`; any other error is kept as text so
            // that it is not lost and does not make the pack load again.
            match LanguagePack::load(&self.dir) {
                Ok(pack) => Ok(pack),
                Err(ParsingError::PackInvalid { pack, reason }) => Err((pack, reason)),
                Err(other) => Err((folder(&self.dir), other.to_string())),
            }
        });
        loaded
            .as_ref()
            .map_err(|(pack, reason)| ParsingError::invalid(pack.clone(), reason.clone()))
    }
}

fn folder(dir: &Path) -> String {
    dir.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// The folders below `root` that hold a `language.toml`, in order. `LanguagePacks::load` and
/// `tiers` use this one search, so the tier shown is that of the pack the scan loads.
fn pack_dirs(root: &Path) -> Result<Vec<PathBuf>, ParsingError> {
    let mut dirs = Vec::new();
    for entry in fs::read_dir(root).map_err(ParsingError::Unreadable)? {
        let dir = entry.map_err(ParsingError::Unreadable)?.path();
        if dir.join(MANIFEST_FILE).is_file() {
            dirs.push(dir);
        }
    }
    dirs.sort();
    Ok(dirs)
}

/// The tier of every pack below `root`, read from the manifests alone: no grammar and no query is
/// compiled, so this is cheap enough for a screen that asks often. A pack whose manifest cannot
/// be read is left out; the scan reports it when it loads the packs.
///
/// # Errors
/// [`ParsingError::Unreadable`] if `root` cannot be listed.
pub fn tiers(root: &Path) -> Result<Vec<(String, Tier)>, ParsingError> {
    Ok(pack_dirs(root)?
        .iter()
        .filter_map(|dir| PackManifest::read(dir).ok())
        .map(|manifest| (manifest.id, manifest.tier))
        .collect())
}

/// All language packs found below one folder, by language id.
pub struct LanguagePacks {
    slots: HashMap<LanguageId, Slot>,
    clock: LoadClock,
}

impl LanguagePacks {
    /// Reads the `language.toml` of every sub folder of `root` that holds one. The grammar and the
    /// queries of a pack are compiled when its language is first needed (ADR 0059); `verify` loads
    /// all of them at once.
    ///
    /// # Errors
    /// [`ParsingError::Unreadable`] if `root` cannot be listed, or the error of the first pack that
    /// is invalid.
    pub fn load(root: &Path) -> Result<Self, ParsingError> {
        let dirs = pack_dirs(root)?;
        // The error reported is the one of the first pack in the order of the folders.
        let read = run_parallel(&dirs, |dir| Slot::read(dir.clone()), || Err(panicked()));
        let mut slots = HashMap::new();
        for slot in read {
            let slot = slot?;
            slots.insert(slot.summary.id.clone(), slot);
        }
        Ok(Self {
            slots,
            clock: LoadClock::default(),
        })
    }

    /// All installed packs, in no particular order. Does not load any grammar.
    pub fn iter(&self) -> impl Iterator<Item = &PackSummary> {
        self.slots.values().map(|slot| &slot.summary)
    }

    /// The naming rules of all packs as one text, sorted by language: it changes whenever a rule
    /// that changes the qualified names does (ADR 0036).
    #[must_use]
    pub fn naming_rules(&self) -> String {
        let mut rules: Vec<String> = self
            .iter()
            .map(|pack| {
                [
                    pack.id().as_str(),
                    NAMING_RULE_SEPARATOR,
                    if pack.file_in_names() {
                        NAMING_FILE_IN_NAMES
                    } else {
                        NAMING_NAMESPACED
                    },
                ]
                .concat()
            })
            .collect();
        rules.push(NAMING_REVISION.to_owned());
        rules.sort();
        rules.join(NAMING_RULES_SEPARATOR)
    }

    /// Loads every installed pack now, on at most one thread per core, so that a broken pack is
    /// found at once instead of when a file of its language is analyzed. For tests and checks of
    /// the packs; a scan does not need it.
    ///
    /// # Errors
    /// The error of the first pack, in the order of the language ids, that cannot be loaded.
    pub fn verify(&self) -> Result<(), ParsingError> {
        let mut slots: Vec<&Slot> = self.slots.values().collect();
        slots.sort_by(|a, b| a.summary.id.as_str().cmp(b.summary.id.as_str()));
        run_parallel(
            &slots,
            |slot| slot.pack(&self.clock).map(drop),
            || Err(panicked()),
        )
        .into_iter()
        .try_for_each(|result| result)
    }

    /// The pack for `language`; it is loaded by the first call.
    ///
    /// # Errors
    /// [`ParsingError::PackMissing`] if none is installed, [`ParsingError::PackInvalid`] if a
    /// basic-tier pack turns out to be invalid when it is loaded.
    pub fn get(&self, language: &LanguageId) -> Result<&LanguagePack, ParsingError> {
        self.slots
            .get(language)
            .ok_or_else(|| ParsingError::PackMissing(language.to_string()))?
            .pack(&self.clock)
    }

    /// How long loading packs has taken so far: the time during which at least one pack was being
    /// loaded. A scan leaves this out of its reported time (ADR 0059).
    #[must_use]
    pub fn load_time(&self) -> Duration {
        self.clock.total()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loading_never_uses_more_threads_than_packs_or_cores() {
        let cores = std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get);
        assert_eq!(load_workers(1), 1);
        assert_eq!(load_workers(0), 1);
        assert_eq!(load_workers(10_000), cores);
    }

    #[test]
    fn a_worker_that_panics_leaves_an_error_and_not_a_success() {
        let items = [1_u8, 2, 3];
        let results = run_parallel(
            &items,
            |item| {
                assert_ne!(*item, 2, "this worker fails");
                Ok(())
            },
            || Err(panicked()),
        );
        assert!(results.into_iter().any(|result| result.is_err()));
    }

    #[test]
    fn loads_that_overlap_count_once() {
        let clock = LoadClock::default();
        let start = Instant::now();
        let (first, second) = (clock.running(), clock.running());
        thread::sleep(Duration::from_millis(20));
        drop((first, second));
        let wall = start.elapsed();
        assert!(clock.total() >= Duration::from_millis(20));
        assert!(clock.total() <= wall);
    }
}
