//! The layout of an abapGit repository (ADR 0113): the file `.abapgit.xml`, the objects as files
//! (`zcl_a.clas.abap`, `zcl_a.clas.xml`, `zif_b.intf.abap`) and the packages as folders.
//!
//! The functions are pure: they read the text of `.abapgit.xml` and the paths that a listing or a
//! snapshot holds, so they work for a folder and for a Git project alike.

use std::collections::{BTreeMap, BTreeSet};

use crate::consts::{
    ABAPGIT_CONFIG_FILE, ABAPGIT_DEFAULT_STARTING_FOLDER, ABAPGIT_FOLDER_LOGIC_FULL,
    ABAPGIT_FOLDER_LOGIC_TAG, ABAPGIT_LOGIC_FULL_LABEL, ABAPGIT_LOGIC_PREFIX_LABEL,
    ABAPGIT_METADATA_EXTENSION, ABAPGIT_NAME_SEPARATOR, ABAPGIT_NAMESPACE_MARK,
    ABAPGIT_NAMESPACE_SLASH, ABAPGIT_PACKAGE_FILE, ABAPGIT_PACKAGE_TYPE, ABAPGIT_ROOT_FOLDER,
    ABAPGIT_SOURCE_EXTENSIONS, ABAPGIT_STARTING_FOLDER_TAG, ABAPGIT_TAG_CLOSE, ABAPGIT_TAG_END,
    ABAPGIT_TAG_OPEN, PATH_SEPARATOR,
};
use crate::ports::ListedFile;

/// The name of the file that marks an abapGit repository, in the project folder.
pub const CONFIG_FILE: &str = ABAPGIT_CONFIG_FILE;

/// How abapGit turns the name of a package into a folder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FolderLogic {
    /// The folder holds the name of the package without the prefix of its parent package.
    Prefix,
    /// The folder holds the full name of the package.
    Full,
}

impl FolderLogic {
    /// The word for the logic in answers: `prefix` or `full`.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Prefix => ABAPGIT_LOGIC_PREFIX_LABEL,
            Self::Full => ABAPGIT_LOGIC_FULL_LABEL,
        }
    }
}

/// What `.abapgit.xml` says that matters for reading the repository.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbapGitConfig {
    /// The folder below which the objects lie, relative to the project, without slashes at the
    /// ends; empty for the project folder itself.
    pub starting_folder: String,
    /// How packages are turned into folders.
    pub folder_logic: FolderLogic,
}

impl Default for AbapGitConfig {
    /// What abapGit uses when the file does not say: `/src/` and the prefix logic.
    fn default() -> Self {
        Self {
            starting_folder: ABAPGIT_DEFAULT_STARTING_FOLDER.to_owned(),
            folder_logic: FolderLogic::Prefix,
        }
    }
}

/// The text between `<tag>` and `</tag>`, trimmed. The file is machine written and flat, so a
/// search for the tags is enough; nothing is parsed as XML and nothing is resolved.
fn tag_text<'a>(text: &'a str, tag: &str) -> Option<&'a str> {
    let open = [ABAPGIT_TAG_OPEN, tag, ABAPGIT_TAG_END].concat();
    let close = [ABAPGIT_TAG_CLOSE, tag, ABAPGIT_TAG_END].concat();
    let start = text.find(&open)?.checked_add(open.len())?;
    let rest = text.get(start..)?;
    let end = rest.find(&close)?;
    rest.get(..end).map(str::trim)
}

/// The configuration in the text of `.abapgit.xml`; what the file does not name is the default.
#[must_use]
pub fn parse_config(text: &str) -> AbapGitConfig {
    let defaults = AbapGitConfig::default();
    let starting_folder =
        tag_text(text, ABAPGIT_STARTING_FOLDER_TAG).map_or(defaults.starting_folder, |folder| {
            folder
                .trim_matches(PATH_SEPARATOR)
                .trim_matches(ABAPGIT_NAMESPACE_SLASH)
                .to_owned()
        });
    let folder_logic = match tag_text(text, ABAPGIT_FOLDER_LOGIC_TAG) {
        Some(logic) if logic.eq_ignore_ascii_case(ABAPGIT_FOLDER_LOGIC_FULL) => FolderLogic::Full,
        _ => defaults.folder_logic,
    };
    AbapGitConfig {
        starting_folder,
        folder_logic,
    }
}

/// The package that the explorer shows for files in `folder`: the folder relative to the starting
/// folder, so `src/zfoo/zbar` below `src` is `zfoo/zbar` and a file directly in `src` is in the
/// root package (empty). A folder that does not lie below the starting folder stays as it is.
#[must_use]
pub fn package_of(folder: &str, starting_folder: &str) -> String {
    if starting_folder.is_empty() {
        return folder.to_owned();
    }
    if folder == starting_folder {
        return ABAPGIT_ROOT_FOLDER.to_owned();
    }
    folder
        .strip_prefix(starting_folder)
        .and_then(|rest| rest.strip_prefix(PATH_SEPARATOR))
        .unwrap_or(folder)
        .to_owned()
}

