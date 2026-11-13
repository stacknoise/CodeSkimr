//! Fakes of the ports and builders for the tests of the MCP server.
#![allow(dead_code, missing_docs)]

use std::sync::{Arc, Mutex};

use codeskimr_application::cancel::CancelToken;
use codeskimr_application::config::{Conventions, McpLimits};
use codeskimr_application::conventions::ConventionChecker;
use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{
    AnalyzeRequest, Clock, CodeIndex, DependencyEdge, DependencyPage, DependencyQuery,
    DiscoveredFile, FileAnalyzer, FileDiscovery, FileFacts, ImpactPage, ImpactedSymbol, MemberRef,
    PreviousIndex, ProjectRepository, ProjectScanner, SearchHit, SimilarHit, SimilarQuery,
    SimilarityIndex, SnippetFingerprinter, SymbolDetail, SymbolKey, SymbolLabel, UsageEntry,
    UsageIndex, UsagePage,
};
use codeskimr_codes::ErrorCode;
use codeskimr_domain::{
    BodyCounts, Confidence, ContentHash, Direction, FileId, Fingerprint, Fingerprints, FolderPath,
    LanguageId, Layer, LayerEdge, LayerName, Loc, Metric, MetricKind, MetricValue, Position,
    Principal, Project, ProjectId, ProjectName, ProjectSource, QualifiedName, Reference,
    ReferenceId, ReferenceKind, RelativePath, ScanMode, Snapshot, SnapshotId, SnapshotStats,
    SourceFile, Span, Symbol, SymbolCounts, SymbolId, SymbolKind, SymbolName, Timestamp,
    UsageQuery, Visibility,
};
use codeskimr_mcp::{Ports, Tools};

pub const PROJECT: ProjectId = ProjectId::new(1);
pub const SNAPSHOT: SnapshotId = SnapshotId::new(10);
pub const SCANNED_AT: i64 = 1_000;
pub const NOW: i64 = 1_090;

pub fn limits() -> McpLimits {
    McpLimits {
        default_limit: 2,
        max_limit: 3,
        default_depth: 1,
        max_depth: 3,
        default_similarity_percent: 50,
        max_snippet_bytes: 64,
    }
}

pub fn symbol_named(id: u64, kind: SymbolKind, qualified: &str) -> Symbol {
    let name = qualified.rsplit('.').next().unwrap();
    let start = Position { line: 4, column: 0 };
    let end = Position { line: 9, column: 1 };
    Symbol {
        id: SymbolId::new(id),
        file: FileId::new(1),
        kind,
        name: SymbolName::new(name).unwrap(),
        qualified_name: QualifiedName::new(qualified).unwrap(),
        parent: None,
        visibility: Some(Visibility::Public),
        is_static: false,
        span: Span::new(start, end).unwrap(),
        signature: Some(format!("{name}()")),
        doc: Some("Does the work.".to_owned()),
    }
}

fn hit(id: u64, kind: SymbolKind, qualified: &str) -> SearchHit {
    let symbol = symbol_named(id, kind, qualified);
    SearchHit {
        symbol: symbol.id,
        name: symbol.name,
        qualified_name: symbol.qualified_name,
        kind,
        path: RelativePath::new("src/order.cs").unwrap(),
        parent: symbol.parent,
    }
}

pub struct FakeIndex {
    pub scanned: bool,
    pub fail: bool,
}

impl FakeIndex {
    fn failing(&self) -> Result<(), CodedError> {
        if self.fail {
            Err(CodedError::new(ErrorCode::DatabaseFailed))
        } else {
            Ok(())
        }
    }
}

impl CodeIndex for FakeIndex {
    fn latest_snapshot(&self, project: ProjectId) -> Result<Option<Snapshot>, CodedError> {
        self.failing()?;
        Ok(self.scanned.then(|| Snapshot {
            id: SNAPSHOT,
            project,
            commit: None,
            created: Timestamp(SCANNED_AT),
            mode: ScanMode::Full,
            stats: SnapshotStats::default(),
        }))
    }

    fn search(
        &self,
        _snapshot: SnapshotId,
        _query: &str,
        limit: u32,
    ) -> Result<Vec<SearchHit>, CodedError> {
        let all = vec![
            hit(1, SymbolKind::Class, "App.Order"),
            hit(2, SymbolKind::Method, "App.Order.Total"),
            hit(3, SymbolKind::Constant, "App.OrderLimit"),
            hit(4, SymbolKind::Class, "App.OrderService"),
        ];
        Ok(all.into_iter().take(limit as usize).collect())
    }

