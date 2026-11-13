//! The `SourceProvider` port.

use codeskimr_application::cancel::CancelToken;
use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{FetchedSource, SourceProvider};
use codeskimr_domain::ProjectSource;

use crate::folder::open_folder;
use crate::git::GitCache;

/// Opens folder sources and clones Git sources.
#[derive(Debug, Clone)]
pub struct SourceFetcher {
    git: GitCache,
}

impl SourceFetcher {
    /// Creates the provider; Git clones live in `git`.
    #[must_use]
    pub fn new(git: GitCache) -> Self {
        Self { git }
    }
}

impl SourceProvider for SourceFetcher {
    fn fetch(
        &self,
        source: &ProjectSource,
        cancel: &CancelToken,
    ) -> Result<FetchedSource, CodedError> {
        let fetched = match source {
            ProjectSource::Folder(root) => open_folder(root),
            ProjectSource::Git { url, branch } => self.git.fetch(url, branch, cancel),
        };
        Ok(fetched?)
    }
}
