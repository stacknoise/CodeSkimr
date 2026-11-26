//! The preferences of the user for the UI: the theme and the language (ADR 0030, decision 8).
//!
//! They are kept on the server, in one small file in the data folder, so that every browser of the
//! user shows the same. The browser keeps a copy in `localStorage` only so that the first paint
//! needs no request. The file holds no secret; a file that is missing, too large or not valid is
//! the same as no preferences.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use codeskimr_application::access;
use codeskimr_application::error::CodedError;
use codeskimr_application::registry::{Command, CommandDescriptor, CommandId, Request};
use codeskimr_codes::ErrorCode;
use codeskimr_domain::Permission;
use serde::{Deserialize, Serialize};

use super::consts::{
    PREFERENCE_EXTRA_CHAR, PREFERENCE_MAX_CHARS, PREFERENCES_FILE, PREFERENCES_GET,
    PREFERENCES_GET_DESCRIPTION, PREFERENCES_GET_TITLE, PREFERENCES_MAX_BYTES, PREFERENCES_SET,
    PREFERENCES_SET_DESCRIPTION, PREFERENCES_SET_TITLE,
};

/// What the user has chosen. `None` means: no choice, the UI decides (system setting, default).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Preferences {
    /// Name of the theme.
    pub theme: Option<String>,
    /// Language tag of the texts.
    pub language: Option<String>,
}

impl Preferences {
    /// A name the UI could have produced: short, lower case letters, digits and `-`. Whether a
    /// theme or a language of that name exists is the UI's to decide; an unknown name is ignored
    /// there.
    fn is_valid(&self) -> bool {
        [&self.theme, &self.language]
            .into_iter()
            .flatten()
            .all(|name| is_name(name))
    }
}

fn is_name(text: &str) -> bool {
    !text.is_empty()
        && text.chars().count() <= PREFERENCE_MAX_CHARS
        && text
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == PREFERENCE_EXTRA_CHAR)
}

fn unavailable() -> CodedError {
    CodedError::new(ErrorCode::DataDirUnavailable)
}

/// The preferences file in the data folder.
#[derive(Debug, Clone)]
pub struct PreferencesFile {
    path: PathBuf,
}

impl PreferencesFile {
    /// The file of the installation whose data folder is `data_dir`.
    #[must_use]
    pub fn in_dir(data_dir: &Path) -> Self {
        Self {
            path: data_dir.join(PREFERENCES_FILE),
        }
    }

    /// The stored preferences; none if there is no usable file.
    #[must_use]
    pub fn read(&self) -> Preferences {
        let usable = fs::metadata(&self.path)
            .is_ok_and(|meta| meta.is_file() && meta.len() <= PREFERENCES_MAX_BYTES);
        let stored = usable
            .then(|| fs::read_to_string(&self.path).ok())
            .flatten()
            .and_then(|text| serde_json::from_str::<Preferences>(&text).ok())
            .filter(Preferences::is_valid);
        stored.unwrap_or_default()
    }

    /// Replaces the file: the new content is written to a new file beside it (a name that is used
    /// once, created exclusively, so that a link someone put there is never followed) and renamed
    /// over it, so that a reader never sees half a file.
    ///
    /// # Errors
    /// `APP-201` if the data folder cannot be written.
    pub fn write(&self, preferences: &Preferences) -> Result<(), CodedError> {
        let text = serde_json::to_string(preferences).map_err(|_| unavailable())?;
        write_private(&self.path, &text)
    }
}

/// Writes `text` to `path` as the whole new content: to a new file beside it (see
/// [`PreferencesFile::write`]), then renamed over it.
///
/// # Errors
/// `APP-201` if the data folder cannot be written.
pub(super) fn write_private(path: &Path, text: &str) -> Result<(), CodedError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|_| unavailable())?;
    }
    let temp = super::temp_file::beside(path);
    let written = create_private(&temp).and_then(|mut file| file.write_all(text.as_bytes()));
    let moved = written.and_then(|()| fs::rename(&temp, path));
    if moved.is_err() {
        let _ = fs::remove_file(&temp);
    }
    moved.map_err(|_| unavailable())
}

/// Creates `path` for writing; an existing file or link of that name makes it fail.
pub(super) fn create_private(path: &Path) -> std::io::Result<fs::File> {
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, super::consts::PREFERENCES_FILE_MODE);
    options.open(path)
}

/// Returns the preferences of the user.
pub struct GetPreferences {
    pub(super) file: PreferencesFile,
}

