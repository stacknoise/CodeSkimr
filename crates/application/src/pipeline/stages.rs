//! The six concrete stages, each built from the ports it needs, and the values passed between them.

use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread;

use codeskimr_domain::{
    CommitHash, FolderPath, Metric, Project, Reference, ScanMode, Snapshot, SnapshotId,
    SnapshotStats, SourceFile, Symbol, SymbolCounts,
};

use codeskimr_codes::ErrorCode;

use super::carry::Carried;
use super::flow::{Interruption, Stage, StageContext};
use crate::consts::{DONE_STEP, FILE_ID_BLOCK, FIRST_INDEX, INDEX_STEP, MIN_WORKERS, NONE_DONE};
use crate::error::CodedError;
use crate::ids::IdSequence;
use crate::ports::{
    AnalyzeRequest, Clock, DiscoveredFile, FileAnalyzer, FileDiscovery, FileFacts,
    MetricsCalculator, PendingReference, ProjectRepository, ReferenceResolver, SnapshotData,
    SnapshotStore, SourceProvider, SymbolFingerprints,
};
use crate::stage::StageKind;

/// What starts a scan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanRequest {
    /// The project to scan.
    pub project: Project,
    /// Full or incremental.
    pub mode: ScanMode,
}

/// A scan whose source is available locally; carried through all later stages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanRun {
    /// The project being scanned.
    pub project: Project,
    /// Full or incremental.
    pub mode: ScanMode,
    /// The snapshot this scan will create.
    pub snapshot: SnapshotId,
    /// Local folder with the files.
    pub root: FolderPath,
    /// Checked out commit, if any.
    pub commit: Option<CommitHash>,
}

/// Output of the discover stage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Discovered {
    /// The scan.
    pub run: ScanRun,
    /// All files found.
    pub files: Vec<DiscoveredFile>,
    /// What an incremental scan keeps of the previous snapshot.
    pub carried: Carried,
}

/// Output of the analyze stage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Analyzed {
    /// The scan.
    pub run: ScanRun,
    /// Files that were analysed.
    pub files: Vec<SourceFile>,
    /// Their symbols.
    pub symbols: Vec<Symbol>,
    /// Names still to resolve.
    pub pending: Vec<PendingReference>,
    /// The counts behind the metrics of the analysed files.
    pub counts: Vec<SymbolCounts>,
    /// The syntax fingerprints of the callables of the analysed files.
    pub fingerprints: Vec<SymbolFingerprints>,
    /// What an incremental scan keeps of the previous snapshot.
    pub carried: Carried,
}

/// Output of the resolve stage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolved {
    /// The scan.
    pub run: ScanRun,
    /// Files that were analysed.
    pub files: Vec<SourceFile>,
    /// Their symbols.
    pub symbols: Vec<Symbol>,
    /// Resolved references, including those of unchanged files that were resolved again.
    pub references: Vec<Reference>,
    /// The counts behind the metrics of the analysed files.
    pub counts: Vec<SymbolCounts>,
    /// The syntax fingerprints of the callables of the analysed files.
    pub fingerprints: Vec<SymbolFingerprints>,
    /// What an incremental scan keeps of the previous snapshot.
    pub carried: Carried,
}

/// Output of the measure stage.
#[derive(Debug, Clone, PartialEq)]
pub struct Measured {
    /// The scan.
    pub run: ScanRun,
    /// Files that were analysed.
    pub files: Vec<SourceFile>,
    /// Their symbols.
    pub symbols: Vec<Symbol>,
    /// Resolved references.
    pub references: Vec<Reference>,
    /// Measured values.
    pub metrics: Vec<Metric>,
    /// The syntax fingerprints of the callables of the analysed files.
    pub fingerprints: Vec<SymbolFingerprints>,
    /// What an incremental scan keeps of the previous snapshot.
    pub carried: Carried,
}

/// Counts a list as `u32`, saturating instead of failing.
fn count<T>(items: &[T]) -> u32 {
    u32::try_from(items.len()).unwrap_or(u32::MAX)
}

