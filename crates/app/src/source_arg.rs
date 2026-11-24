//! Turns the `<path|git-url>` argument of `scan` into a project source and a name.

use std::fs;
use std::path::Path;

use codeskimr_application::error::CodedError;
use codeskimr_codes::ErrorCode;
use codeskimr_domain::{BranchName, CredentialId, FolderPath, GitUrl, ProjectName, ProjectSource};

use crate::consts::{
    AUTHORITY_END, BRANCH_FORBIDDEN, BRANCH_FORBIDDEN_ENDS, BRANCH_FORBIDDEN_PARTS,
    BRANCH_SEPARATOR, DEFAULT_BRANCH, GENERIC_FOLDER_NAMES, GIT_URL_PREFIXES, GIT_URL_SUFFIX,
    HTTP_SCHEME, MAX_BRANCH_CHARS, MAX_GIT_URL_CHARS, MAX_PROJECT_NAME_CHARS, NAME_PART_SEPARATOR,
    NO_SCHEME, OPTION_MARKER, PASSWORD_SEPARATOR, QUERY_MARKERS, SCHEME_SEPARATOR,
    SCP_PATH_SEPARATOR, TRANSPORT_HELPER_MARKER, URL_PATH_SEPARATOR, USERINFO_MARKER,
    VERBATIM_PREFIX, VERBATIM_UNC, is_reserved_credential,
};

/// Why an argument is not a source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceError {
    /// Neither an existing folder nor a Git URL.
    NotASource,
    /// A Git URL with credentials, a query or a fragment in it.
    HasCredentials,
    /// A credential was named for a folder, or its name is empty.
    CredentialNotUsable,
    /// The display name is empty, has control characters or is too long.
    NameInvalid,
}

impl From<SourceError> for CodedError {
    fn from(error: SourceError) -> Self {
        match error {
            SourceError::NameInvalid => CodedError::new(ErrorCode::ProjectNameInvalid),
            _ => invalid(),
        }
    }
}

/// A source to register and scan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSpec {
    /// Where the code comes from.
    pub source: ProjectSource,
    /// Display name of the project.
    pub name: ProjectName,
    /// The user chose the name; a project that is registered already takes it over.
    pub named: bool,
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

impl SourceSpec {
    /// The same source under a name the user chose.
    ///
    /// # Errors
    /// [`SourceError::NameInvalid`] (`PRJ-102`) if the name is empty, has control characters or is
    /// too long.
    pub fn with_name(mut self, name: &str) -> Result<Self, SourceError> {
        self.name = display_name(name)?;
        self.named = true;
        Ok(self)
    }

    /// The same source, logging in with the stored credential `name`.
    ///
    /// # Errors
    /// [`SourceError::CredentialNotUsable`] for a folder source or an empty name.
    pub fn with_credential(mut self, name: &str) -> Result<Self, SourceError> {
        if is_reserved_credential(name) {
            // The token that guards the local UI must never be sent to a Git server.
            return Err(SourceError::CredentialNotUsable);
        }
        let id = CredentialId::new(name).map_err(|_| SourceError::CredentialNotUsable)?;
        match &mut self.source {
            ProjectSource::Git { credential, .. } => *credential = Some(id),
            ProjectSource::Folder(_) => return Err(SourceError::CredentialNotUsable),
        }
        Ok(self)
    }
}

/// Classifies `argument`: an existing folder wins over a Git URL.
///
/// The branch only applies to Git sources; for a folder it is ignored.
///
/// # Errors
/// [`SourceError`] (`PRJ-101`) if it is neither an existing folder nor a Git URL without secrets.
pub fn classify(argument: &str, branch: Option<&str>) -> Result<SourceSpec, SourceError> {
    if let Ok(folder) = fs::canonicalize(argument) {
        return folder_spec(&folder).map_err(|_| SourceError::NotASource);
    }
    if is_git_url(argument) {
        return git_spec(argument, branch.unwrap_or(DEFAULT_BRANCH));
    }
    Err(SourceError::NotASource)
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
    let text = display_path(&canonical.to_string_lossy());
    let name = default_folder_name(&canonical).unwrap_or_else(|| text.clone());
    Ok(SourceSpec {
        source: ProjectSource::Folder(FolderPath::new(text).map_err(|_| invalid())?),
        name: ProjectName::new(name).map_err(|_| invalid())?,
        named: false,
    })
}

