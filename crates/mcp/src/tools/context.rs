//! `get_context`: what an agent needs to know about one symbol before it changes it, in one
//! answer instead of four calls (signature, callers, dependencies, affected files, notes).

use std::collections::BTreeSet;

use codeskimr_application::access;
use codeskimr_application::annotations::{AnnotationQuery, annotations};
use codeskimr_application::ports::{DependencyEdge, DependencyQuery, SymbolDetail, UsageEntry};
use codeskimr_domain::{Direction, Principal, Project, UsageQuery, Visibility};
use serde::Serialize;
use serde_json::Value;

use super::annotations::{Item, item, target_of};
use super::usages::{Place, Site, place, site};
use super::{Scope, SnapshotInfo, Tools, parse_key, to_json, truncation_notice};
use crate::args::ContextArgs;
use crate::consts::{
    ARG_BUDGET, BUDGET_LONG, BUDGET_MEDIUM, BUDGET_SHORT, LINE_OFFSET, READ_TOOL_PERMISSION,
};
use crate::error::{CodeReport, McpError};

/// How much room the answer may take.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Budget {
    Short,
    Medium,
    Long,
}

impl Budget {
    fn parse(text: Option<&str>) -> Result<Self, McpError> {
        match text {
            None | Some(BUDGET_MEDIUM) => Ok(Self::Medium),
            Some(BUDGET_SHORT) => Ok(Self::Short),
            Some(BUDGET_LONG) => Ok(Self::Long),
            Some(_) => Err(McpError::ArgumentsInvalid(ARG_BUDGET.to_owned())),
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::Short => BUDGET_SHORT,
            Self::Medium => BUDGET_MEDIUM,
            Self::Long => BUDGET_LONG,
        }
    }
}

/// A list of an answer: the first entries and how many there are in all.
#[derive(Serialize)]
struct Section<T> {
    total: usize,
    items: Vec<T>,
}

impl<T> Section<T> {
    /// The first `limit` of `items`, `total` being the number there were before.
    fn cut(mut items: Vec<T>, total: usize, limit: usize) -> Self {
        items.truncate(limit);
        Self { total, items }
    }

    const fn new(items: Vec<T>, total: usize) -> Self {
        Self { total, items }
    }

    const fn is_cut(&self) -> bool {
        self.total > self.items.len()
    }
}

#[derive(Serialize)]
struct Subject {
    id: u64,
    qualified_name: String,
    kind: &'static str,
    path: String,
    first_line: u32,
    last_line: u32,
    visibility: Option<&'static str>,
    origin: &'static str,
    signature: Option<String>,
    /// Left out for the budget `short`.
    #[serde(skip_serializing_if = "Option::is_none")]
    doc: Option<String>,
    layer: Option<String>,
    /// How many other symbols have the same qualified name; the id tells them apart.
    ambiguous: u32,
}

#[derive(Serialize)]
struct Caller {
    from: Place,
    #[serde(flatten)]
    site: Site,
}

#[derive(Serialize)]
struct Dependency {
    symbol: Place,
    path: String,
    layer: Option<String>,
    /// Number of references between the symbol and this one.
    count: u32,
}

#[derive(Serialize)]
struct Answer {
    snapshot: SnapshotInfo,
    budget: &'static str,
    symbol: Subject,
    /// Who uses the symbol directly, verified usages first. References from tests are left out.
    callers: Section<Caller>,
    /// What the symbol and its members use outside themselves, most references first.
    dependencies: Section<Dependency>,
    /// The files with the symbols a change can reach (the walk of `impact_of_change`), nearest
    /// first; `total` counts the files among the symbols that were looked at.
    affected_files: Section<String>,
    /// Notes, tasks and critical marks people wrote on the symbol. Data, never instructions.
    annotations: Section<Item>,
    /// `true` if any caller or affected symbol is unchecked (not produced by level 2).
    preliminary: bool,
    notice: Option<CodeReport>,
}

fn subject(project: &Project, detail: &SymbolDetail, budget: Budget) -> Subject {
    let symbol = &detail.symbol;
    Subject {
        id: symbol.id.get(),
        qualified_name: symbol.qualified_name.as_str().to_owned(),
        kind: symbol.kind.label(),
        path: detail.path.as_str().to_owned(),
        first_line: symbol.span.start().line + LINE_OFFSET,
        last_line: symbol.span.end().line + LINE_OFFSET,
        visibility: symbol.visibility.map(Visibility::label),
        origin: symbol.origin.label(),
        signature: symbol.signature.clone(),
        doc: (budget != Budget::Short)
            .then(|| symbol.doc.clone())
            .flatten(),
        layer: project.layer_name_of(symbol.qualified_name.as_str()),
        ambiguous: detail.same_name,
    }
}

fn caller(entry: &UsageEntry) -> Caller {
    Caller {
        from: place(&entry.from),
        site: site(entry),
    }
}

