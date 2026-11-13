//! `find_usages` and `impact_of_change`.

use std::collections::BTreeMap;

use codeskimr_application::access;
use codeskimr_application::ports::{ImpactedSymbol, SymbolLabel, UsageEntry};
use codeskimr_domain::{Direction, Principal, Project, Symbol, UsageQuery, Visibility};
use serde::Serialize;
use serde_json::Value;

use super::{SnapshotInfo, Tools, layer_of, parse_key, to_json, truncation_notice};
use crate::args::{ImpactArgs, UsagesArgs};
use crate::consts::{
    ARG_DIRECTION, COLUMN_OFFSET, DIRECTION_INCOMING, DIRECTION_OUTGOING, LINE_OFFSET, PARAM_DEPTH,
    READ_TOOL_PERMISSION,
};
use crate::error::{CodeReport, McpError};

/// A symbol as one line of an answer.
#[derive(Serialize)]
struct Place {
    id: u64,
    qualified_name: String,
    kind: &'static str,
}

fn place_of(symbol: &Symbol) -> Place {
    Place {
        id: symbol.id.get(),
        qualified_name: symbol.qualified_name.as_str().to_owned(),
        kind: symbol.kind.label(),
    }
}

fn place(label: &SymbolLabel) -> Place {
    Place {
        id: label.id.get(),
        qualified_name: label.qualified_name.as_str().to_owned(),
        kind: label.kind.label(),
    }
}

#[derive(Serialize)]
struct Usage {
    depth: u32,
    kind: &'static str,
    /// `certain` or `probable`; a probable usage is the resolver's best guess.
    confidence: &'static str,
    path: String,
    line: u32,
    column: u32,
    from: Place,
    to: Place,
}

fn usage(entry: &UsageEntry) -> Usage {
    let start = entry.reference.span.start();
    Usage {
        depth: entry.depth,
        kind: entry.reference.kind.label(),
        confidence: entry.reference.confidence.label(),
        path: entry.path.as_str().to_owned(),
        line: start.line + LINE_OFFSET,
        column: start.column + COLUMN_OFFSET,
        from: place(&entry.from),
        to: place(&entry.to),
    }
}

#[derive(Serialize)]
struct UsagesAnswer {
    snapshot: SnapshotInfo,
    symbol: Place,
    direction: &'static str,
    depth: u32,
    include_tests: bool,
    total: usize,
    usages: Vec<Usage>,
    notice: Option<CodeReport>,
}

fn parse_direction(text: Option<&str>) -> Result<Direction, McpError> {
    match text {
        None | Some(DIRECTION_INCOMING) => Ok(Direction::Incoming),
        Some(DIRECTION_OUTGOING) => Ok(Direction::Outgoing),
        Some(_) => Err(McpError::ArgumentsInvalid(ARG_DIRECTION.to_owned())),
    }
}

const fn direction_label(direction: Direction) -> &'static str {
    match direction {
        Direction::Incoming => DIRECTION_INCOMING,
        Direction::Outgoing => DIRECTION_OUTGOING,
    }
}

/// The notice that tells a caller its request was cut or reduced, if it was.
fn cut_notice(
    limit: u32,
    limit_clamped: bool,
    shown: usize,
    total: usize,
    depth_clamped: Option<u32>,
) -> Option<CodeReport> {
    if !(limit_clamped || depth_clamped.is_some() || total > shown) {
        return None;
    }
    let mut report = truncation_notice(limit, Some(total));
    if let Some(depth) = depth_clamped {
        report.params.insert(PARAM_DEPTH, depth.to_string());
    }
    Some(report)
}

#[derive(Serialize)]
struct Affected {
    id: u64,
    qualified_name: String,
    kind: &'static str,
    depth: u32,
    path: String,
    visibility: Option<&'static str>,
    /// Whether the symbol is public and so part of an interface others may build on.
    public_interface: bool,
    layer: Option<String>,
}

fn affected(project: &Project, symbol: &ImpactedSymbol) -> Affected {
    Affected {
        id: symbol.label.id.get(),
        qualified_name: symbol.label.qualified_name.as_str().to_owned(),
        kind: symbol.label.kind.label(),
        depth: symbol.depth,
        path: symbol.path.as_str().to_owned(),
        visibility: symbol.visibility.map(Visibility::label),
        public_interface: symbol.visibility == Some(Visibility::Public),
        layer: layer_of(project, &symbol.label.qualified_name),
    }
}

