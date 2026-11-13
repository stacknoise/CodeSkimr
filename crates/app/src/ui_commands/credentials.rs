//! The commands of the credentials: which logins Git projects use, storing and removing them
//! (blueprint 12.4, issue #161).
//!
//! The secret travels in the request that stores it, once, and is handed straight to the keystore
//! of the system. No answer, no error and no log line carries it; the list shows names only.

use std::collections::BTreeSet;
use std::sync::Arc;

use codeskimr_application::access;
use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{AuthError, Credential, CredentialStore, ProjectCatalog};
use codeskimr_application::registry::{Command, CommandDescriptor, CommandId, Request};
use codeskimr_codes::ErrorCode;
use codeskimr_domain::{CredentialId, Permission, ProjectSource};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};

use super::consts::{
    CREDENTIALS_DELETE, CREDENTIALS_DELETE_DESCRIPTION, CREDENTIALS_DELETE_TITLE, CREDENTIALS_LIST,
    CREDENTIALS_LIST_DESCRIPTION, CREDENTIALS_LIST_TITLE, CREDENTIALS_SET,
    CREDENTIALS_SET_DESCRIPTION, CREDENTIALS_SET_TITLE,
};
use crate::consts::is_token_credential;
use crate::wiring::Secrets;

/// The name of a credential that a user may manage. The installation token is not one of them:
/// replacing it would lock the browser out, deleting it would break the next start.
fn usable(name: &str) -> Result<CredentialId, CodedError> {
    if is_token_credential(name) {
        return Err(CodedError::new(ErrorCode::RequestInvalid));
    }
    CredentialId::new(name).map_err(|_| CodedError::new(ErrorCode::RequestInvalid))
}

/// One credential that a project logs in with.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CredentialRow {
    /// Name of the credential.
    pub id: String,
    /// Whether the keystore holds a credential of that name.
    pub stored: bool,
}

/// Lists the credentials the Git projects name, and whether each exists in the keystore.
pub struct ListCredentials {
    pub(super) catalog: Arc<dyn ProjectCatalog>,
    pub(super) secrets: Arc<Secrets>,
}

impl Command for ListCredentials {
    type Input = ();
    type Output = Vec<CredentialRow>;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(CREDENTIALS_LIST),
        title_key: CREDENTIALS_LIST_TITLE,
        description_key: CREDENTIALS_LIST_DESCRIPTION,
        shortcut: None,
        read_only: true,
        permission: Permission::Read,
    };

    fn execute(&self, _request: Request<'_, ()>) -> Result<Vec<CredentialRow>, CodedError> {
        let named: BTreeSet<CredentialId> = self
            .catalog
            .overviews()?
            .into_iter()
            .filter_map(|overview| match overview.project.source {
                ProjectSource::Git { credential, .. } => credential,
                ProjectSource::Folder(_) => None,
            })
            .collect();
        named
            .into_iter()
            .map(|id| {
                let stored = match self.secrets.with_credential(&id, |_| Ok(())) {
                    Ok(()) => true,
                    Err(AuthError::NotFound) => false,
                    Err(other) => return Err(CodedError::from(other)),
                };
                Ok(CredentialRow {
                    id: id.to_string(),
                    stored,
                })
            })
            .collect()
    }
}

/// The input of `credentials.set`. The secret is never printed: `SecretString` hides it in
/// `Debug` and wipes it when dropped.
#[derive(Debug, Deserialize)]
pub struct SetInput {
    /// Name to store the credential under.
    pub id: String,
    /// User name of the login; a token login uses any non-empty name.
    pub user: String,
    /// Password or token.
    pub secret: SecretString,
}

/// What `credentials.set` and `credentials.delete` report: the name, never the secret.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CredentialDone {
    /// Name of the credential.
    pub id: String,
}

/// Stores or replaces a credential.
pub struct SetCredential {
    pub(super) secrets: Arc<Secrets>,
}

impl Command for SetCredential {
    type Input = SetInput;
    type Output = CredentialDone;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(CREDENTIALS_SET),
        title_key: CREDENTIALS_SET_TITLE,
        description_key: CREDENTIALS_SET_DESCRIPTION,
        shortcut: None,
        read_only: false,
        permission: Permission::RegisterProject,
    };

    /// # Errors
    /// `API-101` for the name of the installation token, an invalid name, an empty user or an
    /// empty secret; `SEC-401`, `SEC-402` or `SEC-601` from the keystore.
    fn execute(&self, request: Request<'_, SetInput>) -> Result<CredentialDone, CodedError> {
        access::require(request.principal, Permission::RegisterProject)?;
        let input = request.input;
        let id = usable(&input.id)?;
        if input.user.trim().is_empty() || input.secret.expose_secret().is_empty() {
            return Err(CodedError::new(ErrorCode::RequestInvalid));
        }
        let credential = Credential::new(SecretString::from(input.user), input.secret);
        self.secrets.store(&id, &credential)?;
        Ok(CredentialDone { id: id.to_string() })
    }
}

/// The input of `credentials.delete`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct DeleteInput {
    /// Name of the credential.
    pub id: String,
}

/// Removes a credential; a missing one is not an error.
pub struct DeleteCredential {
    pub(super) secrets: Arc<Secrets>,
}

impl Command for DeleteCredential {
    type Input = DeleteInput;
    type Output = CredentialDone;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(CREDENTIALS_DELETE),
        title_key: CREDENTIALS_DELETE_TITLE,
        description_key: CREDENTIALS_DELETE_DESCRIPTION,
        shortcut: None,
        read_only: false,
        permission: Permission::RegisterProject,
    };

    /// # Errors
    /// `API-101` for the name of the installation token or an invalid name; `SEC-402` or
    /// `SEC-601` from the keystore.
    fn execute(&self, request: Request<'_, DeleteInput>) -> Result<CredentialDone, CodedError> {
        let id = usable(&request.input.id)?;
        self.secrets.delete(&id)?;
        Ok(CredentialDone { id: id.to_string() })
    }
}
