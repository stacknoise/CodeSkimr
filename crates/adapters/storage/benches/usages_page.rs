//! Benchmark of what `usages.query` asks of the store (blueprint 16.1: depth 2 in under 20 ms).
//! The graph has the size of a 100k-line project and one symbol that a share of all references
//! point at, so the walk returns thousands of entries that are cut to the limit of the screen.
//! Unlike the benchmark of the reverse index alone, this one reads the names and files of the
//! entries from the stored snapshot, as the command does.

use std::hint::black_box;
use std::num::NonZeroU32;
use std::process::ExitCode;

use codeskimr_application::ports::{SnapshotData, SnapshotStore, UsageIndex};
use codeskimr_domain::{
    Confidence, ContentHash, Direction, FileId, FolderPath, LanguageId, Loc, Position, Project,
    ProjectId, ProjectName, ProjectSource, QualifiedName, Reference, ReferenceId, ReferenceKind,
    RelativePath, ScanMode, Snapshot, SnapshotId, SnapshotStats, SourceFile, Span, Symbol,
    SymbolId, SymbolKind, SymbolName, Timestamp, UsageQuery, Visibility,
};
use codeskimr_storage::SqliteStore;
use criterion::Criterion;

const SYMBOLS: u64 = 20_000;
const REFERENCES: u64 = 100_000;
const FILES: u64 = 400;
/// One reference in this many points at the hub, the symbol that nearly everything uses.
const HUB_SHARE: u64 = 20;
const HUB: u64 = 1;
/// Number of entries the screen asks for.
const LIMIT: u32 = 500;
const SNAPSHOT: u64 = 1;
const PROJECT: u64 = 1;
/// Multipliers of the deterministic spread of the references.
const SPREAD_FROM: u64 = 7_919;
const SPREAD_TO: u64 = 104_729;
const LINES: u32 = 500;
const MAX_DEPTH: u32 = 2;

fn span(line: u32) -> Option<Span> {
    let at = Position { line, column: 0 };
    Span::new(at, at).ok()
}

fn symbol(id: u64) -> Option<Symbol> {
    Some(Symbol {
        id: SymbolId::new(id),
        file: FileId::new(id % FILES + 1),
        kind: SymbolKind::Function,
        name: SymbolName::new(format!("item{id}")).ok()?,
        qualified_name: QualifiedName::new(format!("app.module{}.item{id}", id % FILES)).ok()?,
        parent: None,
        visibility: Some(Visibility::Public),
        is_static: false,
        span: span(1)?,
        signature: None,
        doc: None,
    })
}

fn reference(id: u64) -> Option<Reference> {
    let to = if id.is_multiple_of(HUB_SHARE) {
        HUB
    } else {
        // Low ids are used much more often than high ones.
        (id * SPREAD_TO) % ((id * SPREAD_FROM) % SYMBOLS + 1) + 1
    };
    Some(Reference {
        id: ReferenceId::new(id),
        from: SymbolId::new((id * SPREAD_FROM) % SYMBOLS + 1),
        to: SymbolId::new(to),
        file: FileId::new(id % FILES + 1),
        span: span(u32::try_from(id).ok()? % LINES)?,
        kind: ReferenceKind::Call,
        confidence: Confidence::Certain,
    })
}

fn file(id: u64) -> Option<SourceFile> {
    Some(SourceFile {
        id: FileId::new(id),
        snapshot: SnapshotId::new(SNAPSHOT),
        path: RelativePath::new(format!("src/module{id}.rs")).ok()?,
        language: LanguageId::new("rust").ok()?,
        content_hash: ContentHash::new(format!("hash{id}")).ok()?,
        loc: Loc(250),
    })
}

fn project() -> Option<Project> {
    Some(Project {
        id: ProjectId::new(PROJECT),
        name: ProjectName::new("bench").ok()?,
        source: ProjectSource::Folder(FolderPath::new("/bench").ok()?),
        layers: Vec::new(),
    })
}

fn snapshot() -> SnapshotData {
    SnapshotData {
        snapshot: Snapshot {
            id: SnapshotId::new(SNAPSHOT),
            project: ProjectId::new(PROJECT),
            commit: None,
            created: Timestamp(1),
            mode: ScanMode::Full,
            stats: SnapshotStats::default(),
        },
        files: (1..=FILES).filter_map(file).collect(),
        symbols: (1..=SYMBOLS).filter_map(symbol).collect(),
        references: (1..=REFERENCES).filter_map(reference).collect(),
        metrics: Vec::new(),
        fingerprints: Vec::new(),
        removed: Vec::new(),
    }
}

fn store() -> Option<SqliteStore> {
    let store = SqliteStore::open_in_memory(NonZeroU32::MIN).ok()?;
    store.save_project(&project()?).ok()?;
    store.persist(snapshot()).ok()?;
    Some(store)
}

fn query(direction: Direction, depth: u32) -> UsageQuery {
    UsageQuery {
        symbol: SymbolId::new(HUB),
        direction,
        depth,
        include_tests: true,
    }
}

/// The reads of one `usages.query` call at depth 2 for the hub, in the order the command makes
/// them: the walk in both directions, the direct usages for the counts, and the impact. A failed
/// read ends the benchmark.
fn command_reads(store: &SqliteStore) -> Option<()> {
    let snapshot = SnapshotId::new(SNAPSHOT);
    for walk in [
        query(Direction::Incoming, MAX_DEPTH),
        query(Direction::Outgoing, MAX_DEPTH),
        query(Direction::Incoming, 1),
    ] {
        black_box(store.usages(snapshot, &walk, LIMIT).ok()?);
    }
    black_box(
        store
            .impact(snapshot, SymbolId::new(HUB), MAX_DEPTH, LIMIT)
            .ok()?,
    );
    Some(())
}

fn queries(c: &mut Criterion) -> Option<()> {
    let store = store()?;
    command_reads(&store)?;
    let mut group = c.benchmark_group("usages_page");
    group.bench_function("depth_2_hub", |b| b.iter(|| command_reads(&store)));
    group.finish();
    Some(())
}

fn main() -> ExitCode {
    let mut criterion = Criterion::default().configure_from_args();
    if queries(&mut criterion).is_none() {
        return ExitCode::FAILURE;
    }
    criterion.final_summary();
    ExitCode::SUCCESS
}
