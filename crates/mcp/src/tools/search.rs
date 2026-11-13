//! `search_symbols`.

use std::str::FromStr;

use codeskimr_application::access;
use codeskimr_application::ports::SearchHit;
use codeskimr_domain::{Principal, SymbolKind};
use serde::Serialize;
use serde_json::Value;

use super::{AppliedLimit, Scope, SnapshotInfo, Tools, to_json, truncation_notice};
use crate::args::SearchArgs;
use crate::consts::{ARG_KIND, ARG_QUERY, READ_TOOL_PERMISSION};
use crate::error::{CodeReport, McpError};

#[derive(Serialize)]
struct Hit {
    id: u64,
    name: String,
    qualified_name: String,
    kind: &'static str,
    path: String,
}

#[derive(Serialize)]
struct Answer {
    snapshot: SnapshotInfo,
    hits: Vec<Hit>,
    notice: Option<CodeReport>,
}

fn invalid(detail: &str) -> McpError {
    McpError::ArgumentsInvalid(detail.to_owned())
}

fn parse_kind(kind: Option<&str>) -> Result<Option<SymbolKind>, McpError> {
    kind.map(|text| SymbolKind::from_str(text).map_err(|_| invalid(ARG_KIND)))
        .transpose()
}

impl Tools {
    /// Fuzzy search over symbol names, best match first.
    ///
    /// # Errors
    /// `AUTH-401` if the principal may not read, `MCP-101` for an empty query or an unknown
    /// kind, `MCP-201` and `MCP-203` for the project, or the code of a failing port.
    pub fn search_symbols(
        &self,
        principal: &Principal,
        args: &SearchArgs,
    ) -> Result<Value, McpError> {
        access::require(principal, READ_TOOL_PERMISSION)?;
        if args.query.trim().is_empty() {
            return Err(invalid(ARG_QUERY));
        }
        let kind = parse_kind(args.kind.as_deref())?;
        let scope = self.scope(args.project)?;
        let limit = self.apply_limit(args.limit);
        let (found, cut) = self.search_of_kind(&scope, &args.query, kind, &limit)?;
        let hits: Vec<Hit> = found
            .into_iter()
            .map(|hit| Hit {
                id: hit.symbol.get(),
                name: hit.name.as_str().to_owned(),
                qualified_name: hit.qualified_name.as_str().to_owned(),
                kind: hit.kind.label(),
                path: hit.path.as_str().to_owned(),
            })
            .collect();
        to_json(&Answer {
            snapshot: scope.info,
            hits,
            notice: cut.then(|| truncation_notice(limit.value, None)),
        })
    }

    /// The best hits of a name search, restricted to one kind if given, at most the limit.
    /// The flag tells whether results were left out. A kind filter is applied after the
    /// search, so the widest allowed result is looked at first.
    pub(crate) fn search_of_kind(
        &self,
        scope: &Scope,
        query: &str,
        kind: Option<SymbolKind>,
        limit: &AppliedLimit,
    ) -> Result<(Vec<SearchHit>, bool), McpError> {
        let wanted = usize::try_from(limit.value).unwrap_or(usize::MAX);
        let fetch = if kind.is_some() {
            self.max_limit()
        } else {
            limit.value
        };
        let mut hits = self
            .index()
            .search(scope.snapshot.id, query, fetch.saturating_add(1))?;
        hits.retain(|hit| kind.is_none_or(|wanted| hit.kind == wanted));
        let cut = limit.clamped || hits.len() > wanted;
        hits.truncate(wanted);
        Ok((hits, cut))
    }
}
