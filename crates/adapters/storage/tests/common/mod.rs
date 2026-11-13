//! Builders for the integration tests of the storage adapter.
#![allow(dead_code, missing_docs)]

use std::num::NonZeroU32;

use codeskimr_application::ports::SnapshotData;
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

pub fn project(id: u64, folder: &str, layers: &[(&str, &[&str])]) -> Project {
    Project {
        id: ProjectId::new(id),
        name: ProjectName::new(format!("project {id}")).unwrap(),
        source: ProjectSource::Folder(FolderPath::new(folder).unwrap()),
        layers: layers
            .iter()
            .map(|(name, namespaces)| Layer {
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
    use codeskimr_application::ports::SnapshotStore;
    let store = store(3);
    store
        .save_project(&project(1, "/work/app", layers))
        .unwrap();
    let mut scan = data(header(10, ScanMode::Full));
    scan.files = vec![file(10, 11, "src/order.cs", "h")];
    scan.symbols = symbols;
    scan.references = references;
    scan.metrics = metrics;
    store.persist(scan).unwrap();
    store
}
