//! The commands of the explorer: the tree of layers, namespaces and types, and one symbol with
//! its members.

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use codeskimr_application::access;
use codeskimr_application::config::MetricThresholds;
use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{
    CodeIndex, CodeOutline, OutlineNode, SourceText, SyntaxHighlighter,
};
use codeskimr_application::registry::{Command, CommandDescriptor, CommandId, Request};
use codeskimr_codes::ErrorCode;
use codeskimr_domain::{Layer, Permission, Project, ProjectId, Snapshot, SymbolId, SymbolKind};
use serde::{Deserialize, Serialize};

use super::consts::{EXPLORER_OUTLINE, OUTLINE_DESCRIPTION, OUTLINE_TITLE};

/// What the explorer reads: the project repository for the layers, the index for one symbol, the
/// outline for the tree and the project folder for the source of a declaration.
pub struct ExplorerPorts {
    pub(super) projects: Arc<dyn codeskimr_application::ports::ProjectRepository>,
    pub(super) index: Arc<dyn CodeIndex>,
    pub(super) outline: Arc<dyn CodeOutline>,
    pub(super) text: Arc<dyn SourceText>,
    pub(super) highlighter: Arc<dyn SyntaxHighlighter>,
    pub(super) thresholds: MetricThresholds,
}

impl ExplorerPorts {
    /// A registered project and its newest snapshot, `None` before the first scan.
    ///
    /// # Errors
    /// `PRJ-201` for an unknown project.
    pub(super) fn project_and_newest(
        &self,
        project: u64,
    ) -> Result<(Project, Option<Snapshot>), CodedError> {
        let found = self.projects.project(ProjectId::new(project))?;
        let snapshot = self.index.latest_snapshot(found.id)?;
        Ok((found, snapshot))
    }

    /// The newest snapshot of a registered project.
    ///
    /// # Errors
    /// `PRJ` code for an unknown project, `MCP-203` before the first scan.
    pub(super) fn newest(&self, project: u64) -> Result<Snapshot, CodedError> {
        let id = ProjectId::new(project);
        self.projects.project(id)?;
        self.index
            .latest_snapshot(id)?
            .ok_or_else(|| CodedError::new(ErrorCode::ProjectNotScanned))
    }
}

/// A type in the tree.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TypeNode {
    /// Id of the symbol.
    pub id: u64,
    /// Simple name.
    pub name: String,
    /// Qualified name: with `kind` it is the key of the annotations of the type (ADR 0068).
    pub qualified_name: String,
    /// Kind label.
    pub kind: &'static str,
    /// Number of direct members.
    pub members: u32,
    /// Id of the language pack of the declaring file.
    pub language: String,
}

/// A namespace with its types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NamespaceNode {
    /// Qualified name of the namespace, empty for types without one.
    pub name: String,
    /// Its types, ordered by name.
    pub types: Vec<TypeNode>,
    /// The file module that has this very name (Python, Rust, TypeScript, JavaScript: the file
    /// is the namespace), so the tree can show and open it. `None` if no file is the namespace.
    pub module: Option<TypeNode>,
    /// Other symbols that hold functions or constants of this namespace: a namespace symbol (a
    /// TypeScript `namespace`, a Kotlin package, declared in one or more files) or the module of
    /// a file without a namespace. `members` counts the members that are no rows of the tree.
    pub holders: Vec<TypeNode>,
    /// Whether the name is the path of a file (a file without a namespace in a language that has
    /// them, ADR 0036) and no namespace name: the tree shows it as one row, named by the file.
    pub file: bool,
}

/// A layer with its namespaces; the name is empty for types that belong to no layer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LayerNode {
    /// Name of the layer.
    pub name: String,
    /// Its namespaces, ordered by name.
    pub namespaces: Vec<NamespaceNode>,
}

/// The limits behind the colours of the metrics, so the column tooltips can name them.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Thresholds {
    /// Complexity up to this value is good.
    pub complexity_good_max: u32,
    /// Complexity up to this value is medium.
    pub complexity_medium_max: u32,
    /// Maintainability from this value is good.
    pub maintainability_good_min: u32,
    /// Maintainability from this value is medium.
    pub maintainability_medium_min: u32,
    /// Instability up to this value is stable.
    pub instability_stable_max: f64,
    /// Instability up to this value is medium.
    pub instability_medium_max: f64,
}