/// Fetch: make the source available and reserve the snapshot id.
pub struct FetchStage {
    /// The source adapter.
    pub source: Arc<dyn SourceProvider>,
    /// Id allocation for the scan.
    pub ids: Arc<IdSequence>,
}

impl Stage for FetchStage {
    type Input = ScanRequest;
    type Output = ScanRun;

    fn kind(&self) -> StageKind {
        StageKind::Fetch
    }

    fn run(&self, input: ScanRequest, ctx: &StageContext<'_>) -> Result<ScanRun, Interruption> {
        let fetched = self
            .source
            .fetch(&input.project.source, ctx.cancel_token())?;
        Ok(ScanRun {
            project: input.project,
            mode: input.mode,
            snapshot: self.ids.snapshot(),
            root: fetched.root,
            commit: fetched.commit,
        })
        .inspect(|_| ctx.progress(1, 1))
    }
}

/// Discover: list the files and mark the changed ones.
pub struct DiscoverStage {
    /// The discovery adapter.
    pub discovery: Arc<dyn FileDiscovery>,
    /// Source of the previous snapshot's files.
    pub projects: Arc<dyn ProjectRepository>,
}

impl Stage for DiscoverStage {
    type Input = ScanRun;
    type Output = Discovered;

    fn kind(&self) -> StageKind {
        StageKind::Discover
    }

    fn run(&self, run: ScanRun, ctx: &StageContext<'_>) -> Result<Discovered, Interruption> {
        let previous = self.projects.latest_files(run.project.id)?;
        let mut files = self
            .discovery
            .discover(&run.root, &previous, ctx.cancel_token())?;
        let carried = if run.mode == ScanMode::Incremental {
            let index = self.projects.latest_index(run.project.id)?;
            Carried::from_previous(&mut files, previous, index)
        } else {
            Carried::default()
        };
        let found = count(&files);
        ctx.progress(found, found);
        Ok(Discovered {
            run,
            files,
            carried,
        })
    }
}

/// Analyze: parse and extract, file by file, skipping unchanged files in incremental scans.
pub struct AnalyzeStage {
    /// The parsing adapter.
    pub analyzer: Arc<dyn FileAnalyzer>,
    /// Id allocation for files and symbols.
    pub ids: Arc<IdSequence>,
}

impl Stage for AnalyzeStage {
    type Input = Discovered;
    type Output = Analyzed;

    fn kind(&self) -> StageKind {
        StageKind::Analyze
    }

    fn run(&self, input: Discovered, ctx: &StageContext<'_>) -> Result<Analyzed, Interruption> {
        let Discovered {
            run,
            files,
            carried,
        } = input;
        let due: Vec<&DiscoveredFile> = files
            .iter()
            .filter(|file| file.changed || run.mode == ScanMode::Full)
            .collect();
        let mut out = Analyzed {
            run: run.clone(),
            files: Vec::new(),
            symbols: Vec::new(),
            pending: Vec::new(),
            counts: Vec::new(),
            fingerprints: Vec::new(),
            carried,
        };
        for facts in self.analyze_all(&run, &due, ctx)? {
            out.files.push(facts.file);
            out.symbols.extend(facts.symbols);
            out.pending.extend(facts.pending);
            out.counts.extend(facts.counts);
            out.fingerprints.extend(facts.fingerprints);
        }
        Ok(out)
    }
}

/// The failure for a worker that panicked.
fn internal_failure() -> Interruption {
    Interruption::Failed(CodedError::new(ErrorCode::UnexpectedInternalError))
}

/// What the analysis workers share: the next file to take, the files done and a stop flag.
struct Shared {
    next: AtomicUsize,
    /// Files done so far. Progress is reported while the lock is held, so the events arrive in
    /// the order of the counts and the last one is `total` of `total`.
    done: Mutex<u32>,
    stopped: AtomicBool,
    /// First id of the block of the first file; file `n` gets the block from `first + n * BLOCK`.
    first_id: u64,
}