/// An object that a file of the repository belongs to.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ObjectRef {
    /// Name of the object as ABAP writes it: the `#` of a namespace is a slash again.
    pub name: String,
    /// Type of the object in upper case (`CLAS`, `INTF`, `PROG`, `FUGR`, `TABL`).
    pub kind: String,
}

/// What a file name says about the file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectFile {
    /// The object the file belongs to.
    pub object: ObjectRef,
    /// Whether the file holds source (`.abap`, `.asddls`); the XML files hold metadata.
    pub is_source: bool,
}

/// The object of a file name such as `zcl_a.clas.testclasses.abap`, `zcl_a.clas.xml` or
/// `#ns#name.prog.abap`. `None` for a file that is no part of an object: no object type, an
/// extension that abapGit does not write, or `package.devc.xml`, which describes a package.
#[must_use]
pub fn object_of(file_name: &str) -> Option<ObjectFile> {
    let mut parts = file_name.split(ABAPGIT_NAME_SEPARATOR);
    let name = parts.next().filter(|name| !name.is_empty())?;
    let kind = parts.next().filter(|kind| !kind.is_empty())?;
    let extension = parts.next_back()?;
    let is_source = ABAPGIT_SOURCE_EXTENSIONS.contains(&extension);
    if !is_source && extension != ABAPGIT_METADATA_EXTENSION {
        return None;
    }
    if kind.eq_ignore_ascii_case(ABAPGIT_PACKAGE_TYPE) {
        return None;
    }
    Some(ObjectFile {
        object: ObjectRef {
            name: name.replace(ABAPGIT_NAMESPACE_MARK, &ABAPGIT_NAMESPACE_SLASH.to_string()),
            kind: kind.to_uppercase(),
        },
        is_source,
    })
}

/// What an abapGit repository holds, for the project page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbapGitSummary {
    /// The starting folder that was used.
    pub starting_folder: String,
    /// How packages are turned into folders.
    pub folder_logic: FolderLogic,
    /// Whether `.abapgit.xml` was read; without it the defaults were used.
    pub configured: bool,
    /// Objects by type, the type with the most objects first, then by name of the type.
    pub objects: Vec<(String, u64)>,
    /// Number of packages: folders below the starting folder that hold an object or a
    /// `package.devc.xml`.
    pub packages: u64,
    /// Number of files that hold source.
    pub source_files: u64,
}

/// The summary of the repository in `files`, or `None` if it is none.
///
/// A repository is found by its `.abapgit.xml` in the project folder, or, where that file is not
/// among the files (a Git project lists only files of known languages), by at least one source
/// file of an object below the default starting folder. `config` is the configuration that was
/// read, `None` if there was none to read.
#[must_use]
pub fn summarize<'a>(
    files: impl IntoIterator<Item = &'a ListedFile>,
    config: Option<&AbapGitConfig>,
) -> Option<AbapGitSummary> {
    let listed: Vec<&ListedFile> = files.into_iter().collect();
    let marked = listed
        .iter()
        .any(|file| file.path.as_str() == ABAPGIT_CONFIG_FILE);
    let used = config.cloned().unwrap_or_default();
    let prefix = folder_prefix(&used.starting_folder);
    let mut objects: BTreeSet<ObjectRef> = BTreeSet::new();
    let mut packages: BTreeSet<String> = BTreeSet::new();
    let mut source_files = 0;
    for file in &listed {
        let Some(inside) = file.path.as_str().strip_prefix(prefix.as_str()) else {
            continue;
        };
        let (folder, name) = inside
            .rsplit_once(PATH_SEPARATOR)
            .map_or((ABAPGIT_ROOT_FOLDER, inside), |(folder, name)| {
                (folder, name)
            });
        let is_package = name == ABAPGIT_PACKAGE_FILE;
        let object = object_of(name);
        if is_package || object.is_some() {
            packages.insert(folder.to_owned());
        }
        if let Some(found) = object {
            source_files += u64::from(found.is_source);
            objects.insert(found.object);
        }
    }
    let found_by_source = source_files > 0 && config.is_none();
    (marked || found_by_source).then(|| AbapGitSummary {
        starting_folder: used.starting_folder,
        folder_logic: used.folder_logic,
        configured: config.is_some(),
        objects: by_type(&objects),
        packages: u64::try_from(packages.len()).unwrap_or(u64::MAX),
        source_files,
    })
}

/// The start of every path below the starting folder: the folder and a separator, or nothing for
/// the project folder itself.
fn folder_prefix(starting_folder: &str) -> String {
    if starting_folder.is_empty() {
        String::new()
    } else {
        format!("{starting_folder}{PATH_SEPARATOR}")
    }
}

/// The number of objects of each type, the type with the most objects first.
fn by_type(objects: &BTreeSet<ObjectRef>) -> Vec<(String, u64)> {
    let mut counts: BTreeMap<&str, u64> = BTreeMap::new();
    for object in objects {
        *counts.entry(object.kind.as_str()).or_default() += 1;
    }
    let mut list: Vec<(String, u64)> = counts
        .into_iter()
        .map(|(kind, count)| (kind.to_owned(), count))
        .collect();
    list.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    list
}