impl From<&MetricThresholds> for Thresholds {
    fn from(config: &MetricThresholds) -> Self {
        Self {
            complexity_good_max: config.cyclomatic_good_max,
            complexity_medium_max: config.cyclomatic_medium_max,
            maintainability_good_min: config.maintainability_good_min,
            maintainability_medium_min: config.maintainability_medium_min,
            instability_stable_max: config.instability_stable_max,
            instability_medium_max: config.instability_medium_max,
        }
    }
}

/// The tree of the newest snapshot.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Outline {
    /// Id of the snapshot, `None` before the first scan.
    pub snapshot: Option<u64>,
    /// Layers in their declared order; types that belong to none come last, with an empty name.
    pub layers: Vec<LayerNode>,
    /// The limits behind the colours of the metrics.
    pub thresholds: Thresholds,
}

/// The input of both commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct OutlineInput {
    /// Id of the project.
    pub project: u64,
}

/// What one namespace holds while the tree is built.
#[derive(Debug, Clone, Default)]
struct Space {
    types: Vec<TypeNode>,
    module: Option<TypeNode>,
    holders: Vec<TypeNode>,
    file: bool,
}

type Grouped = BTreeMap<String, Space>;

/// Whether a symbol is the module of a file that names its namespace: its qualified name ends
/// with its own name as a whole last segment. A language with real namespaces keeps the path of
/// the file with its extension (ADR 0036), which is no namespace and gets no row.
fn is_namespace_module(node: &OutlineNode) -> bool {
    node.kind == SymbolKind::Module
        && node
            .qualified_name
            .as_str()
            .strip_suffix(node.name.as_str())
            .is_some_and(|before| !before.ends_with(char::is_alphanumeric))
}

fn type_node(node: &OutlineNode) -> TypeNode {
    TypeNode {
        id: node.id.get(),
        name: node.name.to_string(),
        qualified_name: node.qualified_name.to_string(),
        kind: node.kind.label(),
        members: node.members,
        language: node.language.to_string(),
    }
}

/// The namespace a top-level type sits in: its parent container, or the name before its own.
pub(super) fn namespace_of(
    node: &OutlineNode,
    names: &HashMap<SymbolId, &OutlineNode>,
) -> Option<String> {
    match node.parent {
        Some(parent) => names
            .get(&parent)
            .filter(|container| !container.kind.is_type())
            .map(|container| container.qualified_name.to_string()),
        None => Some(
            node.qualified_name
                .enclosing()
                .unwrap_or_default()
                .to_owned(),
        ),
    }
}

fn layer_index(layers: &[Layer], node: &OutlineNode) -> usize {
    layers
        .iter()
        .position(|layer| layer.owns(node.qualified_name.as_str()))
        .unwrap_or(layers.len())
}

fn layer_nodes(layers: &[Layer], mut groups: Vec<Grouped>) -> Vec<LayerNode> {
    let loose = groups.pop().unwrap_or_default();
    let named = layers.iter().map(|layer| layer.name.to_string());
    let mut nodes: Vec<LayerNode> = named
        .zip(groups)
        .map(|(name, grouped)| LayerNode {
            name,
            namespaces: namespaces(grouped),
        })
        .collect();
    if !loose.is_empty() {
        nodes.push(LayerNode {
            name: String::new(),
            namespaces: namespaces(loose),
        });
    }
    nodes
}

fn namespaces(grouped: Grouped) -> Vec<NamespaceNode> {
    grouped
        .into_iter()
        .map(|(name, mut space)| {
            space.types.sort_by(|a, b| a.name.cmp(&b.name));
            NamespaceNode {
                name,
                types: space.types,
                module: space.module,
                holders: space.holders,
                file: space.file,
            }
        })
        .collect()
}

/// Whether a top-level type sits directly in the module of its file and that module is no
/// namespace: the type has no namespace and its namespace name is the path of the file.
fn in_file_module(node: &OutlineNode, names: &HashMap<SymbolId, &OutlineNode>) -> bool {
    node.parent
        .and_then(|parent| names.get(&parent))
        .is_some_and(|container| {
            container.kind == SymbolKind::Module && !is_namespace_module(container)
        })
}

