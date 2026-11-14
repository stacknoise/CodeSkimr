//! `find_constants`.

use codeskimr_application::access;
use codeskimr_application::ports::{KeyKind, SearchHit, SymbolKey};
use codeskimr_domain::{Permission, Principal, Project, ProjectSource, SymbolKind};
use serde::Serialize;
use serde_json::Value;

use super::{AppliedLimit, SnapshotInfo, Tools, to_json, truncation_notice};
use crate::args::ConstantsArgs;
use crate::consts::{ARG_QUERY, KIND_CONFIG, KIND_I18N, LINE_OFFSET, READ_TOOL_PERMISSION};
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
struct Key {
    name: String,
    /// `config` for a key of a TOML file, `i18n` for the id of a translation message.
    kind: &'static str,
    path: String,
    /// One-based.
    line: u32,
}

#[derive(Serialize)]
struct Answer {
    snapshot: SnapshotInfo,
    constants: Vec<Constant>,
    /// Configuration and translation keys whose name contains the query.
    keys: Vec<Key>,
    /// Whether the key files were looked at: the caller needs `read_source` and the project must
    /// be a folder on this machine.
    keys_searched: bool,
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
        let may_read_source = access::require(principal, Permission::ReadSource).is_ok();
        let (keys, keys_cut) =
            self.matching_keys(may_read_source, &scope.project, &args.query, &limit)?;
        to_json(&Answer {
            snapshot: scope.info,
            constants,
            keys_searched: keys.is_some(),
            keys: keys.unwrap_or_default(),
            notice: (cut || keys_cut).then(|| truncation_notice(limit.value, None)),
        })
    }

    /// The keys of the project folder that contain `query` (any case), cut to the limit; `None`
    /// if they cannot be looked at: no `read_source`, or a project that is not a local folder.
    fn matching_keys(
        &self,
        may_read_source: bool,
        project: &Project,
        query: &str,
        limit: &AppliedLimit,
    ) -> Result<(Option<Vec<Key>>, bool), McpError> {
        let ProjectSource::Folder(root) = &project.source else {
            return Ok((None, false));
        };
        if !may_read_source {
            return Ok((None, false));
        }
        let wanted = query.to_lowercase();
        let all = self.project_keys().keys(root, self.key_sources())?;
        let mut hits: Vec<Key> = all
            .into_iter()
            .filter(|entry| entry.name.to_lowercase().contains(&wanted))
            .map(|entry| Key {
                name: entry.name,
                kind: match entry.kind {
                    KeyKind::Config => KIND_CONFIG,
                    KeyKind::I18n => KIND_I18N,
                },
                path: entry.path.as_str().to_owned(),
                line: entry.line + LINE_OFFSET,
            })
            .collect();
        let cut = hits.len() > usize::try_from(limit.value).unwrap_or(usize::MAX);
        hits.truncate(usize::try_from(limit.value).unwrap_or(usize::MAX));
        Ok((Some(hits), cut))
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
