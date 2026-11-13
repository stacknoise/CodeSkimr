//! Turns the `<path|git-url>` argument of `scan` into a project source and a name.

use std::fs;
use std::path::Path;

use codeskimr_application::error::CodedError;
use codeskimr_codes::ErrorCode;
use codeskimr_domain::{BranchName, FolderPath, GitUrl, ProjectName, ProjectSource};

use crate::consts::{
    DEFAULT_BRANCH, GIT_URL_PREFIXES, GIT_URL_SUFFIX, HTTP_SCHEME, PASSWORD_SEPARATOR,
    SCHEME_SEPARATOR, SCP_PATH_SEPARATOR, URL_PATH_SEPARATOR, USERINFO_MARKER,
};

/// A source to register and scan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSpec {
    /// Where the code comes from.
    pub source: ProjectSource,
    /// Display name of the project.
    pub name: ProjectName,
}

impl SourceSpec {
    /// The folder or URL as shown to the user. A Git URL never carries credentials here.
    #[must_use]
    pub fn location(&self) -> &str {
        match &self.source {
            ProjectSource::Folder(folder) => folder.as_str(),
            ProjectSource::Git { url, .. } => url.as_str(),
        }
    }
}

/// Classifies `argument`: an existing folder wins over a Git URL.
///
/// # Errors
/// `PRJ-101` if it is neither an existing folder nor a Git URL without credentials.
pub fn classify(argument: &str, branch: Option<&str>) -> Result<SourceSpec, CodedError> {
    if let Ok(folder) = fs::canonicalize(argument) {
        return folder_spec(&folder);
    }
    if is_git_url(argument) {
        return git_spec(argument, branch.unwrap_or(DEFAULT_BRANCH));
    }
    Err(invalid())
}

/// The spec of a folder that exists; the path is made absolute so that every run finds the project.
///
/// # Errors
/// `PRJ-101` if the path is not a folder.
pub fn folder_spec(folder: &Path) -> Result<SourceSpec, CodedError> {
    let canonical = fs::canonicalize(folder).map_err(|_| invalid())?;
    if !canonical.is_dir() {
        return Err(invalid());
    }
    let text = canonical.to_string_lossy().into_owned();
    let name = canonical
        .file_name()
        .map_or_else(|| text.clone(), |name| name.to_string_lossy().into_owned());
    Ok(SourceSpec {
        source: ProjectSource::Folder(FolderPath::new(text).map_err(|_| invalid())?),
        name: ProjectName::new(name).map_err(|_| invalid())?,
    })
}

fn git_spec(url: &str, branch: &str) -> Result<SourceSpec, CodedError> {
    if has_credentials(url) {
        return Err(invalid());
    }
    let name = url
        .trim_end_matches(URL_PATH_SEPARATOR)
        .trim_end_matches(GIT_URL_SUFFIX)
        .rsplit([URL_PATH_SEPARATOR, SCP_PATH_SEPARATOR])
        .next()
        .unwrap_or(url);
    Ok(SourceSpec {
        source: ProjectSource::Git {
            url: GitUrl::new(url).map_err(|_| invalid())?,
            branch: BranchName::new(branch).map_err(|_| invalid())?,
        },
        name: ProjectName::new(name).map_err(|_| invalid())?,
    })
}

fn is_git_url(argument: &str) -> bool {
    GIT_URL_PREFIXES
        .iter()
        .any(|prefix| argument.starts_with(prefix))
        || argument.ends_with(GIT_URL_SUFFIX)
}

/// Credentials in a URL would end up in the database and on the screen (blueprint 12).
/// A bare user name for SSH (`ssh://git@host/...`) is not a secret; anything for HTTP is.
fn has_credentials(url: &str) -> bool {
    let Some((scheme, rest)) = url.split_once(SCHEME_SEPARATOR) else {
        return false;
    };
    let authority = rest.split(URL_PATH_SEPARATOR).next().unwrap_or_default();
    authority
        .rsplit_once(USERINFO_MARKER)
        .is_some_and(|(userinfo, _)| {
            scheme.starts_with(HTTP_SCHEME) || userinfo.contains(PASSWORD_SEPARATOR)
        })
}

fn invalid() -> CodedError {
    CodedError::new(ErrorCode::ProjectSourceInvalid)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn name_of(url: &str) -> String {
        classify(url, None).unwrap().name.to_string()
    }

    #[test]
    fn git_urls_are_recognised_and_named_after_the_repository() {
        assert_eq!(name_of("https://example.com/org/repo.git"), "repo");
        assert_eq!(name_of("git@example.com:org/tool.git"), "tool");
        assert_eq!(name_of("https://example.com/org/plain/"), "plain");
    }

    #[test]
    fn the_branch_defaults_to_main() {
        let spec = classify("https://example.com/org/repo.git", None).unwrap();
        assert!(
            matches!(spec.source, ProjectSource::Git { branch, .. } if branch.as_str() == DEFAULT_BRANCH)
        );
    }

    #[test]
    fn credentials_in_http_urls_are_refused_but_ssh_user_names_are_not() {
        assert!(classify("https://token@example.com/a/b.git", None).is_err());
        assert!(classify("https://user:pw@example.com/a/b.git", None).is_err());
        assert!(classify("ssh://user:pw@example.com/a/b.git", None).is_err());
        assert!(classify("ssh://git@example.com/a/b.git", None).is_ok());
    }

    #[test]
    fn something_that_is_no_folder_and_no_url_is_invalid() {
        assert!(classify("/no/such/folder", None).is_err());
    }
}