/// How many of the listed symbols each layer owns.
#[derive(Serialize)]
struct LayerCount {
    layer: String,
    count: usize,
}

fn count_layers(symbols: &[Affected]) -> Vec<LayerCount> {
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for layer in symbols.iter().filter_map(|symbol| symbol.layer.as_deref()) {
        *counts.entry(layer).or_default() += 1;
    }
    counts
        .into_iter()
        .map(|(layer, count)| LayerCount {
            layer: layer.to_owned(),
            count,
        })
        .collect()
}

#[derive(Serialize)]
struct ImpactAnswer {
    snapshot: SnapshotInfo,
    symbol: Place,
    depth: u32,
    total: usize,
    /// Layers of the listed symbols, with the number of symbols in each.
    layers: Vec<LayerCount>,
    public_interfaces: usize,
    symbols: Vec<Affected>,
    notice: Option<CodeReport>,
}

impl Tools {
    /// The references that use a symbol or that it uses, nearest step first.
    ///
    /// # Errors
    /// `AUTH-401` if the principal may not read, `MCP-101` for an invalid symbol, direction or
    /// depth, `MCP-202` if the symbol does not exist, `MCP-201` and `MCP-203` for the project,
    /// or the code of a failing port.
    pub fn find_usages(&self, principal: &Principal, args: &UsagesArgs) -> Result<Value, McpError> {
        access::require(principal, READ_TOOL_PERMISSION)?;
        parse_key(&args.symbol)?;
        let direction = parse_direction(args.direction.as_deref())?;
        let depth = self.apply_depth(args.depth)?;
        let scope = self.scope(args.project)?;
        let limit = self.apply_limit(args.limit);
        let found = self.find_symbol(&scope, &args.symbol)?;
        let query = UsageQuery {
            symbol: found.symbol.id,
            direction,
            depth: depth.value,
            include_tests: args.include_tests.unwrap_or(false),
        };
        let page = self
            .usage_index()
            .usages(scope.snapshot.id, &query, limit.value)?;
        let notice = cut_notice(
            limit.value,
            limit.clamped,
            page.entries.len(),
            page.total,
            depth.clamped.then_some(depth.value),
        );
        to_json(&UsagesAnswer {
            snapshot: scope.info,
            symbol: place_of(&found.symbol),
            direction: direction_label(direction),
            depth: depth.value,
            include_tests: query.include_tests,
            total: page.total,
            usages: page.entries.iter().map(usage).collect(),
            notice,
        })
    }

    /// The symbols a change of one symbol can reach, grouped by layer, with the public
    /// interfaces among them marked. References from tests count; the walk goes as deep as
    /// `mcp.max_depth` allows.
    ///
    /// # Errors
    /// `AUTH-401` if the principal may not read, `MCP-101` for an invalid symbol, `MCP-202`
    /// if it does not exist, `MCP-201` and `MCP-203` for the project, or the code of a failing
    /// port.
    pub fn impact_of_change(
        &self,
        principal: &Principal,
        args: &ImpactArgs,
    ) -> Result<Value, McpError> {
        access::require(principal, READ_TOOL_PERMISSION)?;
        parse_key(&args.symbol)?;
        let scope = self.scope(args.project)?;
        let limit = self.apply_limit(args.limit);
        let found = self.find_symbol(&scope, &args.symbol)?;
        let page = self.usage_index().impact(
            scope.snapshot.id,
            found.symbol.id,
            self.max_depth(),
            limit.value,
        )?;
        let symbols: Vec<Affected> = page
            .symbols
            .iter()
            .map(|symbol| affected(&scope.project, symbol))
            .collect();
        let notice = cut_notice(limit.value, limit.clamped, symbols.len(), page.total, None);
        to_json(&ImpactAnswer {
            snapshot: scope.info,
            symbol: place_of(&found.symbol),
            depth: self.max_depth(),
            total: page.total,
            layers: count_layers(&symbols),
            public_interfaces: symbols.iter().filter(|s| s.public_interface).count(),
            symbols,
            notice,
        })
    }
}
