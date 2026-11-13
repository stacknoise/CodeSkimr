//! `get_dependencies`.

use std::collections::BTreeMap;

use codeskimr_application::access;
use codeskimr_application::ports::{DependencyEdge, DependencyQuery};
use codeskimr_domain::{Direction, LayerEdge, Principal, Project};
use serde::Serialize;
use serde_json::Value;

use super::usages::{cut_notice, direction_label, parse_direction};
use super::{Scope, SnapshotInfo, Tools, layer_of, to_json};
use crate::args::DependenciesArgs;
use crate::consts::{ARG_LAYER, ARG_SYMBOL_OR_LAYER, READ_TOOL_PERMISSION};
use crate::error::{CodeReport, McpError};

/// What the caller asked about.
#[derive(Serialize)]
struct Target {
    /// `symbol` or `layer`.
    kind: &'static str,
    name: String,
}

/// The dependencies between the target's layer (or the target layer itself) and one other layer.
#[derive(Serialize)]
struct LayerLink {
    layer: String,
    /// Number of references behind the link.
    count: u32,
    /// Whether the link breaks the declared layering.
    violates_layering: bool,
    in_cycle: bool,
}

/// One symbol on the far side of a dependency.
#[derive(Serialize)]
struct Dependency {
    id: u64,
    qualified_name: String,
    kind: &'static str,
    path: String,
    layer: Option<String>,
    /// Number of references between the target and this symbol.
    count: u32,
    violates_layering: bool,
}

#[derive(Serialize)]
struct Answer {
    snapshot: SnapshotInfo,
    target: Target,
    direction: &'static str,
    /// Number of dependencies, also those that were cut from the list.
    total: usize,
    /// How many of them break the layering.
    violations: usize,
    layers: Vec<LayerLink>,
    dependencies: Vec<Dependency>,
    notice: Option<CodeReport>,
}

/// The layer edge from `from` to `to`, if the scan recorded one.
fn edge_between<'a>(edges: &'a [LayerEdge], from: &str, to: &str) -> Option<&'a LayerEdge> {
    edges
        .iter()
        .find(|edge| edge.from.as_str() == from && edge.to.as_str() == to)
}

/// The two layers of a dependency in the order of the edge, if both are known and differ.
fn layer_pair<'a>(
    direction: Direction,
    own: Option<&'a str>,
    other: Option<&'a str>,
) -> Option<(&'a str, &'a str)> {
    let (own, other) = (own?, other?);
    if own == other {
        return None;
    }
    Some(match direction {
        Direction::Outgoing => (own, other),
        Direction::Incoming => (other, own),
    })
}

/// A dependency with its layer and whether it breaks the layering.
fn dependency(
    project: &Project,
    layers: &[LayerEdge],
    scope: (Direction, Option<&str>),
    edge: &DependencyEdge,
) -> Dependency {
    let layer = layer_of(project, &edge.other.qualified_name);
    let violates_layering = layer_pair(scope.0, scope.1, layer.as_deref())
        .and_then(|(from, to)| edge_between(layers, from, to))
        .is_some_and(|found| found.violates_layering);
    Dependency {
        id: edge.other.id.get(),
        qualified_name: edge.other.qualified_name.as_str().to_owned(),
        kind: edge.other.kind.label(),
        path: edge.path.as_str().to_owned(),
        layer,
        count: edge.count,
        violates_layering,
    }
}

/// The links to other layers that the listed dependencies add up to, in the order of the names.
fn links(
    direction: Direction,
    own: Option<&str>,
    layers: &[LayerEdge],
    found: &[Dependency],
) -> Vec<LayerLink> {
    let mut counts: BTreeMap<&str, u32> = BTreeMap::new();
    for dependency in found {
        if let Some(layer) = dependency.layer.as_deref() {
            *counts.entry(layer).or_default() += dependency.count;
        }
    }
    counts
        .into_iter()
        .filter_map(|(layer, count)| {
            let (from, to) = layer_pair(direction, own, Some(layer))?;
            let recorded = edge_between(layers, from, to);
            Some(LayerLink {
                layer: layer.to_owned(),
                count,
                violates_layering: recorded.is_some_and(|edge| edge.violates_layering),
                in_cycle: recorded.is_some_and(|edge| edge.in_cycle),
            })
        })
        .collect()
}

/// The links of one layer to the others, straight from the recorded layer graph.
fn layer_links(direction: Direction, layer: &str, edges: &[LayerEdge]) -> Vec<LayerLink> {
    edges
        .iter()
        .filter_map(|edge| {
            let (own, other) = match direction {
                Direction::Outgoing => (&edge.from, &edge.to),
                Direction::Incoming => (&edge.to, &edge.from),
            };
            (own.as_str() == layer).then(|| LayerLink {
                layer: other.as_str().to_owned(),
                count: edge.count,
                violates_layering: edge.violates_layering,
                in_cycle: edge.in_cycle,
            })
        })
        .collect()
}

