//! Registered repositories and their source.

use crate::ModelError;
use crate::consts::{NAME_SEPARATOR, URL_AUTHORITY_END, URL_SCHEME_MARKER};
use crate::qualified_name::QualifiedName;
use crate::secret_url::without_secrets;
use crate::vocabulary::{display_as, id_type, text_type};

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
/// Address of a Git repository. Credentials in the address are removed on creation, so a secret
/// never gets stored, shown or written into the `.git/config` of a clone (blueprint 12.2).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GitUrl(String);

impl GitUrl {
    /// Creates the address without credentials.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::Empty`] if the text is empty or only whitespace.
    pub fn new(value: impl Into<String>) -> Result<Self, ModelError> {
        let text = value.into();
        if text.trim().is_empty() {
            Err(ModelError::Empty)
        } else {
            Ok(Self(without_secrets(&text)))
        }
    }

    /// The address.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Scheme and authority (`https://host:port`), which identify the server; empty text for
    /// addresses without `://`.
    #[must_use]
    pub fn origin(&self) -> &str {
        let Some(marker) = self.0.find(URL_SCHEME_MARKER) else {
            return <&str>::default();
        };
        let start = marker + URL_SCHEME_MARKER.len();
        let after = self.0.get(start..).unwrap_or_default();
        let end = start + after.find(URL_AUTHORITY_END).unwrap_or(after.len());
        self.0.get(..end).unwrap_or_default()
    }
}

display_as!(GitUrl, value => value.0.as_str());

impl ::std::str::FromStr for GitUrl {
    type Err = ModelError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Self::new(text)
    }
}
text_type!(
    /// Name of a Git branch.
    BranchName
);
text_type!(
    /// Names a credential in the secret store. It is a reference only and never the secret itself
    /// (blueprint 12.2), so it may be stored, shown and logged.
    CredentialId
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
        /// Credential to log in with; `None` leaves the login to the system's Git credential helper.
        credential: Option<CredentialId>,
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
