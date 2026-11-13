//! `search_symbols`.

use std::str::FromStr;

use codeskimr_domain::SymbolKind;
use serde::Serialize;
use serde_json::Value;

use super::{SnapshotInfo, Tools, to_json, truncation_notice};
use crate::args::SearchArgs;
use crate::consts::{ARG_KIND, ARG_QUERY};
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
    /// `MCP-101` for an empty query or an unknown kind, `MCP-201` and `MCP-203` for the project,
    /// or the code of a failing port.
    pub fn search_symbols(&self, args: &SearchArgs) -> Result<Value, McpError> {
        if args.query.trim().is_empty() {
            return Err(invalid(ARG_QUERY));
        }
        let kind = parse_kind(args.kind.as_deref())?;
        let scope = self.scope(args.project)?;
        let limit = self.apply_limit(args.limit);
        // One more than asked tells whether results were left out. A kind filter is applied
        // after the search, so look at the widest result first.
        let wanted = usize::try_from(limit.value).unwrap_or(usize::MAX);
        let fetch = if kind.is_some() {
            self.max_limit()
        } else {
            limit.value
        };
        let found = self
            .index()
            .search(scope.snapshot.id, &args.query, fetch.saturating_add(1))?;
        let mut hits: Vec<Hit> = found
            .into_iter()
            .filter(|hit| kind.is_none_or(|wanted| hit.kind == wanted))
            .map(|hit| Hit {
                id: hit.symbol.get(),
                name: hit.name.as_str().to_owned(),
                qualified_name: hit.qualified_name.as_str().to_owned(),
                kind: hit.kind.label(),
                path: hit.path.as_str().to_owned(),
            })
            .collect();
        let cut = limit.clamped || hits.len() > wanted;
        hits.truncate(wanted);
        to_json(&Answer {
            snapshot: scope.info,
            hits,
            notice: cut.then(|| truncation_notice(limit.value, None)),
        })
    }
}
