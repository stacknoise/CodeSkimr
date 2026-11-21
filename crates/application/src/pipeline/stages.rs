//! The six concrete stages, each built from the ports it needs, and the values passed between them.

use std::collections::{BTreeMap, HashSet};
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread;
use std::time::Instant;

use codeskimr_domain::{
    CommitHash, FileId, FolderPath, LanguageId, Metric, Project, Reference, RelativePath, ScanMode,
    Snapshot, SnapshotId, SnapshotStats, SourceFile, Symbol, SymbolCounts, SymbolOrigin, Timestamp,
};

use codeskimr_codes::{Coded, ErrorCode};

use super::carry::Carried;
use super::flow::{Interruption, Stage, StageContext};
use crate::consts::{
    DONE_STEP, FILE_ID_BLOCK, FIRST_INDEX, INDEX_STEP, LEFT_OUT_STEP, LOG_FILE_SKIPPED,
    LOG_SCAN_WARNING, LOG_VERIFIED_DISCARDED, MIN_WORKERS, NONE_DONE,
};
use crate::error::CodedError;
use crate::ids::IdSequence;
use crate::ports::{
    AnalyzeRequest, Clock, DiscoveredFile, FileAnalyzer, FileDiscovery, FileFacts, LeftOut,
    MetricsCalculator, PathsReport, PendingReference, PreviousState, ProjectRepository,
    ReferenceResolver, ScanWarning, SnapshotData, SnapshotStore, SourceProvider, StoredNames,
    SymbolFingerprints, SymbolHash,
};
use crate::resolve::ambiguous_occurrences;
use crate::selection::Selection;
use crate::stage::StageKind;

/// What starts a scan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanRequest {
    /// The project to scan.
    pub project: Project,
    /// Full or incremental.
    pub mode: ScanMode,
    /// The paths an incremental scan looks at; empty means all.
    pub paths: Selection,
}

/// A scan whose source is available locally; carried through all later stages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanRun {
    /// The project being scanned.
    pub project: Project,
    /// Full or incremental.
    pub mode: ScanMode,
    /// The paths an incremental scan looks at; empty means all.
    pub paths: Selection,
    /// What became of the paths, set by the discover stage.
    pub paths_report: PathsReport,
    /// The snapshot this scan will create.
    pub snapshot: SnapshotId,
    /// Local folder with the files.
    pub root: FolderPath,
    /// Checked out commit, if any.
    pub commit: Option<CommitHash>,
    /// The naming rules of the packs, set by the discover stage (ADR 0036).
    pub naming_rules: String,
    /// Time spent analysing (parse, symbols, metrics, fingerprints), summed over the files of each
    /// language in microseconds; set by the analyze stage, empty in an incremental scan.
    pub parse_micros: Vec<(LanguageId, u64)>,
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
    /// The content hashes of the types and members of the analysed files.
    pub hashes: Vec<SymbolHash>,
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
    /// The references to write: those of the analysed files and the new ones of unchanged files.
    pub references: Vec<Reference>,
    /// The names of the analysed files, to be stored.
    pub pending: Vec<PendingReference>,
    /// The counts behind the metrics of the analysed files.
    pub counts: Vec<SymbolCounts>,
    /// The syntax fingerprints of the callables of the analysed files.
    pub fingerprints: Vec<SymbolFingerprints>,
    /// The content hashes of the types and members of the analysed files.
    pub hashes: Vec<SymbolHash>,
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
    /// The references to write.
    pub references: Vec<Reference>,
    /// The names of the analysed files, to be stored.
    pub pending: Vec<PendingReference>,
    /// Measured values.
    pub metrics: Vec<Metric>,
    /// The syntax fingerprints of the callables of the analysed files.
    pub fingerprints: Vec<SymbolFingerprints>,
    /// The content hashes of the types and members of the analysed files.
    pub hashes: Vec<SymbolHash>,
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
            paths: input.paths,
            paths_report: PathsReport::default(),
            snapshot: self.ids.snapshot(),
            root: fetched.root,
            commit: fetched.commit,
            naming_rules: String::new(),
            parse_micros: Vec::new(),
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
    /// Tells the naming rules of the packs.
    pub analyzer: Arc<dyn FileAnalyzer>,
}