/// The display name that the user typed: trimmed, not empty, one line and not too long.
///
/// # Errors
/// [`SourceError::NameInvalid`] (`PRJ-102`) otherwise.
pub fn display_name(text: &str) -> Result<ProjectName, SourceError> {
    let text = text.trim();
    if text.chars().count() > MAX_PROJECT_NAME_CHARS || text.chars().any(char::is_control) {
        return Err(SourceError::NameInvalid);
    }
    ProjectName::new(text).map_err(|_| SourceError::NameInvalid)
}

/// The default name of a folder: its own name, or with the parent's name in front if the folder
/// is called something like `src`, which would name many projects alike.
fn default_folder_name(folder: &Path) -> Option<String> {
    let own = folder.file_name()?.to_string_lossy().into_owned();
    let generic = GENERIC_FOLDER_NAMES
        .iter()
        .any(|name| name.eq_ignore_ascii_case(&own));
    let parent = folder.parent().and_then(Path::file_name);
    match parent {
        Some(parent) if generic => Some(format!(
            "{}{NAME_PART_SEPARATOR}{own}",
            parent.to_string_lossy()
        )),
        _ => Some(own),
    }
}

/// Windows answers `canonicalize` with a `\\?\C:\…` path; the plain form is what users know.
/// A UNC path (`\\?\UNC\…`) keeps its prefix.
pub(crate) fn display_path(path: &str) -> String {
    path.strip_prefix(VERBATIM_PREFIX)
        .filter(|rest| !rest.starts_with(VERBATIM_UNC))
        .unwrap_or(path)
        .to_owned()
}

/// The form of a path that the system opener takes: like [`display_path`], but a UNC path loses its
/// verbatim prefix too (`\\?\UNC\srv\share` becomes `\\srv\share`).
#[cfg(any(windows, test))]
pub(crate) fn opener_form(path: &str) -> String {
    use crate::consts::UNC_PREFIX;

    path.strip_prefix(VERBATIM_PREFIX)
        .and_then(|rest| rest.strip_prefix(VERBATIM_UNC))
        .map_or_else(|| display_path(path), |unc| [UNC_PREFIX, unc].concat())
}

/// Whether `text` could be read as an option, holds control characters or blanks, or is too long.
/// What goes to `git` and `ssh` as an argument must be none of these.
fn is_unsafe_argument(text: &str, longest: usize) -> bool {
    text.starts_with(OPTION_MARKER)
        || text.chars().count() > longest
        || text.chars().any(|c| c.is_control() || c.is_whitespace())
}

/// Whether the host of `url` could be read as an option: `ssh://-oProxyCommand=…`,
/// `user@-oProxyCommand=…:path`. The part after the last `@` of the authority is the host.
fn has_option_host(url: &str) -> bool {
    let rest = url
        .split_once(SCHEME_SEPARATOR)
        .map_or(url, |(_, rest)| rest);
    let authority = rest.split(URL_PATH_SEPARATOR).next().unwrap_or(rest);
    let host = authority
        .rsplit(USERINFO_MARKER)
        .next()
        .unwrap_or(authority);
    host.starts_with(OPTION_MARKER)
}

/// The rules of `git check-ref-format` that matter for an argument, plus [`is_unsafe_argument`].
fn is_bad_branch(branch: &str) -> bool {
    is_unsafe_argument(branch, MAX_BRANCH_CHARS)
        || BRANCH_FORBIDDEN_PARTS
            .iter()
            .any(|part| branch.contains(part))
        || branch.contains(BRANCH_FORBIDDEN)
        || BRANCH_FORBIDDEN_ENDS
            .iter()
            .any(|end| branch.ends_with(end))
        || branch.starts_with(BRANCH_SEPARATOR)
}

