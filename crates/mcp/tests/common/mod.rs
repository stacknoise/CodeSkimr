//! Fakes of the ports and builders for the tests of the MCP server.
#![allow(dead_code, missing_docs)]

use std::sync::{Arc, Mutex};

use codeskimr_application::cancel::CancelToken;
use codeskimr_application::config::{Conventions, KeySources, McpLimits, UnusedIgnore};
use codeskimr_application::conventions::ConventionChecker;
use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{
    AlertDismissal, AlertFilter, AlertPurge, AlertRaise, AnalyzeRequest, AnnotationCreation,
    AnnotationFilter, AnnotationStore, AnnotationUpdate, Clock, CodeIndex, CodeSpan,
    DependencyEdge, DependencyPage, DependencyQuery, DiscoveredFile, Excerpt, FileAnalyzer,
    FileDiscovery, FileFacts, FileHit, ImpactPage, ImpactedSymbol, KeyEntry, KeyKind, KeySearch,
    LanguageLine, LanguageOutcome, LanguageShare, LastScan, Level2Report, MemberRef, PathsReport,
    PreviousIndex, ProjectCatalog, ProjectKeys, ProjectOverview, ProjectRepository, ProjectScanner,
    ProjectToolCheck, ReferenceSite, ReferenceSites, ReportWrite, Rescanned, ScanReport,
    ScanReportStore, SearchHit, SimilarHit, SimilarQuery, SimilarityIndex, SnippetFingerprinter,
    SourceText, SymbolDetail, SymbolKey, SymbolLabel, TypeFocus, UsageEntry, UsageIndex, UsagePage,
};
use codeskimr_application::unused::{Finding, Findings, UnusedSymbols};
use codeskimr_codes::ErrorCode;
use codeskimr_domain::{
    Alert, Annotation, AnnotationId, BodyCounts, Confidence, ContentHash, Direction, FileId,
    Fingerprint, Fingerprints, FolderPath, LanguageId, Layer, LayerEdge, LayerName, Loc, Metric,
    MetricKind, MetricValue, NamespaceGraph, Position, Principal, Project, ProjectId, ProjectName,
    ProjectSource, QualifiedName, Reference, ReferenceId, ReferenceKind, RelativePath, ScanLevel,
    ScanMode, ScanOrigin, SkipReason, Snapshot, SnapshotId, SnapshotStats, SourceFile, Span,
    Symbol, SymbolBaseline, SymbolCounts, SymbolId, SymbolKind, SymbolName, Timestamp, ToolFinding,
    ToolReport, ToolRole, ToolState, UsageQuery, Visibility,
};
use codeskimr_domain::{Assessment, RankingQuery, UnusedReason};
use codeskimr_i18n::Messages;
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
        context_items_short: 1,
        context_items_medium: 2,
        context_items_long: 3,
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
        origin: codeskimr_domain::SymbolOrigin::Syntax,
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

    fn search_files(
        &self,
        _snapshot: SnapshotId,
        _query: &str,
        _limit: u32,
    ) -> Result<Vec<FileHit>, CodedError> {
        Ok(Vec::new())
    }

    fn code_spans(
        &self,
        _snapshot: SnapshotId,
        _path: &RelativePath,
        _lines: (u32, u32),
    ) -> Result<Vec<CodeSpan>, CodedError> {
        Ok(Vec::new())
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

    fn namespace_graph(&self, _snapshot: SnapshotId) -> Result<NamespaceGraph, CodedError> {
        Ok(NamespaceGraph::default())
    }

    fn type_focus(
        &self,
        _snapshot: SnapshotId,
        _symbol: SymbolId,
    ) -> Result<Option<TypeFocus>, CodedError> {
        Ok(None)
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
        same_name: 0,
        symbol: symbol_named(1, SymbolKind::Class, "App.Domain.Order"),
        path: RelativePath::new("src/order.cs").unwrap(),
        language: LanguageId::new("csharp").unwrap(),
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
                    may_use: None,
                    name: LayerName::new("Domain").unwrap(),
                    namespaces: vec![QualifiedName::new("App.Domain").unwrap()],
                },
                Layer {
                    may_use: Some(vec![LayerName::new("Domain").unwrap()]),
                    name: LayerName::new("Application").unwrap(),
                    namespaces: vec![QualifiedName::new("App.Application").unwrap()],
                },
                Layer {
                    may_use: Some(vec![LayerName::new("Application").unwrap()]),
                    name: LayerName::new("Web").unwrap(),
                    namespaces: vec![QualifiedName::new("App.Web").unwrap()],
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
    /// The paths of each question, in the same order.
    pub asked_paths: Mutex<Vec<Vec<String>>>,
    /// What the scan reports of its paths.
    pub paths: PathsReport,
    pub fail: bool,
}

impl ProjectScanner for FakeScanner {
    fn rescan_paths(
        &self,
        principal: &Principal,
        project: ProjectId,
        paths: &[String],
    ) -> Result<Rescanned, CodedError> {
        self.asked.lock().unwrap().push((*principal, project));
        self.asked_paths.lock().unwrap().push(paths.to_vec());
        if self.fail {
            return Err(CodedError::new(ErrorCode::UnexpectedInternalError));
        }
        let snapshot = Snapshot {
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
        };
        Ok(Rescanned {
            snapshot,
            paths: self.paths.clone(),
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

/// Stored references: every file has one of a Web page into the domain, which the Web layer may
/// not use, and one into the application, which it may.
pub struct FakeReferences;

impl ReferenceSites for FakeReferences {
    fn references_from(
        &self,
        _snapshot: SnapshotId,
        files: &[RelativePath],
    ) -> Result<Vec<ReferenceSite>, CodedError> {
        let span = Span::new(
            Position { line: 6, column: 0 },
            Position { line: 9, column: 1 },
        )
        .unwrap();
        let site = |path: &RelativePath, to: &str| ReferenceSite {
            path: path.clone(),
            from: QualifiedName::new("App.Web.Page.Show").unwrap(),
            span,
            to: QualifiedName::new(to).unwrap(),
        };
        Ok(files
            .iter()
            .flat_map(|path| {
                [
                    site(path, "App.Domain.Order"),
                    site(path, "App.Application.Service.Place"),
                    site(path, "App.Domain.Customer"),
                ]
            })
            .collect())
    }
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

/// The fake disk has no text lines: a function never carries the marker of a reasoned exception.
impl SourceText for FakeDisk {
    fn excerpt(
        &self,
        _: &FolderPath,
        _: &RelativePath,
        _: (u32, u32),
        _: (u32, u64),
    ) -> Result<Excerpt, CodedError> {
        Ok(Excerpt {
            lines: Vec::new(),
            truncated: false,
        })
    }
}

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
            body: Some(BodyCounts {
                cognitive: Some(20),
                parameters: Some(7),
                literals: Some(3),
                ..BodyCounts::default()
            }),
            counted: true,
        };
        Ok(FileFacts {
            file,
            symbols: vec![function],
            pending: Vec::new(),
            fingerprints: Vec::new(),
            hashes: Vec::new(),
            counts: vec![counts],
        })
    }
}

pub fn checker() -> Arc<ConventionChecker> {
    let disk = Arc::new(FakeDisk);
    Arc::new(ConventionChecker::new(
        disk.clone(),
        disk.clone(),
        disk,
        Conventions {
            max_function_lines: 40,
            max_cognitive_complexity: 15,
            max_parameters: 5,
            duplicate_similarity_percent: 85,
            duplicate_min_lines: 5,
            duplicate_max_owners: 50,
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
    assemble(
        (scanned(), Arc::new(FakeUsages::default()), true),
        scanner,
        Arc::new(FakeToolCheck::default()),
    )
}

fn build(index: FakeIndex, usages: Arc<FakeUsages>, has_prints: bool) -> Tools {
    assemble(
        (index, usages, has_prints),
        Arc::new(FakeScanner::default()),
        Arc::new(FakeToolCheck::default()),
    )
}

/// Tools whose tool check is `check`.
pub fn tools_checking(check: Arc<FakeToolCheck>) -> Tools {
    assemble(
        (scanned(), Arc::new(FakeUsages::default()), true),
        Arc::new(FakeScanner::default()),
        check,
    )
}

fn assemble(
    setup: (FakeIndex, Arc<FakeUsages>, bool),
    scanner: Arc<FakeScanner>,
    tool_check: Arc<FakeToolCheck>,
) -> Tools {
    assemble_with(limits(), setup, scanner, tool_check)
}

/// Tools that may list up to `max_limit` entries, for a test that needs the whole answer.
pub fn tools_listing(max_limit: u32) -> Tools {
    assemble_with(
        McpLimits {
            max_limit,
            ..limits()
        },
        (scanned(), Arc::new(FakeUsages::default()), true),
        Arc::new(FakeScanner::default()),
        Arc::new(FakeToolCheck::default()),
    )
}

fn assemble_with(
    limits: McpLimits,
    (index, usages, has_prints): (FakeIndex, Arc<FakeUsages>, bool),
    scanner: Arc<FakeScanner>,
    tool_check: Arc<FakeToolCheck>,
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
            references: Arc::new(FakeReferences),
            snippets: Arc::new(FakeSnippets),
            checker: checker(),
            clock: Arc::new(FixedClock),
            scanner,
            catalog: Arc::new(FakeCatalog),
            tool_check,
            messages: Arc::new(Messages::load().unwrap()),
            keys: Arc::new(FakeKeys),
            key_sources: key_sources(),
            annotations: Arc::new(FakeAnnotations::default()),
            reports: Arc::new(FakeReports),
            unused: Arc::new(FakeUnused),
        },
        limits,
        PROJECT,
    )
}

pub fn tools() -> Tools {
    tools_with(FakeIndex {
        scanned: true,
        fail: false,
    })
}

/// The languages of the project: C# and Go.
pub struct FakeCatalog;

impl ProjectCatalog for FakeCatalog {
    fn overviews(&self) -> Result<Vec<ProjectOverview>, CodedError> {
        Ok(Vec::new())
    }

    fn languages(&self, _id: ProjectId) -> Result<Vec<LanguageShare>, CodedError> {
        Ok(["csharp", "go"]
            .into_iter()
            .map(|name| LanguageShare {
                language: LanguageId::new(name).unwrap(),
                files: 3,
                loc: 120,
                symbols: 30,
                parse_micros: None,
            })
            .collect())
    }
}

/// A stored report of a browser scan at level 2: C# checked in part, Go without its tools, and
/// Rust not confirmed.
pub struct FakeReports;

impl ScanReportStore for FakeReports {
    fn store_report(&self, _write: &ReportWrite) -> Result<(), CodedError> {
        Err(CodedError::new(ErrorCode::DatabaseFailed))
    }

    fn report(&self, _project: ProjectId) -> Result<Option<ScanReport>, CodedError> {
        let line = |language: &str, outcome| LanguageLine {
            language: LanguageId::new(language).unwrap(),
            outcome,
        };
        Ok(Some(ScanReport {
            last: LastScan {
                at: Timestamp(500),
                level: ScanLevel::Two,
                origin: ScanOrigin::Browser,
            },
            level2: Some(Level2Report {
                at: Timestamp(510),
                origin: ScanOrigin::Browser,
                languages: vec![
                    line("csharp", LanguageOutcome::Checked { files: 7, stale: 2 }),
                    line("go", LanguageOutcome::Skipped(SkipReason::ToolsUnavailable)),
                    line("rust", LanguageOutcome::Skipped(SkipReason::NotConfirmed)),
                ],
            }),
        }))
    }
}

/// A tool check with a fixed answer that counts how often it ran.
pub struct FakeToolCheck {
    pub states: Vec<(&'static str, ToolState)>,
    pub runs: Mutex<u32>,
    pub fail: bool,
}

impl Default for FakeToolCheck {
    /// The indexer is not installed, the SDK is.
    fn default() -> Self {
        Self::with(&[
            ("scip-dotnet", ToolState::Missing),
            ("dotnet", ToolState::Ready),
        ])
    }
}

impl FakeToolCheck {
    pub fn with(states: &[(&'static str, ToolState)]) -> Self {
        Self {
            states: states.to_vec(),
            runs: Mutex::new(0),
            fail: false,
        }
    }
}

impl ProjectToolCheck for FakeToolCheck {
    fn check(&self, _project: ProjectId) -> Result<ToolReport, CodedError> {
        *self.runs.lock().unwrap() += 1;
        if self.fail {
            return Err(CodedError::new(ErrorCode::UnexpectedInternalError));
        }
        let finding = |(tool, state): &(&str, ToolState)| ToolFinding {
            language: LanguageId::new("csharp").unwrap(),
            tool: (*tool).to_owned(),
            role: if *tool == "dotnet" {
                ToolRole::Toolchain
            } else {
                ToolRole::Indexer
            },
            state: *state,
            found: (*state == ToolState::Ready).then(|| "8.0.404".to_owned()),
            required: Some("9.0.100".to_owned()),
            purpose_key: format!("tool-{tool}-purpose"),
            install_key: format!("tool-{tool}-install"),
        };
        Ok(ToolReport {
            checked_at: Timestamp(NOW),
            findings: self.states.iter().map(finding).collect(),
        })
    }

    fn stored(&self, _project: ProjectId) -> Result<Option<ToolReport>, CodedError> {
        self.check(PROJECT).map(Some)
    }
}

pub fn key_sources() -> KeySources {
    KeySources {
        config_folders: vec!["config".to_owned()],
        i18n_folders: vec!["locales".to_owned()],
        max_files: 10,
        max_file_kb: 1,
        max_depth: 2,
    }
}

/// The symbols without a usage: a private function, a public class, one that the exception list
/// takes out, and one test symbol that is only there when the question asks for test code.
pub struct FakeUnused;

/// The language whose question the fake answers with the failure of a broken exception list.
pub const BROKEN_LIST: &str = "brokenlist";

fn finding(id: u64, name: &str, assessment: Assessment, test: bool) -> Finding {
    Finding {
        id: SymbolId::new(id),
        name: name.to_owned(),
        qualified_name: format!("Demo.{name}"),
        kind: SymbolKind::Method,
        path: "src/Demo.cs".to_owned(),
        language: "csharp".to_owned(),
        assessment,
        reason: match assessment {
            Assessment::Likely => UnusedReason::Private,
            Assessment::Public => UnusedReason::Public,
            Assessment::Approximate => UnusedReason::Approximate,
        },
        test,
    }
}

impl UnusedSymbols for FakeUnused {
    fn unused(
        &self,
        _: &Project,
        _: SnapshotId,
        query: &RankingQuery,
    ) -> Result<Findings, CodedError> {
        // A question about this language stands for a project whose exception list is invalid.
        if query
            .language
            .as_ref()
            .is_some_and(|id| id.as_str() == BROKEN_LIST)
        {
            return Err(CodedError::new(ErrorCode::ConfigInvalidValue));
        }
        let mut kept = vec![
            finding(1, "Orphan", Assessment::Likely, false),
            finding(2, "Spare", Assessment::Public, false),
        ];
        if query.include_tests {
            kept.push(finding(3, "Runs", Assessment::Likely, true));
        }
        Ok(Findings {
            kept,
            left_out: vec![(
                finding(4, "Migrate", Assessment::Likely, false),
                UnusedIgnore {
                    paths: Vec::new(),
                    symbols: vec!["*.Migrate".to_owned()],
                    reason: "called by the migration tool".to_owned(),
                },
            )],
            tests_hidden: usize::from(!query.include_tests),
        })
    }
}

/// The keys of the project: two configuration keys and one translation key.
pub struct FakeKeys;

impl ProjectKeys for FakeKeys {
    fn keys(&self, _: &FolderPath, _: &KeySearch) -> Result<Vec<KeyEntry>, CodedError> {
        let key = |name: &str, kind, path: &str, line| KeyEntry {
            name: name.to_owned(),
            kind,
            path: RelativePath::new(path).unwrap(),
            line,
        };
        Ok(vec![
            key(
                "mcp.default_limit",
                KeyKind::Config,
                "config/default.toml",
                4,
            ),
            key("mcp.max_limit", KeyKind::Config, "config/default.toml", 5),
            key("error-LIMIT-1", KeyKind::I18n, "locales/en/errors.ftl", 0),
            key("other.setting", KeyKind::Config, "config/default.toml", 9),
        ])
    }
}

/// Annotations and alerts that a test hands to the tools; nothing can be written to it.
#[derive(Default)]
pub struct FakeAnnotations {
    pub annotations: Vec<Annotation>,
    pub alerts: Vec<Alert>,
}

fn read_only<T>() -> Result<T, CodedError> {
    Err(CodedError::new(ErrorCode::UnexpectedInternalError))
}

impl AnnotationStore for FakeAnnotations {
    fn create_annotation(&self, _creation: &AnnotationCreation) -> Result<Annotation, CodedError> {
        read_only()
    }

    fn update_annotation(&self, _update: &AnnotationUpdate) -> Result<Annotation, CodedError> {
        read_only()
    }

    fn delete_annotation(&self, _id: AnnotationId) -> Result<(), CodedError> {
        read_only()
    }

    fn list_annotations(&self, filter: &AnnotationFilter) -> Result<Vec<Annotation>, CodedError> {
        Ok(self
            .annotations
            .iter()
            .filter(|found| {
                found.project == filter.project
                    && filter.kind.is_none_or(|kind| found.content.kind == kind)
                    && filter
                        .target
                        .as_ref()
                        .is_none_or(|target| &found.target == target)
            })
            .cloned()
            .collect())
    }

    fn refresh_targets(
        &self,
        _project: ProjectId,
        _snapshot: SnapshotId,
    ) -> Result<u64, CodedError> {
        read_only()
    }

    fn baseline(&self, _annotation: AnnotationId) -> Result<Option<SymbolBaseline>, CodedError> {
        read_only()
    }

    fn store_baseline(
        &self,
        _annotation: AnnotationId,
        _baseline: &SymbolBaseline,
    ) -> Result<(), CodedError> {
        read_only()
    }

    fn raise_alert(&self, _raise: &AlertRaise) -> Result<Alert, CodedError> {
        read_only()
    }

    fn list_alerts(&self, filter: &AlertFilter) -> Result<Vec<Alert>, CodedError> {
        let mine: Vec<&Annotation> = self
            .annotations
            .iter()
            .filter(|found| found.project == filter.project)
            .collect();
        Ok(self
            .alerts
            .iter()
            .filter(|alert| {
                mine.iter().any(|found| found.id == alert.annotation)
                    && filter.state.is_none_or(|state| alert.state == state)
            })
            .cloned()
            .collect())
    }

    fn dismiss_alert(&self, _dismissal: &AlertDismissal) -> Result<Alert, CodedError> {
        read_only()
    }

    fn purge_dismissed(&self, _purge: &AlertPurge) -> Result<u64, CodedError> {
        read_only()
    }
}

/// Tools over the same fakes, with these annotations and alerts.
pub fn tools_annotated(annotations: FakeAnnotations) -> Tools {
    Tools::new(
        Ports {
            projects: Arc::new(FakeProjects),
            index: Arc::new(scanned()),
            usages: Arc::new(FakeUsages::default()),
            similarity: Arc::new(FakeSimilarity {
                asked: Mutex::default(),
                has_prints: true,
            }),
            references: Arc::new(FakeReferences),
            snippets: Arc::new(FakeSnippets),
            checker: checker(),
            clock: Arc::new(FixedClock),
            scanner: Arc::new(FakeScanner::default()),
            catalog: Arc::new(FakeCatalog),
            tool_check: Arc::new(FakeToolCheck::default()),
            messages: Arc::new(Messages::load().unwrap()),
            keys: Arc::new(FakeKeys),
            key_sources: key_sources(),
            annotations: Arc::new(annotations),
            reports: Arc::new(FakeReports),
            unused: Arc::new(FakeUnused),
        },
        limits(),
        PROJECT,
    )
}