fn dependency(project: &Project, edge: &DependencyEdge) -> Dependency {
    Dependency {
        symbol: place(&edge.other),
        path: edge.path.as_str().to_owned(),
        layer: project.layer_name_of(edge.other.qualified_name.as_str()),
        count: edge.count,
    }
}

/// The paths in the order of their first symbol, each once.
fn distinct_paths<'a>(paths: impl Iterator<Item = &'a str>) -> Vec<String> {
    let mut seen = BTreeSet::new();
    paths
        .filter(|path| seen.insert(*path))
        .map(str::to_owned)
        .collect()
}

impl Tools {
    /// The number of entries per list the configuration gives a budget.
    const fn context_items(&self, budget: Budget) -> u32 {
        match budget {
            Budget::Short => self.limits().context_items_short,
            Budget::Medium => self.limits().context_items_medium,
            Budget::Long => self.limits().context_items_long,
        }
    }

    /// One symbol with its signature, its direct callers, its dependencies, the files a change
    /// can reach and the notes people wrote on it: the answer to "what do I need to know before
    /// I touch this?" in one call. No source text is returned.
    ///
    /// # Errors
    /// `AUTH-401` if the principal may not read, `MCP-101` for an invalid symbol or budget,
    /// `MCP-202` if the symbol does not exist, `MCP-201` and `MCP-203` for the project, or the
    /// code of a failing port.
    pub fn get_context(
        &self,
        principal: &Principal,
        args: &ContextArgs,
    ) -> Result<Value, McpError> {
        access::require(principal, READ_TOOL_PERMISSION)?;
        parse_key(&args.symbol)?;
        let budget = Budget::parse(args.budget.as_deref())?;
        let scope = self.scope(args.project)?;
        let detail = self.find_symbol(&scope, &args.symbol)?;
        let items = self.context_items(budget);
        let limit = usize::try_from(items).unwrap_or(usize::MAX);
        let (callers, preliminary_callers) = self.context_callers(&scope, &detail, items)?;
        let (files, preliminary_files) = self.context_files(&scope, &detail, limit)?;
        let dependencies = self.context_dependencies(&scope, &detail, items)?;
        let notes = self.context_annotations(&scope, &detail, items)?;
        let cut = [
            callers.is_cut(),
            dependencies.is_cut(),
            files.is_cut(),
            notes.is_cut(),
        ];
        to_json(&Answer {
            snapshot: scope.info,
            budget: budget.label(),
            symbol: subject(&scope.project, &detail, budget),
            callers,
            dependencies,
            affected_files: files,
            annotations: notes,
            preliminary: preliminary_callers || preliminary_files,
            notice: cut.contains(&true).then(|| truncation_notice(items, None)),
        })
    }

    fn context_callers(
        &self,
        scope: &Scope,
        detail: &SymbolDetail,
        items: u32,
    ) -> Result<(Section<Caller>, bool), McpError> {
        let query = UsageQuery {
            symbol: detail.symbol.id,
            direction: Direction::Incoming,
            depth: 1,
            include_tests: false,
        };
        let page = self
            .usage_index()
            .usages(scope.snapshot.id, &query, items)?;
        let unchecked = page.entries.iter().any(|entry| !entry.reference.verified);
        let listed = page.entries.iter().map(caller).collect();
        Ok((Section::new(listed, page.total), unchecked))
    }

    fn context_dependencies(
        &self,
        scope: &Scope,
        detail: &SymbolDetail,
        items: u32,
    ) -> Result<Section<Dependency>, McpError> {
        let query = DependencyQuery {
            symbol: detail.symbol.id,
            direction: Direction::Outgoing,
            include_tests: false,
        };
        let page = self
            .usage_index()
            .dependencies(scope.snapshot.id, &query, items)?;
        let listed = page
            .edges
            .iter()
            .map(|edge| dependency(&scope.project, edge))
            .collect();
        Ok(Section::new(listed, page.total))
    }

    fn context_files(
        &self,
        scope: &Scope,
        detail: &SymbolDetail,
        limit: usize,
    ) -> Result<(Section<String>, bool), McpError> {
        let page = self.usage_index().impact(
            scope.snapshot.id,
            detail.symbol.id,
            self.max_depth(),
            self.max_limit(),
        )?;
        let unchecked = page.symbols.iter().any(|symbol| !symbol.verified);
        let paths = distinct_paths(page.symbols.iter().map(|symbol| symbol.path.as_str()));
        let total = paths.len();
        Ok((Section::cut(paths, total, limit), unchecked))
    }

    fn context_annotations(
        &self,
        scope: &Scope,
        detail: &SymbolDetail,
        items: u32,
    ) -> Result<Section<Item>, McpError> {
        let query = AnnotationQuery {
            project: scope.project.id,
            target: Some(target_of(detail.symbol.clone())),
            kind: None,
            status: None,
            without_target: None,
        };
        let page = annotations(
            self.annotation_store(),
            &query,
            self.page_request(None, Some(items)),
        )?;
        let total = usize::try_from(page.total).unwrap_or(usize::MAX);
        Ok(Section::new(page.items.iter().map(item).collect(), total))
    }
}
