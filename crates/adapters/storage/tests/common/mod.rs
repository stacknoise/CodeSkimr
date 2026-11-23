//! Builders for the integration tests of the storage adapter.
#![allow(dead_code, missing_docs)]

use std::num::NonZeroU32;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use codeskimr_application::ports::{SnapshotData, SnapshotStore};
use codeskimr_codes::Coded;
use codeskimr_domain::{
    Confidence, ContentHash, FileId, FolderPath, LanguageId, Layer, LayerName, Loc, Metric,
    MetricKind, MetricValue, Position, Project, ProjectId, ProjectName, ProjectSource,
    QualifiedName, Reference, ReferenceId, ReferenceKind, RelativePath, ScanMode, Snapshot,
    SnapshotId, SnapshotStats, SourceFile, Span, Symbol, SymbolId, SymbolKind, SymbolName,
    Timestamp, Visibility,
};
use codeskimr_storage::SqliteStore;

pub const PROJECT: ProjectId = ProjectId::new(1);

pub fn store(retention: u32) -> SqliteStore {
    let keep = NonZeroU32::new(retention).expect("retention is positive");
    SqliteStore::open_in_memory(keep).expect("in-memory database opens")
}

/// A store with the retention 3 and the project 1 registered, as most tests start.
pub fn registered_store() -> SqliteStore {
    let store = store(3);
    store.save_project(&project(1, "/work/app", &[])).unwrap();
    store
}

pub fn project(id: u64, folder: &str, layers: &[(&str, &[&str])]) -> Project {
    Project {
        id: ProjectId::new(id),
        name: ProjectName::new(format!("project {id}")).unwrap(),
        source: ProjectSource::Folder(FolderPath::new(folder).unwrap()),
        layers: layers
            .iter()
            .map(|(name, namespaces)| Layer {
                may_use: None,
                name: LayerName::new(*name).unwrap(),
                namespaces: namespaces
                    .iter()
                    .map(|namespace| QualifiedName::new(*namespace).unwrap())
                    .collect(),
            })
            .collect(),
    }
}

pub fn span(line: u32) -> Span {
    let start = Position { line, column: 0 };
    let end = Position {
        line: line + 1,
        column: 4,
    };
    Span::new(start, end).unwrap()
}

pub fn file(snapshot: u64, id: u64, path: &str, hash: &str) -> SourceFile {
    SourceFile {
        id: FileId::new(id),
        snapshot: SnapshotId::new(snapshot),
        path: RelativePath::new(path).unwrap(),
        language: LanguageId::new("csharp").unwrap(),
        content_hash: ContentHash::new(hash).unwrap(),
        loc: Loc(10),
    }
}

pub fn symbol(
    id: u64,
    file: u64,
    kind: SymbolKind,
    qualified: &str,
    parent: Option<u64>,
) -> Symbol {
    let name = qualified.rsplit('.').next().unwrap();
    Symbol {
        id: SymbolId::new(id),
        file: FileId::new(file),
        kind,
        name: SymbolName::new(name).unwrap(),
        qualified_name: QualifiedName::new(qualified).unwrap(),
        parent: parent.map(SymbolId::new),
        visibility: Some(Visibility::Public),
        is_static: false,
        span: span(1),
        signature: Some(format!("{name}()")),
        doc: None,
        origin: codeskimr_domain::SymbolOrigin::Syntax,
    }
}

pub fn reference(id: u64, file: u64, from: u64, to: u64) -> Reference {
    Reference {
        id: ReferenceId::new(id),
        from: SymbolId::new(from),
        to: SymbolId::new(to),
        file: FileId::new(file),
        span: span(2),
        kind: ReferenceKind::Call,
        confidence: Confidence::Certain,
        verified: false,
    }
}

pub fn metric(symbol: u64, kind: MetricKind, value: f64) -> Metric {
    Metric {
        symbol: SymbolId::new(symbol),
        kind,
        value: MetricValue::new(value).unwrap(),
    }
}