/// The result of one file, with the index of the file in the list of due files.
type Slot = (usize, Result<FileFacts, Interruption>);

impl Shared {
    /// First id of the block of the file at `index`.
    fn block(&self, index: usize) -> u64 {
        self.first_id + u64::try_from(index).unwrap_or_default() * FILE_ID_BLOCK
    }
}

/// Counts a list as `u64`.
fn count_u64<T>(items: &[T]) -> u64 {
    u64::try_from(items.len()).unwrap_or(u64::MAX)
}

impl AnalyzeStage {
    /// Analyses the files on all cores. The result keeps the order of `due`; the first failure
    /// stops the other workers.
    fn analyze_all(
        &self,
        run: &ScanRun,
        due: &[&DiscoveredFile],
        ctx: &StageContext<'_>,
    ) -> Result<Vec<FileFacts>, Interruption> {
        let shared = Shared {
            next: AtomicUsize::new(FIRST_INDEX),
            done: Mutex::new(NONE_DONE),
            first_id: self.ids.reserve(count_u64(due) * FILE_ID_BLOCK),
            stopped: AtomicBool::new(false),
        };
        let workers = analyze_workers();
        let mut slots: Vec<Slot> = thread::scope(|scope| {
            let handles: Vec<_> = (0..workers.min(due.len().max(MIN_WORKERS)))
                .map(|_| scope.spawn(|| self.work(run, due, &shared, ctx)))
                .collect();
            let mut all = Vec::new();
            for handle in handles {
                match handle.join() {
                    Ok(mut done) => all.append(&mut done),
                    Err(_) => all.push((usize::MAX, Err(internal_failure()))),
                }
            }
            all
        });
        slots.sort_by_key(|(index, _)| *index);
        slots.into_iter().map(|(_, result)| result).collect()
    }

    /// One worker: takes files until none is left or another worker failed.
    fn work(
        &self,
        run: &ScanRun,
        due: &[&DiscoveredFile],
        shared: &Shared,
        ctx: &StageContext<'_>,
    ) -> Vec<Slot> {
        let total = count(due);
        let mut mine = Vec::new();
        while !shared.stopped.load(Ordering::Relaxed) {
            let index = shared.next.fetch_add(INDEX_STEP, Ordering::Relaxed);
            let Some(file) = due.get(index) else { break };
            let result = self.analyze_one(run, file, shared.block(index), ctx);
            shared.stopped.fetch_or(result.is_err(), Ordering::Relaxed);
            let mut done = shared.done.lock().unwrap_or_else(PoisonError::into_inner);
            *done += DONE_STEP;
            ctx.progress(*done, total);
            drop(done);
            mine.push((index, result));
        }
        mine
    }

    /// Analyses one file with ids from its own block, so that the ids of a scan do not depend on
    /// which worker was faster.
    fn analyze_one(
        &self,
        run: &ScanRun,
        file: &DiscoveredFile,
        first_id: u64,
        ctx: &StageContext<'_>,
    ) -> Result<FileFacts, Interruption> {
        if ctx.is_cancelled() {
            return Err(Interruption::Cancelled);
        }
        let ids = IdSequence::starting_at(first_id);
        let request = AnalyzeRequest {
            root: &run.root,
            snapshot: run.snapshot,
            file,
            ids: &ids,
        };
        let facts = self.analyzer.analyze(&request)?;
        if ids.peek() - first_id > FILE_ID_BLOCK {
            return Err(internal_failure());
        }
        Ok(facts)
    }
}

/// Resolve: map pending names to symbols after the global symbol table is complete.
pub struct ResolveStage {
    /// The resolver strategy.
    pub resolver: Arc<dyn ReferenceResolver>,
    /// Id allocation for references.
    pub ids: Arc<IdSequence>,
}

impl Stage for ResolveStage {
    type Input = Analyzed;
    type Output = Resolved;

    fn kind(&self) -> StageKind {
        StageKind::Resolve
    }

