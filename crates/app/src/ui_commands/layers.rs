//! The commands behind the layer editor: what a project has set and what it could set, and the
//! writing of the choice into the project file (ADR 0065).

use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex};

use codeskimr_application::access;
use codeskimr_application::config::{LayerRule, PROJECT_CONFIG_FILE, with_layers};
use codeskimr_application::error::CodedError;
use codeskimr_application::ports::OutlineNode;
use codeskimr_application::registry::{Command, CommandDescriptor, CommandId, Request};
use codeskimr_codes::ErrorCode;
use codeskimr_domain::{Layer, Permission, Project, ProjectId, ProjectSource, SymbolId};
use serde::{Deserialize, Serialize};

use super::consts::{
    LAYERS_GET, LAYERS_GET_DESCRIPTION, LAYERS_GET_TITLE, LAYERS_SET, LAYERS_SET_DESCRIPTION,
    LAYERS_SET_TITLE,
};
use super::explorer::{ExplorerPorts, namespace_of};
use crate::consts::PROJECT_FILE_MAX_BYTES;
use crate::project_file::{read_project_file, require_version};
use crate::rules_check::project_files;

/// The input of `layers.get`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct GetInput {
    /// Id of the project.
    pub project: u64,
}

/// A layer as the project has it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayerEntry {
    /// Name of the layer.
    pub name: String,
    /// Qualified-name prefixes that belong to the layer.
    pub namespaces: Vec<String>,
}

/// A namespace of the newest scan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NamespaceEntry {
    /// Qualified name of the namespace.
    pub name: String,
    /// Number of types declared directly in it.
    pub types: u32,
    /// The layer that owns it now, `None` if none does.
    pub layer: Option<String>,
}

/// The answer of `layers.get`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LayersOut {
    /// Whether the layers can be written: only a folder has a project file.
    pub editable: bool,
    /// Whether the project has a scan; without one there are no namespaces to choose from.
    pub scanned: bool,
    /// The layers the project has now, outermost first.
    pub layers: Vec<LayerEntry>,
    /// The namespaces of the newest scan, ordered by name.
    pub namespaces: Vec<NamespaceEntry>,
    /// The version of the project file as the layers were read, to send back with `layers.set`.
    /// Empty for a Git project.
    pub file_version: String,
}

/// The type counts of the namespaces of an outline, each with the layer that owns it.
fn namespaces_of(project: &Project, nodes: &[OutlineNode]) -> Vec<NamespaceEntry> {
    let by_id: HashMap<SymbolId, &OutlineNode> = nodes.iter().map(|n| (n.id, n)).collect();
    let mut counts: BTreeMap<String, u32> = BTreeMap::new();
    for node in nodes.iter().filter(|node| node.kind.is_type()) {
        if let Some(name) = namespace_of(node, &by_id).filter(|name| !name.is_empty()) {
            *counts.entry(name).or_default() += 1;
        }
    }
    counts
        .into_iter()
        .map(|(name, types)| NamespaceEntry {
            layer: project.layer_name_of(&name),
            name,
            types,
        })
        .collect()
}

fn layer_entries(layers: &[Layer]) -> Vec<LayerEntry> {
    layers
        .iter()
        .map(|layer| LayerEntry {
            name: layer.name.to_string(),
            namespaces: layer.namespaces.iter().map(ToString::to_string).collect(),
        })
        .collect()
}

/// Shows the layers of a project and the namespaces they can be built from.
pub struct LayersGet {
    pub(super) ports: Arc<ExplorerPorts>,
}

impl Command for LayersGet {
    type Input = GetInput;
    type Output = LayersOut;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(LAYERS_GET),
        title_key: LAYERS_GET_TITLE,
        description_key: LAYERS_GET_DESCRIPTION,
        shortcut: None,
        read_only: true,
        permission: Permission::Read,
    };

    fn execute(&self, request: Request<'_, Self>) -> Result<LayersOut, CodedError> {
        access::require(request.principal, Permission::Read)?;
        let ports = &self.ports;
        let (project, snapshot) = ports.project_and_newest(request.input.project)?;
        let namespaces = match &snapshot {
            Some(snapshot) => namespaces_of(&project, &ports.outline.outline(snapshot.id)?),
            None => Vec::new(),
        };
        let files = project_files(&project.source)?;
        Ok(LayersOut {
            editable: matches!(project.source, ProjectSource::Folder(_)),
            scanned: snapshot.is_some(),
            layers: layer_entries(files.layers.as_deref().unwrap_or(&project.layers)),
            namespaces,
            file_version: files.version,
        })
    }
}

