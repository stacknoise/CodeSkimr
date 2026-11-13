//! The six concrete stages, each built from the ports it needs, and the values passed between them.

use std::sync::Arc;

use codeskimr_domain::{
    CommitHash, FolderPath, Metric, Project, Reference, ScanMode, Snapshot, SnapshotId,
    SnapshotStats, SourceFile, Symbol, SymbolCounts,
};

use super::carry::Carried;
use super::flow::{Interruption, Stage, StageContext};
use crate::ids::IdSequence;
use crate::ports::{
    AnalyzeRequest, Clock, DiscoveredFile, FileAnalyzer, FileDiscovery, MetricsCalculator,
    PendingReference, ProjectRepository, ReferenceResolver, SnapshotData, SnapshotStore,
    SourceProvider, SymbolFingerprints,
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
        let total = count(&due);
        let mut out = Analyzed {
            run: run.clone(),
            files: Vec::new(),
            symbols: Vec::new(),
            pending: Vec::new(),
            counts: Vec::new(),
            fingerprints: Vec::new(),
            carried,
        };
        for (done, file) in (1..).zip(due) {
            if ctx.is_cancelled() {
                return Err(Interruption::Cancelled);
            }
            let request = AnalyzeRequest {
                root: &run.root,
                snapshot: run.snapshot,
                file,
                ids: &self.ids,
            };
            let facts = self.analyzer.analyze(&request)?;
            out.files.push(facts.file);
            out.symbols.extend(facts.symbols);
            out.pending.extend(facts.pending);
            out.counts.extend(facts.counts);
            out.fingerprints.extend(facts.fingerprints);
            ctx.progress(done, total);
        }
        Ok(out)
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

impl Stage for PersistStage {
    type Input = Measured;
    type Output = Snapshot;

    fn kind(&self) -> StageKind {
        StageKind::Persist
    }

    fn run(&self, input: Measured, ctx: &StageContext<'_>) -> Result<Snapshot, Interruption> {
        ctx.progress(0, 1);
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
        Ok(stored)
    }
}