impl DiscoverStage {
    /// The run with the naming rules of the packs. An incremental scan after other rules, or after
    /// a snapshot without a record, is a full scan: the kept files would carry the old names.
    fn with_naming(&self, mut run: ScanRun) -> Result<ScanRun, CodedError> {
        run.naming_rules = self.analyzer.naming_rules();
        if run.mode == ScanMode::Incremental {
            let kept = self.projects.latest_naming_rules(run.project.id)?;
            if kept.unwrap_or_default() != run.naming_rules {
                run.mode = ScanMode::Full;
            }
        }
        Ok(run)
    }
}

impl Stage for DiscoverStage {
    type Input = ScanRun;
    type Output = Discovered;

    fn kind(&self) -> StageKind {
        StageKind::Discover
    }

    fn run(&self, run: ScanRun, ctx: &StageContext<'_>) -> Result<Discovered, Interruption> {
        let mut run = self.with_naming(run)?;
        let state = self.projects.latest_state(run.project.id)?;
        if !state.names_stored {
            // The newest snapshot does not hold the names as written, so the references of its
            // files cannot be resolved again: a full scan writes them.
            run.mode = ScanMode::Full;
        }
        let mut files = self
            .discovery
            .discover(&run.root, &state.files, ctx.cancel_token())?;
        run.paths_report = report_paths(run.mode, &files, &state.files, &run.paths);
        if run.mode == ScanMode::Incremental {
            limit_to_paths(&mut files, &state.files, &run.paths);
        }
        let carried = if let (ScanMode::Incremental, Some(snapshot)) = (run.mode, state.snapshot) {
            let bases = self.projects.stored_names(snapshot, StoredNames::Bases)?;
            Carried::from_previous(&mut files, state, bases)
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

/// What becomes of `paths`: those that name no file now and in the previous snapshot, and whether
/// a full scan makes the scan ignore them.
fn report_paths(
    mode: ScanMode,
    files: &[DiscoveredFile],
    previous: &[SourceFile],
    paths: &Selection,
) -> PathsReport {
    let known = files
        .iter()
        .map(|file| &file.path)
        .chain(previous.iter().map(|file| &file.path));
    PathsReport {
        unmatched: paths.unmatched_in(known),
        ignored: !paths.is_empty() && mode == ScanMode::Full,
        left_out: Vec::new(),
        left_out_more: u64::MIN,
    }
}

/// Limits an incremental scan to `paths`: what lies elsewhere is taken over from `previous`
/// unchanged, a new file there is not looked at and a file that is gone there is not removed.
fn limit_to_paths(files: &mut Vec<DiscoveredFile>, previous: &[SourceFile], paths: &Selection) {
    if paths.is_empty() {
        return;
    }
    files.retain(|file| paths.contains(&file.path));
    files.extend(
        previous
            .iter()
            .filter(|file| !paths.contains(&file.path))
            .map(|file| DiscoveredFile {
                path: file.path.clone(),
                language: file.language.clone(),
                content_hash: file.content_hash.clone(),
                changed: false,
            }),
    );
}

/// Analyze: parse and extract, file by file, skipping unchanged files in incremental scans.
pub struct AnalyzeStage {
    /// The parsing adapter.
    pub analyzer: Arc<dyn FileAnalyzer>,
    /// Id allocation for files and symbols.
    pub ids: Arc<IdSequence>,
    /// Most threads that analyse at the same time; 0 means one per core.
    pub max_workers: u32,
    /// Source of the previous snapshot, read again when a file that could not be analysed keeps
    /// its former state.
    pub projects: Arc<dyn ProjectRepository>,
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
            mut files,
            carried,
        } = input;
        let full = run.mode == ScanMode::Full;
        let outcomes = {
            let due: Vec<&DiscoveredFile> =
                files.iter().filter(|file| file.changed || full).collect();
            self.analyze_all(&run, &due, ctx)?
        };
        let mut out = Analyzed::empty(run, carried);
        let mut times: BTreeMap<LanguageId, u64> = BTreeMap::new();
        for outcome in outcomes {
            match outcome {
                Outcome::Done(facts, micros) => {
                    *times.entry(facts.file.language.clone()).or_default() += micros;
                    out.take(*facts);
                }
                Outcome::Skipped(left_out) => out.run.paths_report.left_out.push(left_out),
            }
        }
        if full {
            out.run.parse_micros = times.into_iter().collect();
        }
        self.settle_left_out(&mut out, &mut files)?;
        Ok(out)
    }
}

impl Analyzed {
    /// The result of a stage that has analysed nothing yet.
    fn empty(run: ScanRun, carried: Carried) -> Self {
        Self {
            run,
            files: Vec::new(),
            symbols: Vec::new(),
            pending: Vec::new(),
            counts: Vec::new(),
            fingerprints: Vec::new(),
            hashes: Vec::new(),
            carried,
        }
    }

