//! A listing of every file of a project folder for the scope page (ADR 0101): the walk of a scan,
//! without the scope and without reading a file.

use std::path::Path;

use codeskimr_application::cancel::CancelToken;
use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{FolderLister, ListedFile};
use codeskimr_domain::{FolderPath, Scope};

use crate::discovery::{DiscoveryConfig, builder};
use crate::error::SourceError;
use crate::folder::open_folder;
use crate::languages::LanguageMap;
use crate::paths::relative_path;

/// Lists the files of a folder with their languages.
#[derive(Debug, Clone)]
pub struct WalkLister {
    config: DiscoveryConfig,
    languages: LanguageMap,
}

impl WalkLister {
    /// Creates the adapter; the folders that a scan never enters are not entered here either.
    #[must_use]
    pub const fn new(config: DiscoveryConfig, languages: LanguageMap) -> Self {
        Self { config, languages }
    }
}

impl FolderLister for WalkLister {
    fn list(&self, root: &FolderPath, cancel: &CancelToken) -> Result<Vec<ListedFile>, CodedError> {
        open_folder(root)?;
        let base = Path::new(root.as_str());
        let walk = builder(base, &Scope::default(), &self.config.excluded_folders).build();
        let mut files = Vec::new();
        for entry in walk {
            if cancel.is_cancelled() {
                break;
            }
            let entry = entry.map_err(|_| SourceError::PathNotFound { path: root.clone() })?;
            if !entry.file_type().is_some_and(|kind| kind.is_file()) {
                continue;
            }
            if let Some(path) = relative_path(base, entry.path()) {
                let language = self.languages.detect(entry.path()).cloned();
                files.push(ListedFile { path, language });
            }
        }
        files.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(files)
    }
}