    fn run(&self, input: Analyzed, ctx: &StageContext<'_>) -> Result<Resolved, Interruption> {
        let Analyzed {
            run,
            files,
            symbols,
            mut pending,
            counts,
            fingerprints,
            carried,
        } = input;
        pending.extend(carried.rebuilt.iter().cloned());
        pending.extend(carried.inheritance_seed());
        let total = count(&pending);
        ctx.progress(0, total);
        let table: Vec<Symbol> = carried.symbols.iter().chain(&symbols).cloned().collect();
        let mut references = self.resolver.resolve(&table, &pending, &self.ids)?;
        carried.drop_duplicates_of_kept(&mut references);
        ctx.progress(total, total);
        Ok(Resolved {
            run,
            files,
            symbols,
            references,
            counts,
            fingerprints,
            carried,
        })
    }
}

/// Measure: compute the metrics.
pub struct MeasureStage {
    /// The metrics strategy.
    pub calculator: Arc<dyn MetricsCalculator>,
}

impl Stage for MeasureStage {
    type Input = Resolved;
    type Output = Measured;

    fn kind(&self) -> StageKind {
        StageKind::Measure
    }

    fn run(&self, input: Resolved, ctx: &StageContext<'_>) -> Result<Measured, Interruption> {
        let symbols: Vec<Symbol> = input
            .carried
            .symbols
            .iter()
            .chain(&input.symbols)
            .cloned()
            .collect();
        let references: Vec<Reference> = input
            .carried
            .references
            .iter()
            .chain(&input.references)
            .copied()
            .collect();
        let total = count(&symbols);
        ctx.progress(0, total);
        let metrics = self
            .calculator
            .measure(&symbols, &references, &input.counts)?;
        ctx.progress(total, total);
        Ok(Measured {
            run: input.run,
            files: input.files,
            symbols: input.symbols,
            references: input.references,
            metrics,
            fingerprints: input.fingerprints,
            carried: input.carried,
        })
    }
}

/// Persist: write the snapshot. This is the only stage with a side effect outside the process,
/// and it does not start once the scan is cancelled.
pub struct PersistStage {
    /// The storage adapter.
    pub store: Arc<dyn SnapshotStore>,
    /// Source of the creation time.
    pub clock: Arc<dyn Clock>,
}

/// Output of the persist stage.
#[derive(Debug, Clone, PartialEq)]
pub struct Persisted {
    /// The snapshot as stored.
    pub snapshot: Snapshot,
    /// Local folder with the files, which level 2 runs its indexers in.
    pub root: FolderPath,
}

impl Stage for PersistStage {
    type Input = Measured;
    type Output = Persisted;

    fn kind(&self) -> StageKind {
        StageKind::Persist
    }

    fn run(&self, input: Measured, ctx: &StageContext<'_>) -> Result<Persisted, Interruption> {
        ctx.progress(0, 1);
        let root = input.run.root.clone();
        let snapshot = Snapshot {
            id: input.run.snapshot,
            project: input.run.project.id,
            commit: input.run.commit,
            created: self.clock.now(),
            mode: input.run.mode,
            stats: SnapshotStats {
                files: count(&input.files) + count(&input.carried.files),
                symbols: count(&input.symbols) + count(&input.carried.symbols),
                references: count(&input.references) + count(&input.carried.references),
            },
        };
        let data = SnapshotData {
            snapshot,
            files: input.files,
            symbols: input.symbols,
            references: input.references,
            metrics: input.metrics,
            fingerprints: input.fingerprints,
            removed: input.carried.removed,
        };
        let stored = self.store.persist(data)?;
        ctx.progress(1, 1);
        Ok(Persisted {
            snapshot: stored,
            root,
        })
    }
}

/// How many threads the analyze stage uses: one per core the system offers, at least one.
#[must_use]
pub fn analyze_workers() -> usize {
    thread::available_parallelism().map_or(MIN_WORKERS, NonZeroUsize::get)
}