    /// Adds the facts of one analysed file.
    fn take(&mut self, facts: FileFacts) {
        self.files.push(facts.file);
        self.symbols.extend(facts.symbols);
        self.pending.extend(facts.pending);
        self.counts.extend(facts.counts);
        self.fingerprints.extend(facts.fingerprints);
        self.hashes.extend(facts.hashes);
    }
}

/// Whether a failure with `code` concerns the one file only. Such a file is left out and the scan
/// goes on; any other failure (a broken language pack, a bug) stops the scan.
fn leaves_file_out(code: ErrorCode) -> bool {
    LeftOut::CODES.contains(&code)
}

/// The outcome for a file that is left out, with a log line that names it.
fn skipped(file: &DiscoveredFile, code: ErrorCode) -> Outcome {
    tracing::warn!(
        event = LOG_FILE_SKIPPED,
        code = code.id(),
        path = file.path.as_str()
    );
    Outcome::Skipped(LeftOut {
        path: file.path.clone(),
        code,
        kept: false,
    })
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
type Slot = (usize, Result<Outcome, Interruption>);

/// What became of one file in the analyze stage.
enum Outcome {
    /// The facts of the file with the microseconds its analysis took.
    Done(Box<FileFacts>, u64),
    /// The file could not be analysed and is left out.
    Skipped(LeftOut),
}

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
    /// Decides what the snapshot shows of the files that were left out, and fails the scan if it
    /// would show no file at all: a project in which nothing can be analysed (every file in an
    /// encoding that is not read, say) must not look like a scan that worked.
    fn settle_left_out(
        &self,
        out: &mut Analyzed,
        files: &mut [DiscoveredFile],
    ) -> Result<(), CodedError> {
        let Some(first) = out.run.paths_report.left_out.first().map(|left| left.code) else {
            return Ok(());
        };
        self.keep_previous(out, files)?;
        let kept: HashSet<RelativePath> = out
            .carried
            .files
            .iter()
            .map(|file| file.path.clone())
            .collect();
        for left in &mut out.run.paths_report.left_out {
            left.kept = kept.contains(&left.path);
            if !left.kept {
                // An incremental scan must not carry the stale row of the file over.
                out.carried.removed.push(left.path.clone());
            }
        }
        if out.files.is_empty() && kept.is_empty() {
            return Err(CodedError::new(first));
        }
        Ok(())
    }

    /// Gives the files whose failure may pass (a busy machine, a locked file) their former state
    /// back, so that their symbols, references, notes and critical marks do not vanish for one
    /// scan. The previous snapshot is split again with them as unchanged. Their stored content
    /// hash stays the old one, so the next scan tries them again.
    ///
    /// A full scan goes on as an incremental one in which every other file changed, which gives
    /// what a full scan gives for those and the former state for these (see [`Carried`]). It
    /// cannot when the previous snapshot holds other naming rules or no names: then the files
    /// are left out.
    fn keep_previous(
        &self,
        out: &mut Analyzed,
        files: &mut [DiscoveredFile],
    ) -> Result<(), CodedError> {
        let may_pass: HashSet<&RelativePath> = out
            .run
            .paths_report
            .left_out
            .iter()
            .filter(|left| LeftOut::MAY_PASS.contains(&left.code))
            .map(|left| &left.path)
            .collect();
        if may_pass.is_empty() {
            return Ok(());
        }
        let state = self.projects.latest_state(out.run.project.id)?;
        let Some(snapshot) = state.snapshot else {
            return Ok(());
        };
        if out.run.mode == ScanMode::Full && !self.can_follow(&out.run, &state)? {
            return Ok(());
        }
        let full = out.run.mode == ScanMode::Full;
        for file in files.iter_mut() {
            if may_pass.contains(&file.path) {
                file.changed = false;
            } else if full {
                file.changed = true;
            }
        }
        let bases = self.projects.stored_names(snapshot, StoredNames::Bases)?;
        out.carried = Carried::from_previous(files, state, bases);
        if full {
            // The times of a scan that missed a file are not those of a full scan.
            out.run.parse_micros.clear();
            out.run.mode = ScanMode::Incremental;
        }
        Ok(())
    }

    /// Whether a full scan can build on the previous snapshot: it holds the names as written, and
    /// they were made under the naming rules of this run (the same test the discover stage makes
    /// for an incremental scan).
    fn can_follow(&self, run: &ScanRun, state: &PreviousState) -> Result<bool, CodedError> {
        if !state.names_stored {
            return Ok(false);
        }
        let kept = self.projects.latest_naming_rules(run.project.id)?;
        Ok(kept.unwrap_or_default() == run.naming_rules)
    }

    /// Analyses the files on all cores. The result keeps the order of `due`; the first failure
    /// stops the other workers.
    fn analyze_all(
        &self,
        run: &ScanRun,
        due: &[&DiscoveredFile],
        ctx: &StageContext<'_>,
    ) -> Result<Vec<Outcome>, Interruption> {
        let shared = Shared {
            next: AtomicUsize::new(FIRST_INDEX),
            done: Mutex::new(NONE_DONE),
            first_id: self.ids.reserve(count_u64(due) * FILE_ID_BLOCK),
            stopped: AtomicBool::new(false),
        };
        let workers = analyze_workers(self.max_workers);
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
    ) -> Result<Outcome, Interruption> {
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
        let started = Instant::now();
        let facts = match self.analyzer.analyze(&request) {
            Ok(facts) => facts,
            Err(cause) if leaves_file_out(cause.code()) => return Ok(skipped(file, cause.code())),
            Err(cause) => return Err(cause.into()),
        };
        let micros = u64::try_from(started.elapsed().as_micros()).unwrap_or(u64::MAX);
        if ids.peek() - first_id > FILE_ID_BLOCK {
            return Err(internal_failure());
        }
        Ok(Outcome::Done(Box::new(facts), micros))
    }
}

/// Resolve: map pending names to symbols after the global symbol table is complete.
pub struct ResolveStage {
    /// Source of the stored names of the previous snapshot.
    pub projects: Arc<dyn ProjectRepository>,
    /// The resolver strategy.
    pub resolver: Arc<dyn ReferenceResolver>,
    /// Tells which files are test files.
    pub analyzer: Arc<dyn FileAnalyzer>,
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
            pending,
            counts,
            fingerprints,
            hashes,
            mut carried,
        } = input;
        let due = carried.due_names((&symbols, &pending), self.projects.as_ref())?;
        let names: Vec<PendingReference> = pending.iter().chain(&due).cloned().collect();
        let total = count(&names);
        ctx.progress(0, total);
        let table = level_one_symbols(&carried.symbols, &symbols);
        let test_files = self.test_files(files.iter().chain(&carried.files));
        let resolved = self
            .resolver
            .resolve(&table, &names, &test_files, &self.ids)?;
        let references = carried.reconcile(resolved);
        if carried.discarded_verified > 0 {
            tracing::info!(
                event = LOG_VERIFIED_DISCARDED,
                count = carried.discarded_verified
            );
        }
        ctx.progress(total, total);
        Ok(Resolved {
            run,
            files,
            symbols,
            references,
            pending,
            counts,
            fingerprints,
            hashes,
            carried,
        })
    }
}

