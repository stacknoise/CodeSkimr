//! The scope a user asks for, checked before it is stored (ADR 0097).
//!
//! Folders and languages are texts that a person typed or a browser sent, so they are checked here
//! once: a folder must be a folder inside the project, a language must look like the id of a
//! pack. Nothing is opened: like a [`crate::selection::Selection`], a folder is only compared with
//! the paths that discovery found.

use std::collections::BTreeMap;

use codeskimr_codes::ErrorCode;
use codeskimr_domain::{LanguageId, RelativePath, Scope};

use crate::consts::{
    CURRENT_DIR, DRIVE_MARK, PARENT_DIR, PATH_SEPARATOR, SCOPE_LANGUAGE_EXTRA,
    SCOPE_LANGUAGE_MAX_CHARS,
};
use crate::error::CodedError;
use crate::ports::{ListedFile, matches_project_file};
use crate::selection::Selection;

/// The scope that `languages` and `folders` ask for, with every folder in the form the project
/// uses (forward slashes, no `./` in front, no separator at the end).
///
/// # Errors
/// `PRJ-103` for a folder that is empty, absolute, names a drive, has a `..` part or a control
/// character, for a language that is not named like a pack, and for more than `max_entries`
/// entries.
pub fn checked_scope(
    languages: &[String],
    folders: &[String],
    max_entries: u32,
) -> Result<Scope, CodedError> {
    let wanted = usize::try_from(max_entries).unwrap_or(usize::MAX);
    if languages.len().saturating_add(folders.len()) > wanted {
        return Err(invalid());
    }
    let languages: Option<Vec<LanguageId>> = languages.iter().map(|name| language(name)).collect();
    let folders: Option<Vec<RelativePath>> = Selection::new(folders)
        .paths()
        .iter()
        .map(|path| folder(path))
        .collect();
    match (languages, folders) {
        (Some(languages), Some(folders)) => Ok(Scope::new(languages, folders)),
        _ => Err(invalid()),
    }
}

fn invalid() -> CodedError {
    CodedError::new(ErrorCode::ProjectScopeInvalid)
}

/// The language, if the text is named like the id of a pack.
fn language(text: &str) -> Option<LanguageId> {
    let name = text.trim();
    let fits = name.chars().count() <= SCOPE_LANGUAGE_MAX_CHARS
        && name
            .chars()
            .all(|c| c.is_alphanumeric() || SCOPE_LANGUAGE_EXTRA.contains(&c));
    fits.then(|| LanguageId::new(name).ok()).flatten()
}

/// The folder, if it is a folder inside the project.
fn folder(path: &str) -> Option<RelativePath> {
    let inside = !path.is_empty()
        && path != CURRENT_DIR
        && !path.starts_with(PATH_SEPARATOR)
        && !path.chars().any(char::is_control)
        && path
            .split(PATH_SEPARATOR)
            .all(|part| !part.is_empty() && part != PARENT_DIR && !part.contains(DRIVE_MARK));
    inside.then(|| RelativePath::new(path).ok()).flatten()
}

/// How far the overview of a folder goes.
#[derive(Debug, Clone, Copy)]
pub struct OverviewLimits {
    /// Folders are listed down to this depth; the project folder is depth 0.
    pub depth: u32,
    /// Most folders that are listed; the others are counted.
    pub max_folders: u32,
}

/// One folder of the overview.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderRow {
    /// The folder, relative to the project folder.
    pub path: RelativePath,
    /// Number of files of known languages in it and below it.
    pub files: u64,
    /// The same files by language, the language with the most files first.
    pub languages: Vec<(LanguageId, u64)>,
    /// The names of the files in the folder itself that an indexer needs (`Cargo.toml`).
    pub project_files: Vec<String>,
}

/// What the files of a project hold, by language and by folder.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Overview {
    /// Files of known languages by language, the language with the most files first.
    pub languages: Vec<(LanguageId, u64)>,
    /// The folders down to the depth that hold files of known languages or a project file, in
    /// path order.
    pub folders: Vec<FolderRow>,
    /// Folders that were not listed because of `max_folders`.
    pub folders_more: u64,
    /// The names of the files in the project folder itself that an indexer needs.
    pub root_project_files: Vec<String>,
}

/// What is counted for one folder.
#[derive(Default)]
struct Tally {
    files: u64,
    languages: BTreeMap<LanguageId, u64>,
    project_files: Vec<String>,
}

/// Counts `files` per language and per folder. `project_files` are the names (or `*` patterns)
/// that an indexer needs; a file of that name is noted for the folder it lies in.
#[must_use]
pub fn overview<'a>(
    files: impl IntoIterator<Item = &'a ListedFile>,
    project_files: &[String],
    limits: OverviewLimits,
) -> Overview {
    let mut folders: BTreeMap<RelativePath, Tally> = BTreeMap::new();
    let mut totals: BTreeMap<LanguageId, u64> = BTreeMap::new();
    let mut root_project_files = Vec::new();
    for file in files {
        let (parents, name) = split(&file.path);
        let wanted = project_files
            .iter()
            .any(|pattern| matches_project_file(pattern, name));
        if let (true, true) = (wanted, parents.is_empty()) {
            root_project_files.push(name.to_owned());
        }
        let listed = folders_above(&parents, limits.depth);
        for folder in &listed {
            let tally = folders.entry(folder.clone()).or_default();
            if let Some(language) = &file.language {
                tally.files += 1;
                *tally.languages.entry(language.clone()).or_default() += 1;
            }
        }
        if let (true, Some(own)) = (wanted, owner(&listed, &parents)) {
            folders
                .entry(own)
                .or_default()
                .project_files
                .push(name.to_owned());
        }
        if let Some(language) = &file.language {
            *totals.entry(language.clone()).or_default() += 1;
        }
    }
    listed_overview(folders, totals, root_project_files, limits.max_folders)
}

