//! What the benchmarks of the store share: a stored snapshot with the size of a 100k-line project.

use std::num::NonZeroU32;

use codeskimr_application::ports::{PendingReference, SnapshotData, SnapshotStore, SymbolHash};
use codeskimr_domain::{
    ContentHash, FileId, FolderPath, LanguageId, Loc, Position, Project, ProjectId, ProjectName,
    ProjectSource, QualifiedName, Reference, RelativePath, ScanMode, Snapshot, SnapshotId,
    SnapshotStats, SourceFile, Span, Symbol, SymbolId, SymbolKind, SymbolName, Timestamp,
    Visibility,
};
use codeskimr_storage::SqliteStore;

/// Symbols of a project of about 100k lines.
pub const SYMBOLS: u64 = 20_000;
/// Files of such a project.
pub const FILES: u64 = 400;
pub const SNAPSHOT: u64 = 1;
const PROJECT: u64 = 1;

pub fn span(line: u32) -> Option<Span> {
    let at = Position { line, column: 0 };
    Span::new(at, at).ok()
}

/// Symbol `id`, named by `name`, in one of the files.
pub fn symbol(id: u64, name: &str) -> Option<Symbol> {
    Some(Symbol {
        id: SymbolId::new(id),
        file: FileId::new(id % FILES + 1),
        kind: SymbolKind::Function,
        name: SymbolName::new(name).ok()?,
        qualified_name: QualifiedName::new(format!("app.module{}.{name}", id % FILES)).ok()?,
        parent: None,
        visibility: Some(Visibility::Public),
        is_static: false,
        span: span(1)?,
        signature: None,
        doc: None,
        origin: codeskimr_domain::SymbolOrigin::Syntax,
    })
}

fn file(snapshot: u64, id: u64) -> Option<SourceFile> {
    Some(SourceFile {
        id: FileId::new(id),
        snapshot: SnapshotId::new(snapshot),
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

/// What a snapshot of the benchmarks holds besides its files.
pub struct Content {
    pub symbols: Vec<Symbol>,
    pub references: Vec<Reference>,
    pub hashes: Vec<SymbolHash>,
    pub pending: Vec<PendingReference>,
}

/// The data of snapshot `snapshot` with `FILES` files from `first_file` on.
pub fn data(snapshot: u64, first_file: u64, content: Content) -> SnapshotData {
    SnapshotData {
        snapshot: Snapshot {
            id: SnapshotId::new(snapshot),
            project: ProjectId::new(PROJECT),
            commit: None,
            created: Timestamp(1),
            mode: ScanMode::Full,
            stats: SnapshotStats::default(),
        },
        files: (first_file..first_file + FILES)
            .filter_map(|id| file(snapshot, id))
            .collect(),
        symbols: content.symbols,
        references: content.references,
        metrics: Vec::new(),
        fingerprints: Vec::new(),
        removed: Vec::new(),
        pending: content.pending,
        dropped_references: Vec::new(),
        parse_micros: Vec::new(),
        hashes: content.hashes,
        warnings: Vec::new(),
        left_out: Vec::new(),
        naming_rules: String::new(),
    }
}

/// A store that holds one snapshot with `symbols`, their content `hashes` and `references` in
/// `FILES` files.
pub fn store(
    symbols: Vec<Symbol>,
    references: Vec<Reference>,
    hashes: Vec<SymbolHash>,
) -> Option<SqliteStore> {
    let content = Content {
        symbols,
        references,
        hashes,
        pending: Vec::new(),
    };
    let store = SqliteStore::open_in_memory(NonZeroU32::MIN).ok()?;
    store.save_project(&project()?).ok()?;
    store.persist(data(SNAPSHOT, 1, content)).ok()?;
    Some(store)
}
