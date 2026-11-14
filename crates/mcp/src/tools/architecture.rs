//! `get_architecture`.

use codeskimr_application::access;
use codeskimr_domain::{Layer, LayerEdge, Principal};
use serde::Serialize;
use serde_json::Value;

use super::{SnapshotInfo, Tools, to_json};
use crate::args::ArchitectureArgs;
use crate::consts::READ_TOOL_PERMISSION;
use crate::error::McpError;

#[derive(Serialize)]
struct LayerInfo {
    name: String,
    namespaces: Vec<String>,
    /// The layers this one may depend on; `null` when it has no limit.
    may_use: Option<Vec<String>>,
}

/// The code rules the project is checked against (`check_conventions`).
#[derive(Serialize)]
struct ConventionInfo {
    max_function_lines: u32,
    duplicate_similarity_percent: u32,
    duplicate_min_lines: u32,
    test_folders: Vec<String>,
}

#[derive(Serialize)]
struct Dependency {
    from: String,
    to: String,
    count: u32,
    violates_layering: bool,
    in_cycle: bool,
}

#[derive(Serialize)]
struct Answer {
    snapshot: SnapshotInfo,
    project: String,
    layers: Vec<LayerInfo>,
    dependencies: Vec<Dependency>,
    conventions: ConventionInfo,
}

fn layer(layer: &Layer) -> LayerInfo {
    LayerInfo {
        name: layer.name.as_str().to_owned(),
        namespaces: layer
            .namespaces
            .iter()
            .map(|namespace| namespace.as_str().to_owned())
            .collect(),
        may_use: layer
            .may_use
            .as_ref()
            .map(|names| names.iter().map(|name| name.as_str().to_owned()).collect()),
    }
}

fn dependency(edge: &LayerEdge) -> Dependency {
    Dependency {
        from: edge.from.as_str().to_owned(),
        to: edge.to.as_str().to_owned(),
        count: edge.count,
        violates_layering: edge.violates_layering,
        in_cycle: edge.in_cycle,
    }
}

impl Tools {
    fn conventions(&self) -> ConventionInfo {
        let limits = self.checker().limits();
        ConventionInfo {
            max_function_lines: limits.max_function_lines,
            duplicate_similarity_percent: limits.duplicate_similarity_percent,
            duplicate_min_lines: limits.duplicate_min_lines,
            test_folders: limits.test_folders.clone(),
        }
    }

    /// The layers of a project with their namespaces and the dependencies between them.
    ///
    /// # Errors
    /// `AUTH-401` if the principal may not read, `MCP-201` and `MCP-203` for the project,
    /// or the code of a failing port.
    pub fn get_architecture(
        &self,
        principal: &Principal,
        args: &ArchitectureArgs,
    ) -> Result<Value, McpError> {
        access::require(principal, READ_TOOL_PERMISSION)?;
        let scope = self.scope(args.project)?;
        let edges = self.index().layer_edges(scope.snapshot.id)?;
        to_json(&Answer {
            snapshot: scope.info,
            project: scope.project.name.as_str().to_owned(),
            layers: scope.project.layers.iter().map(layer).collect(),
            dependencies: edges.iter().map(dependency).collect(),
            conventions: self.conventions(),
        })
    }
}
