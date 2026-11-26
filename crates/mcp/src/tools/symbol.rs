//! `get_symbol`.

use std::collections::BTreeMap;

use codeskimr_application::access;
use codeskimr_application::ports::{MemberRef, SymbolDetail};
use codeskimr_domain::{Principal, Project, Visibility};
use serde::Serialize;
use serde_json::Value;

use super::{SnapshotInfo, Tools, parse_key, to_json, truncation_notice};
use crate::args::SymbolArgs;
use crate::consts::{LINE_OFFSET, READ_TOOL_PERMISSION};
use crate::error::{CodeReport, McpError};

#[derive(Serialize)]
struct Member {
    id: u64,
    name: String,
    kind: &'static str,
}

/// Where a symbol is and what it is, as `get_symbol` and `get_context` both say it.
#[derive(Serialize)]
pub(super) struct Header {
    qualified_name: String,
    kind: &'static str,
    path: String,
    first_line: u32,
    last_line: u32,
    visibility: Option<&'static str>,
}

pub(super) fn header(detail: &SymbolDetail) -> Header {
    let symbol = &detail.symbol;
    Header {
        qualified_name: symbol.qualified_name.as_str().to_owned(),
        kind: symbol.kind.label(),
        path: detail.path.as_str().to_owned(),
        first_line: symbol.span.start().line + LINE_OFFSET,
        last_line: symbol.span.end().line + LINE_OFFSET,
        visibility: symbol.visibility.map(Visibility::label),
    }
}

#[derive(Serialize)]
struct Answer {
    snapshot: SnapshotInfo,
    id: u64,
    name: String,
    #[serde(flatten)]
    header: Header,
    is_static: bool,
    /// `syntax` (level 1) or `index` (created by level 2, ADR 0055).
    origin: &'static str,
    signature: Option<String>,
    doc: Option<String>,
    layer: Option<String>,
    metrics: BTreeMap<&'static str, f64>,
    member_count: usize,
    members: Vec<Member>,
    /// How many other symbols have the same qualified name; the id tells them apart (ADR 0036).
    ambiguous: u32,
    notice: Option<CodeReport>,
}

fn member(member: &MemberRef) -> Member {
    Member {
        id: member.id.get(),
        name: member.name.as_str().to_owned(),
        kind: member.kind.label(),
    }
}

fn answer(
    scope_info: SnapshotInfo,
    project: &Project,
    detail: &SymbolDetail,
    members: Vec<Member>,
    notice: Option<CodeReport>,
) -> Answer {
    let symbol = &detail.symbol;
    Answer {
        snapshot: scope_info,
        id: symbol.id.get(),
        name: symbol.name.as_str().to_owned(),
        header: header(detail),
        is_static: symbol.is_static,
        origin: symbol.origin.label(),
        signature: symbol.signature.clone(),
        doc: symbol.doc.clone(),
        layer: project.layer_name_of(symbol.qualified_name.as_str()),
        metrics: detail
            .metrics
            .iter()
            .map(|metric| (metric.kind.label(), metric.value.get()))
            .collect(),
        member_count: detail.members.len(),
        members,
        ambiguous: detail.same_name,
        notice,
    }
}

impl Tools {
    /// One symbol with signature, documentation, members, metrics and layer.
    ///
    /// # Errors
    /// `AUTH-401` if the principal may not read, `MCP-101` for an invalid symbol, `MCP-202`
    /// if it does not exist, `MCP-201` and `MCP-203` for the project, or the code of a
    /// failing port.
    pub fn get_symbol(&self, principal: &Principal, args: &SymbolArgs) -> Result<Value, McpError> {
        access::require(principal, READ_TOOL_PERMISSION)?;
        // Validate the argument before the project is looked up, as before.
        parse_key(&args.symbol)?;
        let scope = self.scope(args.project)?;
        let detail = self.find_symbol(&scope, &args.symbol)?;
        let limit = usize::try_from(self.max_limit()).unwrap_or(usize::MAX);
        let members = detail
            .members
            .iter()
            .take(limit)
            .map(member)
            .collect::<Vec<_>>();
        let notice = (detail.members.len() > members.len())
            .then(|| truncation_notice(self.max_limit(), Some(detail.members.len())));
        to_json(&answer(
            scope.info,
            &scope.project,
            &detail,
            members,
            notice,
        ))
    }
}
