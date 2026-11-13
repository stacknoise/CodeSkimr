//! The `SourceProvider` port.

use std::sync::Arc;

use codeskimr_application::cancel::CancelToken;
use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{CredentialStore, FetchedSource, SourceProvider};
use codeskimr_domain::{CredentialId, ProjectSource};

use crate::folder::open_folder;
use crate::git::{GitCache, Login};

/// Opens folder sources and clones Git sources; credentials come from `S`.
#[derive(Debug)]
pub struct SourceFetcher<S> {
    git: GitCache,
    credentials: Arc<S>,
}

impl<S> Clone for SourceFetcher<S> {
    fn clone(&self) -> Self {
        Self {
            git: self.git.clone(),
            credentials: Arc::clone(&self.credentials),
        }
    }
}

impl<S: CredentialStore + 'static> SourceFetcher<S> {
    /// Creates the provider; Git clones live in `git`, credentials of Git sources are lent by
    /// `credentials`.
    #[must_use]
    pub fn new(git: GitCache, credentials: Arc<S>) -> Self {
        Self { git, credentials }
    }

    /// The login for a source, or `None` to leave it to the system's Git credential helper.
    fn login(&self, id: Option<&CredentialId>) -> Option<Login> {
        let id = id?.clone();
        let store = Arc::clone(&self.credentials);
        Some(Login::new(move |action| {
            store.with_credential(&id, |credential| {
                action(credential);
                Ok(())
            })
        }))
    }
}

impl<S: CredentialStore + 'static> SourceProvider for SourceFetcher<S> {
    fn fetch(
        &self,
        source: &ProjectSource,
        cancel: &CancelToken,
    ) -> Result<FetchedSource, CodedError> {
        let fetched = match source {
            ProjectSource::Folder(root) => open_folder(root),
            ProjectSource::Git {
                url,
                branch,
                credential,
            } => {
                let login = self.login(credential.as_ref());
                self.git.fetch(url, branch, login.as_ref(), cancel)
            }
        };
        Ok(fetched?)
    }
}
