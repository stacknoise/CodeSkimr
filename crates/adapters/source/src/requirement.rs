//! Reading the version a project requires from one of its files.

use std::fs::File;
use std::io::Read;
use std::iter::Peekable;
use std::path::{Path, PathBuf};
use std::str::Chars;

use codeskimr_application::ports::{RequirementReader, RequirementValue};
use codeskimr_domain::{ProjectSource, RelativePath, RequirementFormat, RequirementSpec};

use crate::consts::{
    JSON_CLOSERS, JSON_COMMA, JSON_ESCAPE, JSON_NEWLINE, JSON_QUOTE, JSON_SLASH, JSON_STAR,
    REQUIREMENT_COMMENT, REQUIREMENT_MAX_DEPTH, REQUIREMENT_MAX_FILES, REQUIREMENT_MAX_FOLDERS,
    REQUIREMENT_SKIPPED_FOLDERS,
};
use crate::git::GitCache;
use crate::paths::{inside, relative_path};

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
        self.read_regular(&real)
    }

    fn read_regular(&self, real: &Path) -> Option<String> {
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
    string_at(&serde_json::from_str(&lenient_json(text)).ok()?, path)
}

fn toml_string(text: &str, path: &[String]) -> Option<String> {
    string_at(&toml::from_str(text).ok()?, path)
}

/// `text` without comments (`//` and `/* */`) and trailing commas, which the .NET SDK accepts in
/// `global.json` and `serde_json` does not. What lies inside a string stays as it is.
fn lenient_json(text: &str) -> String {
    let stripped = rewrite(text, |c, chars, out| {
        if c == JSON_SLASH && chars.peek() == Some(&JSON_SLASH) {
            chars.by_ref().find(|next| *next == JSON_NEWLINE);
            out.push(JSON_NEWLINE);
        } else if c == JSON_SLASH && chars.peek() == Some(&JSON_STAR) {
            skip_block_comment(chars);
        } else {
            out.push(c);
        }
    });
    drop_trailing_commas(&stripped)
}

/// Walks `text`, copies every string untouched and hands each other character to `step`.
fn rewrite(
    text: &str,
    mut step: impl FnMut(char, &mut Peekable<Chars<'_>>, &mut String),
) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == JSON_QUOTE {
            copy_string(&mut out, &mut chars);
        } else {
            step(c, &mut chars, &mut out);
        }
    }
    out
}

/// Copies a string (the opening quote is out already) up to and including its closing quote.
fn copy_string(out: &mut String, chars: &mut Peekable<Chars<'_>>) {
    out.push(JSON_QUOTE);
    while let Some(c) = chars.next() {
        out.push(c);
        if c == JSON_ESCAPE {
            out.extend(chars.next());
        } else if c == JSON_QUOTE {
            break;
        }
    }
}

/// Skips up to and including the `*/` that ends a block comment; the `/` of the opening is read.
fn skip_block_comment(chars: &mut Peekable<Chars<'_>>) {
    // The star of the opening `/*` is not the star of a closing `*/`: `/*/` is still open.
    chars.next();
    let mut last = JSON_NEWLINE;
    for c in chars.by_ref() {
        if last == JSON_STAR && c == JSON_SLASH {
            return;
        }
        last = c;
    }
}

/// Removes a comma that is followed, ignoring white space, by `}` or `]`.
fn drop_trailing_commas(text: &str) -> String {
    rewrite(text, |c, chars, out| {
        let closes = || {
            let next = chars.clone().find(|next| !next.is_whitespace());
            next.is_some_and(|next| JSON_CLOSERS.contains(&next))
        };
        if c != JSON_COMMA || !closes() {
            out.push(c);
        }
    })
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

impl ProjectRequirements {
    /// The version `requirement` finds in the text of one file.
    fn value(requirement: &RequirementSpec, text: &str) -> Option<String> {
        match requirement.format {
            RequirementFormat::Json => json_string(text, &requirement.path),
            RequirementFormat::Toml => toml_string(text, &requirement.path),
            RequirementFormat::Text => first_line(text),
        }
    }

    /// The policy that limits how new a tool may be, from the same file. Plain text has no keys.
    fn policy(requirement: &RequirementSpec, text: &str) -> Option<String> {
        let path = &requirement.roll_forward.as_ref()?.path;
        match requirement.format {
            RequirementFormat::Json => json_string(text, path),
            RequirementFormat::Toml => toml_string(text, path),
            RequirementFormat::Text => None,
        }
    }

    fn values(requirement: &RequirementSpec, text: &str) -> Option<RequirementValue> {
        Some(RequirementValue {
            version: Self::value(requirement, text)?,
            policy: Self::policy(requirement, text),
        })
    }

    /// The folders below `root` that hold a file called `name`, root first. Folders that hold
    /// dependencies or build output are not entered, links are not followed, and the search is
    /// bounded in depth and in the number of files.
    fn manifests(root: &Path, name: &str) -> Vec<PathBuf> {
        let mut found = Vec::new();
        let mut open = vec![(root.to_path_buf(), usize::default())];
        let mut visited = usize::default();
        while let Some((folder, depth)) = open.pop() {
            // Enough files, or enough folders looked at: a huge tree must not keep the check busy.
            visited = visited.saturating_add(1);
            if found.len() >= REQUIREMENT_MAX_FILES || visited > REQUIREMENT_MAX_FOLDERS {
                break;
            }
            let candidate = folder.join(name);
            if candidate.is_file() {
                found.push(candidate);
            }
            if depth >= REQUIREMENT_MAX_DEPTH {
                continue;
            }
            let Ok(entries) = std::fs::read_dir(&folder) else {
                continue;
            };
            let mut below: Vec<PathBuf> = entries
                .filter_map(Result::ok)
                .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
                .filter(|entry| {
                    !REQUIREMENT_SKIPPED_FOLDERS
                        .iter()
                        .any(|skipped| entry.file_name() == *skipped)
                })
                .map(|entry| entry.path())
                .collect();
            below.sort();
            open.extend(
                below
                    .into_iter()
                    .rev()
                    .map(|path| (path, depth.saturating_add(1))),
            );
        }
        found
    }
}

impl RequirementReader for ProjectRequirements {
    fn read(&self, source: &ProjectSource, requirement: &RequirementSpec) -> Option<String> {
        Self::value(requirement, &self.text(source, &requirement.file)?)
    }

    fn read_all(
        &self,
        source: &ProjectSource,
        requirement: &RequirementSpec,
    ) -> Vec<RequirementValue> {
        let root = self.root(source);
        let name = Path::new(&requirement.file)
            .file_name()
            .and_then(|name| name.to_str());
        let Some(name) = name.filter(|name| *name == requirement.file) else {
            return self
                .text(source, &requirement.file)
                .and_then(|text| Self::values(requirement, &text))
                .into_iter()
                .collect();
        };
        Self::manifests(&root, name)
            .iter()
            .filter_map(|file| relative_path(&root, file))
            .filter_map(|file| self.text(source, file.as_str()))
            .filter_map(|text| Self::values(requirement, &text))
            .collect()
    }
}