impl ResolveStage {
    /// Ids of the files among `files` that are test files of their language.
    fn test_files<'a>(&self, files: impl Iterator<Item = &'a SourceFile>) -> HashSet<FileId> {
        files
            .filter(|file| self.analyzer.is_test_file(&file.language, &file.path))
            .map(|file| file.id)
            .collect()
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
            pending: input.pending,
            metrics,
            fingerprints: input.fingerprints,
            hashes: input.hashes,
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
    /// Most files the result names when they were left out; the others are counted.
    pub max_left_out_listed: u32,
}

/// Output of the persist stage.
#[derive(Debug, Clone, PartialEq)]
pub struct Persisted {
    /// The snapshot as stored.
    pub snapshot: Snapshot,
    /// Local folder with the files, which level 2 runs its indexers in.
    pub root: FolderPath,
    /// What the scan wants to tell besides its result: one entry per code, with a count.
    pub warnings: Vec<ScanWarning>,
    /// What became of the paths of the scan.
    pub paths: PathsReport,
}

/// Writes one log line per warning of the scan.
fn log_warnings(warnings: &[ScanWarning]) {
    for warning in warnings {
        tracing::warn!(
            event = LOG_SCAN_WARNING,
            code = warning.code.id(),
            count = warning.count
        );
    }
}