/// The folders of a path and its file name.
fn split(path: &RelativePath) -> (Vec<&str>, &str) {
    let mut parts: Vec<&str> = path.as_str().split(PATH_SEPARATOR).collect();
    let name = parts.pop().unwrap_or_default();
    (parts, name)
}

/// The folders above a file that the overview lists, outermost first, down to `depth`.
fn folders_above(parents: &[&str], depth: u32) -> Vec<RelativePath> {
    let wanted = usize::try_from(depth).unwrap_or(usize::MAX);
    (1..=parents.len().min(wanted))
        .filter_map(|count| {
            let joined = parents.get(..count)?.join(&PATH_SEPARATOR.to_string());
            RelativePath::new(joined).ok()
        })
        .collect()
}

/// The folder a file lies in, if the overview lists that folder.
fn owner(listed: &[RelativePath], parents: &[&str]) -> Option<RelativePath> {
    listed
        .last()
        .filter(|_| listed.len() == parents.len())
        .cloned()
}

/// Orders the languages by file count, the highest first, then by name.
fn by_count(counts: BTreeMap<LanguageId, u64>) -> Vec<(LanguageId, u64)> {
    let mut list: Vec<(LanguageId, u64)> = counts.into_iter().collect();
    list.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    list
}

fn listed_overview(
    folders: BTreeMap<RelativePath, Tally>,
    totals: BTreeMap<LanguageId, u64>,
    root_project_files: Vec<String>,
    max_folders: u32,
) -> Overview {
    let wanted = usize::try_from(max_folders).unwrap_or(usize::MAX);
    let more = folders.len().saturating_sub(wanted);
    let rows = folders
        .into_iter()
        .take(wanted)
        .map(|(path, tally)| FolderRow {
            path,
            files: tally.files,
            languages: by_count(tally.languages),
            project_files: tally.project_files,
        })
        .collect();
    Overview {
        languages: by_count(totals),
        folders: rows,
        folders_more: u64::try_from(more).unwrap_or(u64::MAX),
        root_project_files,
    }
}

/// A language on the scope page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageEntry {
    /// The language.
    pub language: LanguageId,
    /// Its files, `None` if they were not counted (the language is left out and no live count
    /// exists).
    pub files: Option<u64>,
    /// Whether scans cover it.
    pub covered: bool,
}

/// A folder on the scope page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderEntry {
    /// The folder, relative to the project folder.
    pub path: RelativePath,
    /// Its files of known languages, `None` if they were not counted.
    pub files: Option<u64>,
    /// Whether scans cover it: neither it nor a folder above it is left out.
    pub covered: bool,
    /// Whether the scope names this very folder (a folder below a left-out one is not covered but
    /// is not named).
    pub left_out: bool,
    /// Its files by language.
    pub languages: Vec<(LanguageId, u64)>,
    /// The files in it that an indexer needs.
    pub project_files: Vec<String>,
}

/// What the scope page shows: the counted languages and folders, each with whether scans cover it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScopeView {
    /// The languages, the ones with the most files first, then those that are only left out.
    pub languages: Vec<LanguageEntry>,
    /// The folders in path order.
    pub folders: Vec<FolderEntry>,
    /// Folders that were not listed (`max_folders`).
    pub folders_more: u64,
    /// The files in the project folder itself that an indexer needs.
    pub root_project_files: Vec<String>,
}

/// Puts the scope next to what was counted. What the scope leaves out is always listed, so that
/// it can be taken back, also when the count does not show it any more (the files of a language
/// that is left out are not in the newest scan).
#[must_use]
pub fn scope_view(scope: &Scope, counted: Overview) -> ScopeView {
    ScopeView {
        languages: language_entries(scope, &counted.languages),
        folders: folder_entries(scope, counted.folders),
        folders_more: counted.folders_more,
        root_project_files: counted.root_project_files,
    }
}

fn language_entries(scope: &Scope, counted: &[(LanguageId, u64)]) -> Vec<LanguageEntry> {
    let mut entries: Vec<LanguageEntry> = counted
        .iter()
        .map(|(language, files)| LanguageEntry {
            language: language.clone(),
            files: Some(*files),
            covered: scope.covers_language(language),
        })
        .collect();
    let unseen = scope
        .languages()
        .iter()
        .filter(|language| !counted.iter().any(|(seen, _)| seen == *language));
    entries.extend(unseen.map(|language| LanguageEntry {
        language: language.clone(),
        files: None,
        covered: false,
    }));
    entries
}

fn folder_entries(scope: &Scope, counted: Vec<FolderRow>) -> Vec<FolderEntry> {
    let listed: Vec<RelativePath> = counted.iter().map(|row| row.path.clone()).collect();
    let mut entries: Vec<FolderEntry> = counted
        .into_iter()
        .map(|row| FolderEntry {
            covered: scope.covers_folder(&row.path),
            left_out: scope.folders().contains(&row.path),
            files: Some(row.files),
            path: row.path,
            languages: row.languages,
            project_files: row.project_files,
        })
        .collect();
    let unseen = scope
        .folders()
        .iter()
        .filter(|folder| !listed.contains(folder));
    entries.extend(unseen.map(|folder| FolderEntry {
        path: folder.clone(),
        files: None,
        covered: false,
        left_out: true,
        languages: Vec::new(),
        project_files: Vec::new(),
    }));
    entries.sort_by(|a, b| a.path.cmp(&b.path));
    entries
}
