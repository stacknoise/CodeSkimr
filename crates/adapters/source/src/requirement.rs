//! Reading the version a project requires from one of its files.

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use codeskimr_application::ports::RequirementReader;
use codeskimr_domain::{ProjectSource, RelativePath, RequirementFormat, RequirementSpec};

use crate::consts::REQUIREMENT_COMMENT;
use crate::git::GitCache;
use crate::paths::inside;

/// Reads `global.json`, `rust-toolchain.toml`, `.nvmrc` and the like from the root of a project.
///
/// A file is read only if it lies inside the project root once links are resolved, is a regular
/// file and is not larger than the limit. Nothing is fetched: a Git project is read from the clone
/// that the last scan left in the cache.
#[derive(Debug, Clone)]
pub struct ProjectRequirements {
    git: GitCache,
    max_bytes: u64,
}

impl ProjectRequirements {
    /// Creates the reader. `git` says where clones lie; files above `max_bytes` are not read.
    #[must_use]
    pub const fn new(git: GitCache, max_bytes: u64) -> Self {
        Self { git, max_bytes }
    }

    fn root(&self, source: &ProjectSource) -> PathBuf {
        match source {
            ProjectSource::Folder(folder) => PathBuf::from(folder.as_str()),
            ProjectSource::Git { url, branch, .. } => self.git.checkout_dir(url, branch),
        }
    }

    fn text(&self, source: &ProjectSource, file: &str) -> Option<String> {
        let real = inside(&self.root(source), &RelativePath::new(file).ok()?)?;
        let handle = File::open(real).ok()?;
        if handle.metadata().ok()?.len() > self.max_bytes {
            return None;
        }
        let mut text = String::new();
        handle.take(self.max_bytes).read_to_string(&mut text).ok()?;
        Some(text)
    }
}

fn json_string(text: &str, path: &[String]) -> Option<String> {
    string_at(&serde_json::from_str(text).ok()?, path)
}

fn toml_string(text: &str, path: &[String]) -> Option<String> {
    string_at(&toml::from_str(text).ok()?, path)
}

/// The text at `path` of a parsed document (TOML is read into the same value type as JSON).
fn string_at(document: &serde_json::Value, path: &[String]) -> Option<String> {
    let leaf = path.iter().try_fold(document, |node, key| node.get(key))?;
    leaf.as_str().map(str::to_owned)
}

/// The first line that says something: not empty, not a comment.
fn first_line(text: &str) -> Option<String> {
    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && !line.starts_with(REQUIREMENT_COMMENT))
        .map(str::to_owned)
}

impl RequirementReader for ProjectRequirements {
    fn read(&self, source: &ProjectSource, requirement: &RequirementSpec) -> Option<String> {
        let text = self.text(source, &requirement.file)?;
        match requirement.format {
            RequirementFormat::Json => json_string(&text, &requirement.path),
            RequirementFormat::Toml => toml_string(&text, &requirement.path),
            RequirementFormat::Text => first_line(&text),
        }
    }
}