    fn symbol(
        &self,
        _snapshot: SnapshotId,
        key: &SymbolKey,
    ) -> Result<Option<SymbolDetail>, CodedError> {
        let found = match key {
            SymbolKey::Id(id) => id.get() == 1,
            SymbolKey::Qualified(name) => name.as_str() == "App.Domain.Order",
        };
        Ok(found.then(detail))
    }

    fn layer_edges(&self, _snapshot: SnapshotId) -> Result<Vec<LayerEdge>, CodedError> {
        let edge = |from: &str, to: &str, count: u32, bad: bool| LayerEdge {
            from: LayerName::new(from).unwrap(),
            to: LayerName::new(to).unwrap(),
            count,
            violates_layering: bad,
            in_cycle: bad,
        };
        Ok(vec![
            edge("Application", "Domain", 7, false),
            edge("Domain", "Application", 2, true),
        ])
    }
}

fn detail() -> SymbolDetail {
    let members = (2..=5)
        .map(|id| MemberRef {
            id: SymbolId::new(id),
            name: SymbolName::new(format!("member{id}")).unwrap(),
            kind: SymbolKind::Method,
        })
        .collect();
    SymbolDetail {
        symbol: symbol_named(1, SymbolKind::Class, "App.Domain.Order"),
        path: RelativePath::new("src/order.cs").unwrap(),
        members,
        metrics: vec![Metric {
            symbol: SymbolId::new(1),
            kind: MetricKind::Loc,
            value: MetricValue::new(40.0).unwrap(),
        }],
    }
}

pub struct FakeProjects;

impl ProjectRepository for FakeProjects {
    fn project(&self, id: ProjectId) -> Result<Project, CodedError> {
        if id != PROJECT {
            return Err(CodedError::new(ErrorCode::ProjectNotFound));
        }
        Ok(Project {
            id,
            name: ProjectName::new("Shop").unwrap(),
            source: ProjectSource::Folder(FolderPath::new("/work/shop").unwrap()),
            layers: vec![
                Layer {
                    name: LayerName::new("Domain").unwrap(),
                    namespaces: vec![QualifiedName::new("App.Domain").unwrap()],
                },
                Layer {
                    name: LayerName::new("Application").unwrap(),
                    namespaces: vec![QualifiedName::new("App.Application").unwrap()],
                },
            ],
        })
    }

    fn latest_files(&self, _id: ProjectId) -> Result<Vec<SourceFile>, CodedError> {
        Ok(Vec::new())
    }

    fn latest_index(&self, _id: ProjectId) -> Result<PreviousIndex, CodedError> {
        Ok(PreviousIndex::default())
    }
}

pub struct FixedClock;

impl Clock for FixedClock {
    fn now(&self) -> Timestamp {
        Timestamp(NOW)
    }
}

/// Remembers what it was asked, so a test can see how limits and depth were held.
#[derive(Default)]
pub struct FakeUsages {
    pub asked: Mutex<Vec<(UsageQuery, u32)>>,
    pub impact_asked: Mutex<Vec<(u32, u32)>>,
    pub dependencies_asked: Mutex<Vec<DependencyQuery>>,
}

fn label(id: u64, kind: SymbolKind, qualified: &str) -> SymbolLabel {
    SymbolLabel {
        id: SymbolId::new(id),
        qualified_name: QualifiedName::new(qualified).unwrap(),
        kind,
    }
}

fn entry(depth: u32, kind: ReferenceKind, confidence: Confidence) -> UsageEntry {
    let start = Position {
        line: 11,
        column: 7,
    };
    let end = Position {
        line: 11,
        column: 12,
    };
    UsageEntry {
        depth,
        reference: Reference {
            id: ReferenceId::new(u64::from(depth)),
            from: SymbolId::new(20),
            to: SymbolId::new(1),
            file: FileId::new(2),
            span: Span::new(start, end).unwrap(),
            kind,
            confidence,
            verified: false,
        },
        path: RelativePath::new("src/app/service.cs").unwrap(),
        from: label(20, SymbolKind::Method, "App.Application.Service.Run"),
        to: label(1, SymbolKind::Class, "App.Domain.Order"),
    }
}

fn affected(id: u64, qualified: &str, visibility: Option<Visibility>) -> ImpactedSymbol {
    ImpactedSymbol {
        label: label(id, SymbolKind::Method, qualified),
        depth: 1,
        verified: false,
        visibility,
        path: RelativePath::new("src/app/service.cs").unwrap(),
    }
}