pub fn header(id: u64, mode: ScanMode) -> Snapshot {
    Snapshot {
        id: SnapshotId::new(id),
        project: PROJECT,
        commit: None,
        created: Timestamp(1_000),
        mode,
        stats: SnapshotStats::default(),
    }
}

pub fn data(snapshot: Snapshot) -> SnapshotData {
    SnapshotData {
        snapshot,
        files: Vec::new(),
        symbols: Vec::new(),
        references: Vec::new(),
        metrics: Vec::new(),
        fingerprints: Vec::new(),
        hashes: Vec::new(),
        removed: Vec::new(),
        pending: Vec::new(),
        dropped_references: Vec::new(),
        naming_rules: String::new(),
        parse_micros: Vec::new(),
        warnings: Vec::new(),
        ambiguous_places: None,
        left_out: Vec::new(),
    }
}

/// Short form of the registered code of a failure, for assertions.
pub fn code_of(error: &impl Coded) -> &'static str {
    error.code().entry().id
}

/// A project with the given layers, registered in a new store, and a full scan of snapshot 10
/// that has one file `a.cs` (id 11) holding `symbols`, `references` and `metrics`.
pub fn scanned(
    layers: &[(&str, &[&str])],
    symbols: Vec<Symbol>,
    references: Vec<Reference>,
    metrics: Vec<Metric>,
) -> SqliteStore {
    scanned_project(
        &project(1, "/work/app", layers),
        symbols,
        references,
        metrics,
    )
}

pub fn scanned_project(
    project: &Project,
    symbols: Vec<Symbol>,
    references: Vec<Reference>,
    metrics: Vec<Metric>,
) -> SqliteStore {
    use codeskimr_application::ports::SnapshotStore;
    let store = store(3);
    store.save_project(project).unwrap();
    let mut scan = data(header(10, ScanMode::Full));
    scan.files = vec![file(10, 11, "src/order.cs", "h")];
    scan.symbols = symbols;
    scan.references = references;
    scan.metrics = metrics;
    store.persist(scan).unwrap();
    store
}

static NEXT: AtomicU32 = AtomicU32::new(0);

/// A database path in a fresh temporary folder that is removed again on drop.
pub struct TempDb(pub PathBuf);

impl TempDb {
    pub fn new() -> Self {
        let unique = NEXT.fetch_add(1, Ordering::Relaxed);
        let folder =
            std::env::temp_dir().join(format!("codeskimr-storage-{}-{unique}", std::process::id()));
        Self(folder.join("nested").join("workspace.db"))
    }
}

impl Drop for TempDb {
    fn drop(&mut self) {
        if let Some(root) = self.0.parent().and_then(std::path::Path::parent) {
            let _ = std::fs::remove_dir_all(root);
        }
    }
}

/// The symbols and references one snapshot adds.
pub type Part = (Vec<Symbol>, Vec<Reference>);

/// A store with two projects and three snapshots: a full scan of project 1 (files 11 and 12), an
/// incremental one that carries them and adds file 13, and a full scan of project 2 (file 300).
pub fn three_snapshots(first: Part, second: Part, other: Part) -> SqliteStore {
    let store = store(5);
    store.save_project(&project(1, "/work/app", &[])).unwrap();
    store.save_project(&project(2, "/work/other", &[])).unwrap();
    let mut scan = data(header(10, ScanMode::Full));
    scan.files = vec![file(10, 11, "a.cs", "h1"), file(10, 12, "b.cs", "h2")];
    (scan.symbols, scan.references) = first;
    store.persist(scan).unwrap();
    let mut scan = data(header(20, ScanMode::Incremental));
    scan.files = vec![file(20, 13, "c.cs", "h3")];
    (scan.symbols, scan.references) = second;
    store.persist(scan).unwrap();
    let mut scan = data(header(30, ScanMode::Full));
    scan.snapshot.project = ProjectId::new(2);
    scan.files = vec![file(30, 300, "other.cs", "h4")];
    (scan.symbols, scan.references) = other;
    store.persist(scan).unwrap();
    store
}
