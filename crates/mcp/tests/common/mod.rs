//! Fakes of the ports and builders for the tests of the MCP server.
#![allow(dead_code, missing_docs)]

use std::sync::{Arc, Mutex};

use codeskimr_application::config::McpLimits;
use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{
    Clock, CodeIndex, ImpactPage, ImpactedSymbol, MemberRef, PreviousIndex, ProjectRepository,
    SearchHit, SymbolDetail, SymbolKey, SymbolLabel, UsageEntry, UsageIndex, UsagePage,
};
use codeskimr_codes::ErrorCode;
use codeskimr_domain::{
    Confidence, Direction, FileId, FolderPath, Layer, LayerEdge, LayerName, Metric, MetricKind,
    MetricValue, Position, Project, ProjectId, ProjectName, ProjectSource, QualifiedName,
    Reference, ReferenceId, ReferenceKind, RelativePath, ScanMode, Snapshot, SnapshotId,
    SnapshotStats, SourceFile, Span, Symbol, SymbolId, SymbolKind, SymbolName, Timestamp,
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
        Ok(vec![LayerEdge {
            from: LayerName::new("Application").unwrap(),
            to: LayerName::new("Domain").unwrap(),
            count: 7,
            violates_layering: false,
            in_cycle: false,
        }])
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
        let all = vec![
            entry(1, ReferenceKind::Call, Confidence::Certain),
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
        let all = vec![
            affected(20, "App.Application.Service.Run", Some(Visibility::Public)),
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
}

pub fn tools_with(index: FakeIndex) -> Tools {
    tools_using(index, Arc::new(FakeUsages::default()))
}

pub fn tools_using(index: FakeIndex, usages: Arc<FakeUsages>) -> Tools {
    Tools::new(
        Ports {
            projects: Arc::new(FakeProjects),
            index: Arc::new(index),
            usages,
            clock: Arc::new(FixedClock),
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