/// A transport helper (`ext::sh -c …`) in the first part of the address.
fn has_transport_helper(url: &str) -> bool {
    let head = url.split(URL_PATH_SEPARATOR).next().unwrap_or(url);
    head.contains(TRANSPORT_HELPER_MARKER)
}

/// The default name of a repository: the last part of its address without `.git`.
fn git_default_name(url: &str) -> &str {
    url.trim_end_matches(URL_PATH_SEPARATOR)
        .trim_end_matches(GIT_URL_SUFFIX)
        .rsplit([URL_PATH_SEPARATOR, SCP_PATH_SEPARATOR])
        .next()
        .unwrap_or(url)
}

/// The name a project gets if the user chose none: what [`classify`] makes of its source.
#[must_use]
pub fn default_name(source: &ProjectSource) -> Option<ProjectName> {
    let name = match source {
        ProjectSource::Folder(folder) => default_folder_name(Path::new(folder.as_str()))?,
        ProjectSource::Git { url, .. } => git_default_name(url.as_str()).to_owned(),
    };
    ProjectName::new(name).ok()
}

fn git_spec(url: &str, branch: &str) -> Result<SourceSpec, SourceError> {
    if has_credentials(url) {
        return Err(SourceError::HasCredentials);
    }
    if is_unsafe_argument(url, MAX_GIT_URL_CHARS)
        || has_transport_helper(url)
        || has_option_host(url)
        || is_bad_branch(branch)
    {
        return Err(SourceError::NotASource);
    }
    let rejected = |_| SourceError::NotASource;
    let name = git_default_name(url);
    Ok(SourceSpec {
        source: ProjectSource::Git {
            url: GitUrl::new(url).map_err(rejected)?,
            branch: BranchName::new(branch).map_err(rejected)?,
            credential: None,
        },
        name: ProjectName::new(name).map_err(rejected)?,
        named: false,
    })
}

fn is_git_url(argument: &str) -> bool {
    let lower = argument.to_ascii_lowercase();
    GIT_URL_PREFIXES
        .iter()
        .any(|prefix| lower.starts_with(prefix))
        || lower.ends_with(GIT_URL_SUFFIX)
}

/// Secrets in a URL would end up in the database and on the screen (blueprint 12): a password
/// anywhere, any user info for HTTP(S), and a query or fragment, which is where tokens go.
/// A bare user name for SSH (`ssh://git@host/...`, `git@host:org/repo`) is not a secret.
fn has_credentials(url: &str) -> bool {
    let (scheme, rest) = url.split_once(SCHEME_SEPARATOR).unwrap_or((NO_SCHEME, url));
    let authority = rest.split(AUTHORITY_END).next().unwrap_or_default();
    let secret_user_info = authority
        .rsplit_once(USERINFO_MARKER)
        .is_some_and(|(user_info, _)| {
            scheme.to_ascii_lowercase().starts_with(HTTP_SCHEME)
                || user_info.contains(PASSWORD_SEPARATOR)
        });
    secret_user_info || url.contains(QUERY_MARKERS)
}