/// The input of `layers.set`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SetInput {
    /// Id of the project.
    pub project: u64,
    /// The layers the project file gets, outermost first; they replace the ones it has.
    pub layers: Vec<LayerEntry>,
    /// The version `layers.get` answered; a file that changed since is refused with `CFG-106`.
    /// Left out, the layers are written without that check.
    pub file_version: Option<String>,
}

/// The answer of `layers.set`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Saved {
    /// Number of layers in the file now.
    pub layers: usize,
}

fn rule_of(entry: &LayerEntry) -> LayerRule {
    LayerRule {
        name: entry.name.clone(),
        namespaces: entry.namespaces.clone(),
        may_use: None,
    }
}

/// Writes `text` to the new file `temp`, gives it the mode of the file it replaces and makes sure
/// it is on disk.
fn write_temp(temp: &Path, path: &Path, text: &str) -> std::io::Result<()> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(temp)?;
    file.write_all(text.as_bytes())?;
    if let Ok(old) = fs::metadata(path) {
        fs::set_permissions(temp, old.permissions())?;
    }
    file.sync_all()
}

/// Replaces the file: the text goes to a new file beside it (created exclusively, so a link
/// someone put there is never followed) that is renamed over it, so a reader never sees half a
/// file. A text over the limit is refused, because the next read would refuse the file.
pub(super) fn write_project_file(path: &Path, text: &str) -> Result<(), CodedError> {
    if text.len() as u64 > PROJECT_FILE_MAX_BYTES {
        return Err(CodedError::new(ErrorCode::ConfigInvalidValue));
    }
    let temp = super::temp_file::beside(path);
    let moved = write_temp(&temp, path, text).and_then(|()| fs::rename(&temp, path));
    if moved.is_err() {
        let _ = fs::remove_file(&temp);
    }
    moved.map_err(|_| CodedError::new(ErrorCode::ProjectFileNotWritable))
}

/// Held while a project file is read, changed and written, so that two edits at the same time
/// (two tabs, a dialog and a layer editor) do not overwrite each other without a trace.
static PROJECT_FILE_LOCK: Mutex<()> = Mutex::new(());

/// Reads the project file, lets `change` build the new text (and an answer) from it and writes
/// that text, all under one process-wide lock (ADR 0112). A failed `change` writes nothing.
pub(super) fn update_project_file<T>(
    path: &Path,
    change: impl FnOnce(&str) -> Result<(String, T), CodedError>,
) -> Result<T, CodedError> {
    let _held = PROJECT_FILE_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let (text, answer) = change(&read_project_file(path)?)?;
    write_project_file(path, &text)?;
    Ok(answer)
}

/// Writes the layers of a folder project into its `.codeskimr.toml`.
pub struct LayersSet {
    pub(super) ports: Arc<ExplorerPorts>,
}

impl Command for LayersSet {
    type Input = SetInput;
    type Output = Saved;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(LAYERS_SET),
        title_key: LAYERS_SET_TITLE,
        description_key: LAYERS_SET_DESCRIPTION,
        shortcut: None,
        read_only: false,
        permission: Permission::Manage,
    };

    /// # Errors
    /// - `PRJ-201` for an unknown project, `PRJ-101` for a Git project (it has no project file).
    /// - `CFG-101` if a layer is not valid, `CFG-102` if the file cannot be read or is not TOML.
    /// - `CFG-104` if the file cannot be written.
    /// - `CFG-106` if the file changed since `layers.get` answered.
    fn execute(&self, request: Request<'_, Self>) -> Result<Saved, CodedError> {
        access::require(request.principal, Permission::Manage)?;
        let project = self
            .ports
            .projects
            .project(ProjectId::new(request.input.project))?;
        let ProjectSource::Folder(folder) = &project.source else {
            return Err(CodedError::new(ErrorCode::ProjectSourceInvalid));
        };
        let path = Path::new(folder.as_str()).join(PROJECT_CONFIG_FILE);
        let wanted: Vec<LayerRule> = request.input.layers.iter().map(rule_of).collect();
        let saved = Saved {
            layers: wanted.len(),
        };
        let version = request.input.file_version.as_deref();
        update_project_file(&path, |text| {
            require_version(text, version)?;
            let text = with_layers(text, wanted).map_err(CodedError::caused_by)?;
            Ok((text, saved))
        })
    }
}
