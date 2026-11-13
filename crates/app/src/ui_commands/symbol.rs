//! The command that shows one symbol of the explorer: breadcrumb, metrics and member table.

use std::sync::Arc;

use codeskimr_application::access;
use codeskimr_application::config::MetricThresholds;
use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{MemberRow, SymbolDetail, SymbolKey};
use codeskimr_application::registry::{Command, CommandDescriptor, CommandId, Request};
use codeskimr_codes::ErrorCode;
use codeskimr_domain::{Permission, SnapshotId, SymbolId, Visibility};
use serde::{Deserialize, Serialize};

use super::consts::{
    BREADCRUMB_DEPTH, EXPLORER_SYMBOL, LINE_OFFSET, SYMBOL_DESCRIPTION, SYMBOL_TITLE,
};
use super::explorer::ExplorerPorts;
use super::rating;

/// A symbol in the breadcrumb.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Crumb {
    /// Id of the symbol.
    pub id: u64,
    /// Simple name.
    pub name: String,
    /// Kind label.
    pub kind: &'static str,
}

/// One measured value of the symbol.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MetricRow {
    /// Metric label.
    pub kind: &'static str,
    /// The value.
    pub value: f64,
    /// `good`, `medium` or `poor` against the configured thresholds; `None` if there are none.
    pub rating: Option<&'static str>,
}

/// One row of the member table.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MemberOut {
    /// Id of the member.
    pub id: u64,
    /// Simple name.
    pub name: String,
    /// Kind label.
    pub kind: &'static str,
    /// Visibility label, if there is one.
    pub visibility: Option<&'static str>,
    /// Signature as written.
    pub signature: Option<String>,
    /// Cyclomatic complexity, if measured.
    pub complexity: Option<f64>,
    /// Rating of the complexity.
    pub complexity_rating: Option<&'static str>,
    /// Number of places that use it.
    pub usages: u32,
    /// First line, counted from 1.
    pub first_line: u32,
    /// Last line, counted from 1.
    pub last_line: u32,
}

/// The answer of `explorer.symbol`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SymbolOut {
    /// Id of the snapshot the answer is from.
    pub snapshot: u64,
    /// Id of the symbol.
    pub id: u64,
    /// Simple name.
    pub name: String,
    /// Qualified name.
    pub qualified_name: String,
    /// Kind label.
    pub kind: &'static str,
    /// Visibility label, if there is one.
    pub visibility: Option<&'static str>,
    /// Whether the symbol is static.
    pub is_static: bool,
    /// Path of the declaring file.
    pub path: String,
    /// First line, counted from 1.
    pub first_line: u32,
    /// Last line, counted from 1.
    pub last_line: u32,
    /// Signature as written.
    pub signature: Option<String>,
    /// Documentation comment.
    pub doc: Option<String>,
    /// Enclosing symbols, outermost first.
    pub breadcrumb: Vec<Crumb>,
    /// Measured values.
    pub metrics: Vec<MetricRow>,
    /// Member table in declaration order.
    pub members: Vec<MemberOut>,
}

/// The input of `explorer.symbol`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct SymbolInput {
    /// Id of the project.
    pub project: u64,
    /// Id of the symbol.
    pub symbol: u64,
}

fn member_out(row: &MemberRow, thresholds: &MetricThresholds) -> MemberOut {
    MemberOut {
        id: row.id.get(),
        name: row.name.to_string(),
        kind: row.kind.label(),
        visibility: row.visibility.map(Visibility::label),
        signature: row.signature.clone(),
        complexity: row.complexity,
        complexity_rating: row
            .complexity
            .map(|value| rating::complexity(value, thresholds)),
        usages: row.usages,
        first_line: row.first_line + LINE_OFFSET,
        last_line: row.last_line + LINE_OFFSET,
    }
}

fn crumb(detail: &SymbolDetail) -> Crumb {
    Crumb {
        id: detail.symbol.id.get(),
        name: detail.symbol.name.to_string(),
        kind: detail.symbol.kind.label(),
    }
}

impl ExplorerPorts {
    pub(super) fn lookup(
        &self,
        snapshot: SnapshotId,
        id: SymbolId,
    ) -> Result<SymbolDetail, CodedError> {
        self.index
            .symbol(snapshot, &SymbolKey::Id(id))?
            .ok_or_else(|| CodedError::new(ErrorCode::SymbolNotFound))
    }

    /// The enclosing symbols of `detail`, outermost first, at most `BREADCRUMB_DEPTH` of them.
    fn breadcrumb(
        &self,
        snapshot: SnapshotId,
        detail: &SymbolDetail,
    ) -> Result<Vec<Crumb>, CodedError> {
        let mut crumbs = Vec::new();
        let mut parent = detail.symbol.parent;
        while let Some(id) = parent
            && crumbs.len() < BREADCRUMB_DEPTH
        {
            let enclosing = self.lookup(snapshot, id)?;
            parent = enclosing.symbol.parent;
            crumbs.push(crumb(&enclosing));
        }
        crumbs.reverse();
        Ok(crumbs)
    }
}

fn answer(
    snapshot: SnapshotId,
    detail: &SymbolDetail,
    breadcrumb: Vec<Crumb>,
    rows: &[MemberRow],
    thresholds: &MetricThresholds,
) -> SymbolOut {
    let symbol = &detail.symbol;
    SymbolOut {
        snapshot: snapshot.get(),
        id: symbol.id.get(),
        name: symbol.name.to_string(),
        qualified_name: symbol.qualified_name.to_string(),
        kind: symbol.kind.label(),
        visibility: symbol.visibility.map(Visibility::label),
        is_static: symbol.is_static,
        path: detail.path.to_string(),
        first_line: symbol.span.start().line + LINE_OFFSET,
        last_line: symbol.span.end().line + LINE_OFFSET,
        signature: symbol.signature.clone(),
        doc: symbol.doc.clone(),
        breadcrumb,
        metrics: detail
            .metrics
            .iter()
            .map(|metric| MetricRow {
                kind: metric.kind.label(),
                value: metric.value.get(),
                rating: rating::metric(metric.kind, metric.value.get(), thresholds),
            })
            .collect(),
        members: rows.iter().map(|row| member_out(row, thresholds)).collect(),
    }
}

/// One symbol of the newest snapshot with breadcrumb, metrics and member table.
pub struct ExplorerSymbol {
    pub(super) ports: Arc<ExplorerPorts>,
}

impl Command for ExplorerSymbol {
    type Input = SymbolInput;
    type Output = SymbolOut;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(EXPLORER_SYMBOL),
        title_key: SYMBOL_TITLE,
        description_key: SYMBOL_DESCRIPTION,
        shortcut: None,
        read_only: true,
        permission: Permission::Read,
    };

    fn execute(&self, request: Request<'_, SymbolInput>) -> Result<SymbolOut, CodedError> {
        access::require(request.principal, Permission::Read)?;
        let ports = &self.ports;
        let snapshot = ports.newest(request.input.project)?.id;
        let id = SymbolId::new(request.input.symbol);
        let detail = ports.lookup(snapshot, id)?;
        let breadcrumb = ports.breadcrumb(snapshot, &detail)?;
        let rows = ports.outline.member_rows(snapshot, id)?;
        Ok(answer(
            snapshot,
            &detail,
            breadcrumb,
            &rows,
            &ports.thresholds,
        ))
    }
}