/// What the caller wants the dependencies of.
enum Subject<'a> {
    Symbol(&'a str),
    Layer(&'a str),
}

/// A question about one symbol.
struct Ask<'a> {
    text: &'a str,
    direction: Direction,
    include_tests: bool,
}

fn subject(args: &DependenciesArgs) -> Result<Subject<'_>, McpError> {
    match (args.symbol.as_deref(), args.layer.as_deref()) {
        (Some(symbol), None) => Ok(Subject::Symbol(symbol)),
        (None, Some(layer)) => Ok(Subject::Layer(layer)),
        _ => Err(McpError::ArgumentsInvalid(ARG_SYMBOL_OR_LAYER.to_owned())),
    }
}

impl Tools {
    /// The dependencies of a symbol (with everything it contains, so also of a namespace or
    /// type) or of a layer: what it uses (`outgoing`, the default) or what uses it. Each
    /// dependency says whether it breaks the layering the project declares.
    ///
    /// # Errors
    /// `AUTH-401` if the principal may not read, `MCP-101` for invalid arguments or an unknown
    /// layer, `MCP-202` if the symbol does not exist, `MCP-201` and `MCP-203` for the project,
    /// or the code of a failing port.
    pub fn get_dependencies(
        &self,
        principal: &Principal,
        args: &DependenciesArgs,
    ) -> Result<Value, McpError> {
        access::require(principal, READ_TOOL_PERMISSION)?;
        let subject = subject(args)?;
        let direction = parse_direction(args.direction.as_deref(), Direction::Outgoing)?;
        let scope = self.scope(args.project)?;
        let limit = self.apply_limit(args.limit);
        let layer_edges = self.index().layer_edges(scope.snapshot.id)?;
        let answer = match subject {
            Subject::Symbol(text) => {
                let ask = Ask {
                    text,
                    direction,
                    include_tests: args.include_tests.unwrap_or(false),
                };
                self.symbol_answer(&scope, &ask, &layer_edges)?
            }
            Subject::Layer(name) => layer_answer(&scope, name, direction, &layer_edges)?,
        };
        to_json(&answer.cut_to(limit.value, limit.clamped))
    }

    fn symbol_answer(
        &self,
        scope: &Scope,
        ask: &Ask<'_>,
        layers: &[LayerEdge],
    ) -> Result<Answer, McpError> {
        let direction = ask.direction;
        let found = self.find_symbol(scope, ask.text)?;
        let query = DependencyQuery {
            symbol: found.symbol.id,
            direction,
            include_tests: ask.include_tests,
        };
        let page = self
            .usage_index()
            .dependencies(scope.snapshot.id, &query, u32::MAX)?;
        let own = layer_of(&scope.project, &found.symbol.qualified_name);
        let context = (direction, own.as_deref());
        let dependencies: Vec<Dependency> = page
            .edges
            .iter()
            .map(|edge| dependency(&scope.project, layers, context, edge))
            .collect();
        Ok(Answer {
            snapshot: scope.info,
            target: Target {
                kind: crate::consts::TARGET_SYMBOL,
                name: found.symbol.qualified_name.as_str().to_owned(),
            },
            direction: direction_label(direction),
            total: page.total,
            violations: dependencies.iter().filter(|d| d.violates_layering).count(),
            layers: links(direction, own.as_deref(), layers, &dependencies),
            dependencies,
            notice: None,
        })
    }
}

fn layer_answer(
    scope: &Scope,
    name: &str,
    direction: Direction,
    edges: &[LayerEdge],
) -> Result<Answer, McpError> {
    let known = scope
        .project
        .layers
        .iter()
        .any(|layer| layer.name.as_str() == name);
    if !known {
        return Err(McpError::ArgumentsInvalid(ARG_LAYER.to_owned()));
    }
    let layers = layer_links(direction, name, edges);
    Ok(Answer {
        snapshot: scope.info,
        target: Target {
            kind: crate::consts::TARGET_LAYER,
            name: name.to_owned(),
        },
        direction: direction_label(direction),
        total: layers.len(),
        violations: layers.iter().filter(|link| link.violates_layering).count(),
        layers,
        dependencies: Vec::new(),
        notice: None,
    })
}

impl Answer {
    /// Cuts the lists to `limit` and adds the notice if anything was cut or the limit was lowered.
    fn cut_to(mut self, limit: u32, clamped: bool) -> Self {
        self.layers.truncate(limit as usize);
        self.dependencies.truncate(limit as usize);
        let shown = self.layers.len().max(self.dependencies.len());
        self.notice = cut_notice(limit, clamped, shown, self.total, None);
        self
    }
}