/// The warnings of a scan over all references of the snapshot, changed and kept ones.
fn warnings_of(references: &[Reference], carried: &[Reference]) -> Vec<ScanWarning> {
    let ambiguous = ambiguous_occurrences(references.iter().chain(carried));
    let found = [(ErrorCode::AmbiguousReference, ambiguous)];
    found
        .into_iter()
        .filter(|(_, count)| *count > 0)
        .map(|(code, count)| ScanWarning { code, count })
        .collect()
}

/// The report with at most `most` files that were left out by name; the others are counted.
fn listed(mut paths: PathsReport, most: u32) -> PathsReport {
    let most = usize::try_from(most).unwrap_or(usize::MAX);
    let more = paths.left_out.len().saturating_sub(most);
    paths.left_out.truncate(most);
    paths.left_out_more = u64::try_from(more).unwrap_or(u64::MAX);
    paths
}

/// One warning per code of the files that were left out, with the number of files, in the order
/// the codes first came up.
fn left_out_warnings(left_out: &[LeftOut]) -> Vec<ScanWarning> {
    let mut warnings: Vec<ScanWarning> = Vec::new();
    for left in left_out {
        match warnings.iter_mut().find(|known| known.code == left.code) {
            Some(known) => known.count += LEFT_OUT_STEP,
            None => warnings.push(ScanWarning {
                code: left.code,
                count: LEFT_OUT_STEP,
            }),
        }
    }
    warnings
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
        let snapshot = snapshot_of(&input, self.clock.now());
        let mut warnings = warnings_of(&input.references, &input.carried.references);
        warnings.extend(left_out_warnings(&input.run.paths_report.left_out));
        let data = SnapshotData {
            snapshot,
            files: input.files,
            symbols: input.symbols,
            references: input.references,
            metrics: input.metrics,
            fingerprints: input.fingerprints,
            hashes: input.hashes,
            removed: input.carried.removed,
            pending: input.pending,
            dropped_references: input.carried.dropped,
            naming_rules: input.run.naming_rules,
            parse_micros: input.run.parse_micros,
            warnings: warnings.clone(),
        };
        let stored = self.store.persist(data)?;
        log_warnings(&warnings);
        ctx.progress(1, 1);
        Ok(Persisted {
            snapshot: stored,
            root,
            warnings,
            paths: listed(input.run.paths_report, self.max_left_out_listed),
        })
    }
}

/// The snapshot row of a run: who it belongs to and how many files, symbols and references it holds.
fn snapshot_of(input: &Measured, created: Timestamp) -> Snapshot {
    Snapshot {
        id: input.run.snapshot,
        project: input.run.project.id,
        commit: input.run.commit.clone(),
        created,
        mode: input.run.mode,
        stats: SnapshotStats {
            files: count(&input.files) + count(&input.carried.files),
            symbols: count(&input.symbols) + count(&input.carried.symbols),
            references: count(&input.references) + count(&input.carried.references),
        },
    }
}

/// How many threads the analyze stage uses: one per core the system offers, at least one, and
/// no more than `max_workers` (`scan.max_workers`) when that is not 0.
#[must_use]
pub fn analyze_workers(max_workers: u32) -> usize {
    let cores = thread::available_parallelism().map_or(MIN_WORKERS, NonZeroUsize::get);
    match usize::try_from(max_workers) {
        Ok(limit) if limit > 0 => cores.min(limit),
        _ => cores,
    }
}

/// The symbols the resolver looks names up in. Symbols that level 2 created are not names level 1
/// resolves to (ADR 0055, decision 5): the result of a scan must not depend on whether level 2 ran
/// before.
fn level_one_symbols(carried: &[Symbol], fresh: &[Symbol]) -> Vec<Symbol> {
    carried
        .iter()
        .chain(fresh)
        .filter(|symbol| symbol.origin == SymbolOrigin::Syntax)
        .cloned()
        .collect()
}