/// How many members of each container are no rows of the tree: its functions, constants, fields
/// and aliases. The outline holds containers and types only, so they are the difference between
/// the member count and the nodes of the outline that have the container as parent.
fn own_members(nodes: &[OutlineNode]) -> HashMap<SymbolId, u32> {
    let mut rows: HashMap<SymbolId, u32> = HashMap::new();
    for parent in nodes.iter().filter_map(|node| node.parent) {
        *rows.entry(parent).or_default() += 1;
    }
    nodes
        .iter()
        .map(|node| {
            let shown = rows.get(&node.id).copied().unwrap_or_default();
            (node.id, node.members.saturating_sub(shown))
        })
        .collect()
}

/// Whether a symbol holds functions or constants that need a row to hang on, apart from the
/// file module that is its own namespace: a namespace symbol, or the module of a file without a
/// namespace (ADR 0036) with members of its own.
fn holds_members(node: &OutlineNode, own: &HashMap<SymbolId, u32>) -> bool {
    let holder = node.kind == SymbolKind::Namespace
        || (node.kind == SymbolKind::Module && !is_namespace_module(node));
    holder && own.get(&node.id).is_some_and(|count| *count > 0)
}

/// Sorts the top-level types into layers and namespaces.
pub(super) fn build(layers: &[Layer], nodes: &[OutlineNode]) -> Vec<LayerNode> {
    let by_id: HashMap<SymbolId, &OutlineNode> = nodes.iter().map(|n| (n.id, n)).collect();
    let mut groups: Vec<Grouped> = vec![Grouped::new(); layers.len() + 1];
    for node in nodes.iter().filter(|node| node.kind.is_type()) {
        if let Some(namespace) = namespace_of(node, &by_id)
            && let Some(group) = groups.get_mut(layer_index(layers, node))
        {
            let space = group.entry(namespace).or_default();
            space.file |= in_file_module(node, &by_id);
            space.types.push(type_node(node));
        }
    }
    let own = own_members(nodes);
    for node in nodes.iter().filter(|node| is_namespace_module(node)) {
        if let Some(group) = groups.get_mut(layer_index(layers, node)) {
            let name = node.qualified_name.to_string();
            let mut shown = type_node(node);
            shown.members = own.get(&node.id).copied().unwrap_or_default();
            group.entry(name).or_default().module.get_or_insert(shown);
        }
    }
    for node in nodes.iter().filter(|node| holds_members(node, &own)) {
        if let Some(group) = groups.get_mut(layer_index(layers, node)) {
            let mut shown = type_node(node);
            shown.members = own.get(&node.id).copied().unwrap_or_default();
            let name = node.qualified_name.to_string();
            let space = group.entry(name).or_default();
            space.file |= node.kind == SymbolKind::Module;
            space.holders.push(shown);
        }
    }
    layer_nodes(layers, groups)
}

/// The tree of layers, namespaces and types of the newest snapshot.
pub struct ExplorerOutline {
    pub(super) ports: Arc<ExplorerPorts>,
}

impl Command for ExplorerOutline {
    type Input = OutlineInput;
    type Output = Outline;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(EXPLORER_OUTLINE),
        title_key: OUTLINE_TITLE,
        description_key: OUTLINE_DESCRIPTION,
        shortcut: None,
        read_only: true,
        permission: Permission::Read,
    };

    fn execute(&self, request: Request<'_, Self>) -> Result<Outline, CodedError> {
        access::require(request.principal, Permission::Read)?;
        let ports = &self.ports;
        let (project, snapshot) = ports.project_and_newest(request.input.project)?;
        let Some(snapshot) = snapshot else {
            return Ok(Outline {
                snapshot: None,
                layers: Vec::new(),
                thresholds: Thresholds::from(&ports.thresholds),
            });
        };
        let nodes = ports.outline.outline(snapshot.id)?;
        Ok(Outline {
            snapshot: Some(snapshot.id.get()),
            layers: build(&project.layers, &nodes),
            thresholds: Thresholds::from(&ports.thresholds),
        })
    }
}