impl Command for GetPreferences {
    type Input = ();
    type Output = Preferences;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(PREFERENCES_GET),
        title_key: PREFERENCES_GET_TITLE,
        description_key: PREFERENCES_GET_DESCRIPTION,
        shortcut: None,
        read_only: true,
        permission: Permission::Read,
    };

    fn execute(&self, request: Request<'_, Self>) -> Result<Preferences, CodedError> {
        access::require(request.principal, Permission::Read)?;
        Ok(self.file.read())
    }
}

/// Stores the preferences of the user; both values are replaced, `null` forgets a choice.
pub struct SetPreferences {
    pub(super) file: PreferencesFile,
}

impl Command for SetPreferences {
    type Input = Preferences;
    type Output = Preferences;

    const DESCRIPTOR: CommandDescriptor = CommandDescriptor {
        id: CommandId::new(PREFERENCES_SET),
        title_key: PREFERENCES_SET_TITLE,
        description_key: PREFERENCES_SET_DESCRIPTION,
        shortcut: None,
        read_only: false,
        permission: Permission::Manage,
    };

    /// # Errors
    /// - `API-101` if a name is not a plain lower-case name.
    /// - `APP-201` if the data folder cannot be written.
    fn execute(&self, request: Request<'_, Self>) -> Result<Preferences, CodedError> {
        access::require(request.principal, Permission::Manage)?;
        if !request.input.is_valid() {
            return Err(CodedError::new(ErrorCode::RequestInvalid));
        }
        self.file.write(&request.input)?;
        Ok(request.input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chosen(theme: &str, language: &str) -> Preferences {
        Preferences {
            theme: Some(theme.to_owned()),
            language: Some(language.to_owned()),
        }
    }

    #[test]
    fn nothing_stored_means_no_choice() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(
            PreferencesFile::in_dir(dir.path()).read(),
            Preferences::default()
        );
    }

    #[test]
    fn what_is_written_is_read_back_and_a_choice_can_be_forgotten() {
        let dir = tempfile::tempdir().unwrap();
        let file = PreferencesFile::in_dir(&dir.path().join("data"));
        file.write(&chosen("petrol", "de")).unwrap();
        assert_eq!(file.read(), chosen("petrol", "de"));
        file.write(&Preferences::default()).unwrap();
        assert_eq!(file.read(), Preferences::default());
        let leftovers = fs::read_dir(dir.path().join("data")).unwrap().count();
        assert_eq!(leftovers, 1);
    }

    #[test]
    fn a_damaged_or_foreign_file_is_the_same_as_none() {
        let dir = tempfile::tempdir().unwrap();
        let file = PreferencesFile::in_dir(dir.path());
        let path = dir.path().join(PREFERENCES_FILE);
        for content in [
            "not json",
            "{\"theme\":\"Petrol\"}",
            "{\"theme\":\"a b\"}",
            "{\"other\":1}",
            "{\"language\":\"\"}",
        ] {
            fs::write(&path, content).unwrap();
            assert_eq!(file.read(), Preferences::default(), "{content}");
        }
        fs::write(&path, "x".repeat(10_000)).unwrap();
        assert_eq!(file.read(), Preferences::default());
    }

    #[cfg(unix)]
    #[test]
    fn a_link_at_the_temporary_name_is_never_followed_and_the_file_is_private() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let victim = dir.path().join("victim");
        fs::write(&victim, "keep").unwrap();
        let link = dir.path().join("planted.tmp");
        std::os::unix::fs::symlink(&victim, &link).unwrap();
        assert!(create_private(&link).is_err());
        assert_eq!(fs::read_to_string(&victim).unwrap(), "keep");
        let file = PreferencesFile::in_dir(dir.path());
        file.write(&chosen("petrol", "de")).unwrap();
        file.write(&chosen("hell", "en")).unwrap();
        assert_eq!(file.read(), chosen("hell", "en"));
        let leftovers = fs::read_dir(dir.path()).unwrap().count();
        assert_eq!(leftovers, 3);
        let mode = fs::metadata(dir.path().join(PREFERENCES_FILE))
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600);
    }

    #[test]
    fn only_plain_names_are_valid() {
        assert!(chosen("dunkel", "en").is_valid());
        assert!(chosen("a-b-1", "de-at").is_valid());
        for bad in ["", "Hell", "a b", "../x", "ä", &"a".repeat(33)] {
            assert!(!chosen(bad, "de").is_valid(), "{bad}");
            assert!(!chosen("hell", bad).is_valid(), "{bad}");
        }
    }
}
