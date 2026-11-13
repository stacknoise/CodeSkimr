//! Where `CodeSkimr` keeps its data, its configuration and its language packs.
//!
//! Order for each location: command line flag, environment variable, platform default.

use std::path::{Path, PathBuf};

use codeskimr_application::error::CodedError;
use codeskimr_cli::GlobalOptions;
use codeskimr_codes::ErrorCode;

use crate::consts::{
    APP_DIR, DATABASE_FILE, ENV_APPDATA, ENV_CONFIG, ENV_DATA_DIR, ENV_HOME, ENV_LANGUAGES_DIR,
    ENV_LOCAL_APPDATA, ENV_XDG_CONFIG_HOME, ENV_XDG_DATA_HOME, GIT_CACHE_DIR, HOME_CONFIG_PARTS,
    HOME_DATA_PARTS, HOME_MACOS_PARTS, LANGUAGES_DIR, OS_MACOS, USER_CONFIG_FILE,
};

/// Reads one environment variable; the real one in `main`, a table in tests.
pub type Lookup<'a> = &'a dyn Fn(&str) -> Option<String>;

/// The user configuration file and whether it has to exist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserConfig {
    /// Path of the file.
    pub path: PathBuf,
    /// True if the user named it (flag or environment): a missing file is then an error.
    pub required: bool,
}

/// All folders and files the application reads or writes outside of the scanned source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Locations {
    /// Folder for the database and the Git cache.
    pub data_dir: PathBuf,
    /// Folder with the language packs.
    pub languages_dir: PathBuf,
    /// The user configuration file, if any location is known.
    pub user_config: Option<UserConfig>,
    /// Where a relative folder such as the built UI is looked up: next to the program, then in
    /// the current folder.
    pub bases: Vec<PathBuf>,
}

/// The platform folders that differ between data and configuration.
struct PlatformBase {
    windows_var: &'static str,
    xdg_var: &'static str,
    home_parts: &'static [&'static str],
}

const DATA_BASE: PlatformBase = PlatformBase {
    windows_var: ENV_LOCAL_APPDATA,
    xdg_var: ENV_XDG_DATA_HOME,
    home_parts: &HOME_DATA_PARTS,
};

const CONFIG_BASE: PlatformBase = PlatformBase {
    windows_var: ENV_APPDATA,
    xdg_var: ENV_XDG_CONFIG_HOME,
    home_parts: &HOME_CONFIG_PARTS,
};

impl Locations {
    /// Resolves the locations. Nothing is created or read here.
    ///
    /// # Errors
    /// `APP-201` if no data folder can be determined.
    pub fn resolve(
        global: &GlobalOptions,
        env: Lookup<'_>,
        executable_dir: Option<&Path>,
        current_dir: &Path,
    ) -> Result<Self, CodedError> {
        let data_dir = global
            .data_dir
            .clone()
            .or_else(|| env(ENV_DATA_DIR).map(PathBuf::from))
            .or_else(|| platform_dir(env, &DATA_BASE))
            .ok_or_else(|| CodedError::new(ErrorCode::DataDirUnavailable))?;
        Ok(Self {
            data_dir,
            languages_dir: languages_dir(global, env, executable_dir, current_dir),
            user_config: user_config(global, env),
            bases: vec![
                executable_dir.unwrap_or(current_dir).to_path_buf(),
                current_dir.to_path_buf(),
            ],
        })
    }

    /// The database file.
    #[must_use]
    pub fn database(&self) -> PathBuf {
        self.data_dir.join(DATABASE_FILE)
    }

    /// The folder `configured` names: an absolute path as it is, a relative one below the first
    /// base where it exists. `None` if there is no such folder; the server then has no UI.
    #[must_use]
    pub fn existing_dir(&self, configured: &str) -> Option<PathBuf> {
        let path = Path::new(configured);
        if path.is_absolute() {
            return path.is_dir().then(|| path.to_path_buf());
        }
        self.bases
            .iter()
            .map(|base| base.join(path))
            .find(|folder| folder.is_dir())
    }

    /// The folder of the Git clones.
    #[must_use]
    pub fn git_cache(&self) -> PathBuf {
        self.data_dir.join(GIT_CACHE_DIR)
    }
}

