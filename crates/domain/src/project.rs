//! Registered repositories and their source.

use crate::consts::NAME_SEPARATOR;
use crate::qualified_name::QualifiedName;
use crate::vocabulary::{id_type, text_type};

id_type!(
    /// Identifies a registered project.
    ProjectId
);
text_type!(
    /// Display name of a project.
    ProjectName
);
text_type!(
    /// Absolute path of a local folder.
    FolderPath
);
text_type!(
    /// Address of a Git repository.
    GitUrl
);
text_type!(
    /// Name of a Git branch.
    BranchName
);
text_type!(
    /// Name of an architecture layer.
    LayerName
);

/// Where the code of a project comes from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectSource {
    /// A folder on this machine.
    Folder(FolderPath),
    /// A Git repository at one branch.
    Git {
        /// Repository address.
        url: GitUrl,
        /// Branch that is scanned.
        branch: BranchName,
    },
}

/// A named architecture layer made of namespace prefixes (blueprint 6.1, `LayerEdge`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layer {
    /// Name of the layer.
    pub name: LayerName,
    /// Namespaces that belong to the layer.
    pub namespaces: Vec<QualifiedName>,
}

impl Layer {
    /// Whether a qualified name belongs to this layer: it equals one of the namespaces or lies
    /// below it, up to a separator (`App.Domain` owns `App.Domain.Order`, not `App.DomainX`).
    #[must_use]
    pub fn owns(&self, name: &str) -> bool {
        self.namespaces.iter().any(|namespace| {
            let prefix = namespace.as_str();
            name == prefix
                || name
                    .strip_prefix(prefix)
                    .is_some_and(|rest| rest.starts_with(NAME_SEPARATOR))
        })
    }
}

/// A registered repository.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    /// Identity.
    pub id: ProjectId,
    /// Display name.
    pub name: ProjectName,
    /// Where the code comes from.
    pub source: ProjectSource,
    /// Architecture layers, empty when none are defined.
    pub layers: Vec<Layer>,
}