impl UsageIndex for FakeUsages {
    fn usages(
        &self,
        _snapshot: SnapshotId,
        query: &UsageQuery,
        limit: u32,
    ) -> Result<UsagePage, CodedError> {
        self.asked.lock().unwrap().push((*query, limit));
        let mut verified = entry(1, ReferenceKind::Call, Confidence::Certain);
        verified.reference.verified = true;
        let all = vec![
            verified,
            entry(2, ReferenceKind::Read, Confidence::Probable),
            entry(2, ReferenceKind::TypeUse, Confidence::Certain),
        ];
        let total = all.len();
        let entries = match query.direction {
            Direction::Incoming => all.into_iter().take(limit as usize).collect(),
            Direction::Outgoing => Vec::new(),
        };
        Ok(UsagePage { total, entries })
    }

    fn impact(
        &self,
        _snapshot: SnapshotId,
        _symbol: SymbolId,
        depth: u32,
        limit: u32,
    ) -> Result<ImpactPage, CodedError> {
        self.impact_asked.lock().unwrap().push((depth, limit));
        let mut verified = affected(20, "App.Application.Service.Run", Some(Visibility::Public));
        verified.verified = true;
        let all = vec![
            verified,
            affected(
                21,
                "App.Application.Service.Helper",
                Some(Visibility::Private),
            ),
            affected(22, "App.Domain.Invoice.Pay", Some(Visibility::Public)),
            affected(23, "Other.Thing", None),
        ];
        let total = all.len();
        Ok(ImpactPage {
            total,
            symbols: all.into_iter().take(limit as usize).collect(),
        })
    }

    fn dependencies(
        &self,
        _snapshot: SnapshotId,
        query: &DependencyQuery,
        limit: u32,
    ) -> Result<DependencyPage, CodedError> {
        self.dependencies_asked.lock().unwrap().push(*query);
        let all = Self::far_sides();
        Ok(DependencyPage {
            total: all.len(),
            edges: all.into_iter().take(limit as usize).collect(),
        })
    }
}

/// Scans nothing: remembers who asked and answers with snapshot 11, or fails.
#[derive(Default)]
pub struct FakeScanner {
    pub asked: Mutex<Vec<(Principal, ProjectId)>>,
    pub fail: bool,
}

impl ProjectScanner for FakeScanner {
    fn rescan(&self, principal: &Principal, project: ProjectId) -> Result<Snapshot, CodedError> {
        self.asked.lock().unwrap().push((*principal, project));
        if self.fail {
            return Err(CodedError::new(ErrorCode::UnexpectedInternalError));
        }
        Ok(Snapshot {
            id: SnapshotId::new(11),
            project,
            commit: None,
            created: Timestamp(NOW - 5),
            mode: ScanMode::Incremental,
            stats: SnapshotStats {
                files: 3,
                symbols: 40,
                references: 90,
            },
        })
    }
}

fn far_side(id: u64, qualified: &str, count: u32) -> DependencyEdge {
    DependencyEdge {
        other: label(id, SymbolKind::Class, qualified),
        path: RelativePath::new("src/far.cs").unwrap(),
        count,
    }
}

impl FakeUsages {
    /// What the fake index finds around `App.Domain.Order`, whichever way it is asked.
    fn far_sides() -> Vec<DependencyEdge> {
        vec![
            far_side(22, "App.Domain.Invoice", 4),
            far_side(20, "App.Application.Service", 2),
            far_side(40, "Other.Thing", 1),
        ]
    }
}

/// Stored fingerprints: symbol 1 has some, the three functions it finds are made up.
pub struct FakeSimilarity {
    pub asked: Mutex<Vec<(u32, Option<SymbolId>, u32)>>,
    /// Whether symbol 1 has stored fingerprints.
    pub has_prints: bool,
}

pub fn prints(hashes: &[u64]) -> Fingerprints {
    let entries = (0..).zip(hashes).map(|(position, hash)| Fingerprint {
        hash: *hash,
        position,
    });
    Fingerprints::from_entries(entries.collect())
}

impl SimilarityIndex for FakeSimilarity {
    fn fingerprints_of(
        &self,
        _snapshot: SnapshotId,
        symbol: SymbolId,
    ) -> Result<Option<Fingerprints>, CodedError> {
        Ok((self.has_prints && symbol.get() == 1).then(|| prints(&[1, 2, 3])))
    }