fn languages_dir(
    global: &GlobalOptions,
    env: Lookup<'_>,
    executable_dir: Option<&Path>,
    current_dir: &Path,
) -> PathBuf {
    global
        .languages_dir
        .clone()
        .or_else(|| env(ENV_LANGUAGES_DIR).map(PathBuf::from))
        .or_else(|| {
            [executable_dir.unwrap_or(current_dir), current_dir]
                .into_iter()
                .map(|base| base.join(LANGUAGES_DIR))
                .find(|folder| folder.is_dir())
        })
        .unwrap_or_else(|| executable_dir.unwrap_or(current_dir).join(LANGUAGES_DIR))
}

fn user_config(global: &GlobalOptions, env: Lookup<'_>) -> Option<UserConfig> {
    global
        .config_file
        .clone()
        .or_else(|| env(ENV_CONFIG).map(PathBuf::from))
        .map(|path| UserConfig {
            path,
            required: true,
        })
        .or_else(|| {
            platform_dir(env, &CONFIG_BASE).map(|base| UserConfig {
                path: base.join(USER_CONFIG_FILE),
                required: false,
            })
        })
}

/// The per-user folder of `CodeSkimr` for data or configuration on this platform.
fn platform_dir(env: Lookup<'_>, base: &PlatformBase) -> Option<PathBuf> {
    let root = if cfg!(windows) {
        env(base.windows_var).map(PathBuf::from)
    } else if std::env::consts::OS == OS_MACOS {
        home(env, &HOME_MACOS_PARTS)
    } else {
        env(base.xdg_var)
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .or_else(|| home(env, base.home_parts))
    };
    root.map(|folder| folder.join(APP_DIR))
}

fn home(env: Lookup<'_>, parts: &[&str]) -> Option<PathBuf> {
    env(ENV_HOME).filter(|value| !value.is_empty()).map(|home| {
        parts
            .iter()
            .fold(PathBuf::from(home), |path, part| path.join(part))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn none(_: &str) -> Option<String> {
        None
    }

    fn resolve(global: &GlobalOptions, env: Lookup<'_>) -> Result<Locations, CodedError> {
        Locations::resolve(global, env, None, Path::new("/nowhere"))
    }

    #[test]
    fn a_flag_beats_the_environment() {
        let global = GlobalOptions {
            data_dir: Some(PathBuf::from("/flag")),
            ..GlobalOptions::default()
        };
        let env = |name: &str| (name == ENV_DATA_DIR).then(|| "/env".to_owned());
        assert_eq!(resolve(&global, &env).unwrap().data_dir, Path::new("/flag"));
    }

    #[test]
    fn the_environment_beats_the_platform_default() {
        let env = |name: &str| (name == ENV_DATA_DIR).then(|| "/env".to_owned());
        let found = resolve(&GlobalOptions::default(), &env).unwrap();
        assert_eq!(found.data_dir, Path::new("/env"));
        assert_eq!(found.database(), Path::new("/env").join(DATABASE_FILE));
    }

    #[test]
    fn without_any_hint_there_is_no_data_folder() {
        let error = resolve(&GlobalOptions::default(), &none).unwrap_err();
        assert_eq!(
            codeskimr_codes::Coded::code(&error),
            ErrorCode::DataDirUnavailable
        );
    }

    #[test]
    fn a_named_config_file_is_required_a_default_one_is_not() {
        let named = |name: &str| match name {
            ENV_DATA_DIR => Some("/d".to_owned()),
            ENV_CONFIG => Some("/c.toml".to_owned()),
            _ => None,
        };
        let found = resolve(&GlobalOptions::default(), &named).unwrap();
        let expected = UserConfig {
            path: PathBuf::from("/c.toml"),
            required: true,
        };
        assert_eq!(found.user_config, Some(expected));
        let only_data = |name: &str| (name == ENV_DATA_DIR).then(|| "/d".to_owned());
        let default = resolve(&GlobalOptions::default(), &only_data).unwrap();
        assert!(default.user_config.is_none_or(|config| !config.required));
    }
}
