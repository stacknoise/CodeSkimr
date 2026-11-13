//! `find_constants`.

use codeskimr_application::access;
use codeskimr_application::ports::{SearchHit, SymbolKey};
use codeskimr_domain::{Principal, SymbolKind};
use serde::Serialize;
use serde_json::Value;

use super::{SnapshotInfo, Tools, to_json, truncation_notice};
use crate::args::ConstantsArgs;
use crate::consts::{ARG_QUERY, READ_TOOL_PERMISSION};
use crate::error::{CodeReport, McpError};

#[derive(Serialize)]
struct Constant {
    id: u64,
    name: String,
    qualified_name: String,
    path: String,
    /// The declaration as written, which usually shows the value, if the language pack found it.
    signature: Option<String>,
}

#[derive(Serialize)]
struct Answer {
    snapshot: SnapshotInfo,
    constants: Vec<Constant>,
    notice: Option<CodeReport>,
}

impl Tools {
    /// Finds existing constants by name, so that a value is declared once.
    ///
    /// # Errors
    /// `AUTH-401` if the principal may not read, `MCP-101` for an empty query, `MCP-201` and
    /// `MCP-203` for the project, or the code of a failing port.
    pub fn find_constants(
        &self,
        principal: &Principal,
        args: &ConstantsArgs,
    ) -> Result<Value, McpError> {
        access::require(principal, READ_TOOL_PERMISSION)?;
        if args.query.trim().is_empty() {
            return Err(McpError::ArgumentsInvalid(ARG_QUERY.to_owned()));
        }
        let scope = self.scope(args.project)?;
        let limit = self.apply_limit(args.limit);
        let (found, cut) =
            self.search_of_kind(&scope, &args.query, Some(SymbolKind::Constant), &limit)?;
        let constants = found
            .iter()
            .map(|hit| self.constant(&scope, hit))
            .collect::<Result<Vec<_>, _>>()?;
        to_json(&Answer {
            snapshot: scope.info,
            constants,
            notice: cut.then(|| truncation_notice(limit.value, None)),
        })
    }

    fn constant(&self, scope: &super::Scope, hit: &SearchHit) -> Result<Constant, McpError> {
        let detail = self
            .index()
            .symbol(scope.snapshot.id, &SymbolKey::Id(hit.symbol))?;
        Ok(Constant {
            id: hit.symbol.get(),
            name: hit.name.as_str().to_owned(),
            qualified_name: hit.qualified_name.as_str().to_owned(),
            path: hit.path.as_str().to_owned(),
            signature: detail.and_then(|found| found.symbol.signature),
        })
    }
}