    fn similar(
        &self,
        _snapshot: SnapshotId,
        query: &SimilarQuery<'_>,
    ) -> Result<Vec<SimilarHit>, CodedError> {
        let (minimum_percent, limit) = (query.minimum_percent, query.limit);
        self.asked
            .lock()
            .unwrap()
            .push((minimum_percent, query.exclude, limit));
        let start = Position { line: 4, column: 0 };
        let end = Position { line: 9, column: 1 };
        let all = [(30, 92), (31, 71), (32, 55)];
        Ok(all
            .iter()
            .map(|(id, percent)| SimilarHit {
                symbol: SymbolId::new(*id),
                qualified_name: QualifiedName::new(format!("App.Copy{id}")).unwrap(),
                kind: SymbolKind::Method,
                path: RelativePath::new("src/copy.cs").unwrap(),
                span: Span::new(start, end).unwrap(),
                percent: *percent,
            })
            .filter(|hit| hit.percent >= minimum_percent)
            .take(limit as usize)
            .collect())
    }
}

/// Fingerprints every snippet of the language `csharp` as `[7, 8]`; anything else has no pack,
/// and a snippet that says `short` is too short to have fingerprints.
pub struct FakeSnippets;

impl SnippetFingerprinter for FakeSnippets {
    fn fingerprint(&self, language: &LanguageId, source: &str) -> Result<Fingerprints, CodedError> {
        if language.as_str() != "csharp" {
            return Err(CodedError::new(ErrorCode::LanguagePackMissing));
        }
        Ok(if source.contains("short") {
            Fingerprints::default()
        } else {
            prints(&[7, 8])
        })
    }
}

/// A disk with one function of 50 code lines.
pub struct FakeDisk;

impl FileDiscovery for FakeDisk {
    fn discover(
        &self,
        _root: &FolderPath,
        _previous: &[SourceFile],
        _cancel: &CancelToken,
    ) -> Result<Vec<DiscoveredFile>, CodedError> {
        Ok(vec![DiscoveredFile {
            path: RelativePath::new("src/big.cs").unwrap(),
            language: LanguageId::new("csharp").unwrap(),
            content_hash: ContentHash::new("h").unwrap(),
            changed: true,
        }])
    }
}

impl FileAnalyzer for FakeDisk {
    fn analyze(&self, request: &AnalyzeRequest<'_>) -> Result<FileFacts, CodedError> {
        let file = SourceFile {
            loc: Loc(60),
            content_hash: request.file.content_hash.clone(),
            language: request.file.language.clone(),
            path: request.file.path.clone(),
            snapshot: request.snapshot,
            id: request.ids.file(),
        };
        let mut function = symbol_named(
            request.ids.symbol().get(),
            SymbolKind::Method,
            "App.Big.Run",
        );
        function.file = file.id;
        let counts = SymbolCounts {
            symbol: function.id,
            loc: Loc(50),
            body: Some(BodyCounts::default()),
            counted: true,
        };
        Ok(FileFacts {
            file,
            symbols: vec![function],
            pending: Vec::new(),
            fingerprints: Vec::new(),
            counts: vec![counts],
        })
    }
}

pub fn checker() -> Arc<ConventionChecker> {
    let disk = Arc::new(FakeDisk);
    Arc::new(ConventionChecker::new(
        disk.clone(),
        disk,
        Conventions {
            max_function_lines: 40,
            duplicate_similarity_percent: 85,
            duplicate_min_lines: 5,
            test_folders: vec!["tests".to_owned()],
        },
    ))
}

pub fn tools_with(index: FakeIndex) -> Tools {
    tools_using(index, Arc::new(FakeUsages::default()))
}

pub fn tools_without_prints() -> Tools {
    build(scanned(), Arc::new(FakeUsages::default()), false)
}

fn scanned() -> FakeIndex {
    FakeIndex {
        scanned: true,
        fail: false,
    }
}

pub fn tools_using(index: FakeIndex, usages: Arc<FakeUsages>) -> Tools {
    build(index, usages, true)
}

pub fn tools_scanning(scanner: Arc<FakeScanner>) -> Tools {
    assemble(scanned(), Arc::new(FakeUsages::default()), true, scanner)
}

fn build(index: FakeIndex, usages: Arc<FakeUsages>, has_prints: bool) -> Tools {
    assemble(index, usages, has_prints, Arc::new(FakeScanner::default()))
}

fn assemble(
    index: FakeIndex,
    usages: Arc<FakeUsages>,
    has_prints: bool,
    scanner: Arc<FakeScanner>,
) -> Tools {
    Tools::new(
        Ports {
            projects: Arc::new(FakeProjects),
            index: Arc::new(index),
            usages,
            similarity: Arc::new(FakeSimilarity {
                asked: Mutex::default(),
                has_prints,
            }),
            snippets: Arc::new(FakeSnippets),
            checker: checker(),
            clock: Arc::new(FixedClock),
            scanner,
        },
        limits(),
        PROJECT,
    )
}

pub fn tools() -> Tools {
    tools_with(FakeIndex {
        scanned: true,
        fail: false,
    })
}