fn invalid() -> CodedError {
    CodedError::new(ErrorCode::ProjectSourceInvalid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::TOKEN_CREDENTIAL_ID;

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
    fn a_generic_folder_name_gets_the_name_of_its_parent_in_front() {
        let root = tempfile::tempdir().unwrap();
        let generic = root.path().join("shop").join("SRC");
        let plain = root.path().join("shop").join("billing");
        fs::create_dir_all(&generic).unwrap();
        fs::create_dir_all(&plain).unwrap();
        let name = |folder: &Path| folder_spec(folder).unwrap().name.to_string();
        assert_eq!(name(&generic), "shop/SRC");
        assert_eq!(name(&plain), "billing");
    }

    #[test]
    fn a_chosen_name_is_trimmed_and_checked() {
        let spec = classify("https://example.com/org/repo.git", None).unwrap();
        let named = spec.clone().with_name("  Shop  ").unwrap();
        assert_eq!((named.name.as_str(), named.named), ("Shop", true));
        for bad in ["", "   ", "a\nb", &"x".repeat(MAX_PROJECT_NAME_CHARS + 1)] {
            assert_eq!(spec.clone().with_name(bad), Err(SourceError::NameInvalid));
        }
    }

    #[test]
    fn the_branch_defaults_to_main() {
        let spec = classify("https://example.com/org/repo.git", None).unwrap();
        assert!(
            matches!(spec.source, ProjectSource::Git { branch, .. } if branch.as_str() == DEFAULT_BRANCH)
        );
    }

    #[test]
    fn the_installation_token_is_never_a_git_credential() {
        let spec = classify("https://example.com/org/repo.git", None).unwrap();
        assert!(spec.clone().with_credential("work").is_ok());
        assert_eq!(
            spec.clone().with_credential(TOKEN_CREDENTIAL_ID),
            Err(SourceError::CredentialNotUsable)
        );
        // A keystore may fold case and blanks together: those spellings are the token too.
        let variants = [
            "Installation-Token",
            " installation-token",
            "work.refresh",
            " Work.REFRESH ",
        ];
        for variant in variants {
            assert_eq!(
                spec.clone().with_credential(variant),
                Err(SourceError::CredentialNotUsable),
                "{variant:?}"
            );
        }
    }

    #[test]
    fn addresses_and_branches_that_could_act_as_options_or_commands_are_refused() {
        for url in [
            "--upload-pack=touch /tmp/x.git",
            "-oProxyCommand=evil.git",
            "ext::sh -c touch% /tmp/x.git",
            "ssh://-oProxyCommand=evil/r.git",
            "git@-oProxyCommand=evil:org/r.git",
            "https://example.com/a b.git",
            "https://example.com/a\nb.git",
        ] {
            assert!(classify(url, None).is_err(), "{url:?}");
        }
        let long = format!("https://example.com/{}.git", "a".repeat(3000));
        assert!(classify(&long, None).is_err());
        for branch in [
            "-x", "a..b", "a b", "a~1", "a:b", "a\\b", "a\u{7}b", "a*", "a@{1}", "a.lock", "a/",
            "a.", "/a", "a//b",
        ] {
            assert!(
                classify("https://example.com/o/r.git", Some(branch)).is_err(),
                "{branch:?}"
            );
        }
        assert!(classify("https://example.com/o/r.git", Some("feature/x-1")).is_ok());
    }

    #[test]
    fn credentials_in_http_urls_are_refused_but_ssh_user_names_are_not() {
        assert!(classify("https://token@example.com/a/b.git", None).is_err());
        assert!(classify("https://user:pw@example.com/a/b.git", None).is_err());
        assert!(classify("ssh://user:pw@example.com/a/b.git", None).is_err());
        assert!(classify("ssh://git@example.com/a/b.git", None).is_ok());
        assert!(classify("git@example.com:org/repo.git", None).is_ok());
    }

    #[test]
    fn secrets_are_found_in_every_spelling() {
        for url in [
            "HTTPS://token@example.com/a.git",
            "user:pw@example.com/org/repo.git",
            "https://example.com/a.git?token=abc",
            "https://example.com/a.git#frag",
        ] {
            assert_eq!(
                classify(url, None),
                Err(SourceError::HasCredentials),
                "{url}"
            );
        }
    }

    #[test]
    fn a_windows_verbatim_prefix_is_dropped_but_not_for_unc() {
        assert_eq!(display_path("\\\\?\\C:\\work\\app"), "C:\\work\\app");
        assert_eq!(
            display_path("\\\\?\\UNC\\srv\\share"),
            "\\\\?\\UNC\\srv\\share"
        );
    }

    #[test]
    fn the_opener_gets_a_plain_path_also_for_unc() {
        assert_eq!(opener_form("\\\\?\\C:\\work\\app"), "C:\\work\\app");
        assert_eq!(
            opener_form("\\\\?\\UNC\\srv\\share\\launch"),
            "\\\\srv\\share\\launch"
        );
        assert_eq!(opener_form("C:\\plain"), "C:\\plain");
    }

    #[test]
    fn something_that_is_no_folder_and_no_url_is_invalid() {
        assert!(classify("/no/such/folder", None).is_err());
    }
}
