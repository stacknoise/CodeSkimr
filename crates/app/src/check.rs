//! `codeskimr check`: runs the convention checks over the current folder.

use std::fs;
use std::path::{Path, PathBuf};

use codeskimr_application::conventions::{CheckReport, CheckRequest, ConventionChecker};
use codeskimr_application::error::CodedError;
use codeskimr_cli::{CheckArgs, GlobalOptions};
use codeskimr_codes::ErrorCode;
use codeskimr_domain::FolderPath;

use crate::config_files;
use crate::locations::Locations;
use crate::principal;
use crate::wiring::{Analysis, build_analysis};

/// Checks the files named in `args` (everything if none) below `current_dir`.
///
/// Nothing is stored: the files are analysed afresh, so the result shows the code as it is now,
/// not as the last scan saw it. The project's `.codeskimr.toml` in `current_dir` applies.
///
/// # Errors
/// - `PRJ-101` if the current folder cannot be used as a project root.
/// - Any coded failure of the configuration, the language packs or the file discovery.
pub fn run(
    args: &CheckArgs,
    global: &GlobalOptions,
    locations: &Locations,
    current_dir: &Path,
) -> Result<CheckReport, CodedError> {
    let root = fs::canonicalize(current_dir)
        .ok()
        .and_then(|path| FolderPath::new(path.to_str()?).ok())
        .ok_or_else(|| CodedError::new(ErrorCode::ProjectSourceInvalid))?;
    let config = config_files::load(locations, global, Some(Path::new(root.as_str())))?;
    let Analysis {
        discovery,
        analyzer,
        ..
    } = build_analysis(&config, locations)?;
    let checker = ConventionChecker::new(discovery, analyzer, config.conventions);
    let paths: Vec<String> = args
        .paths
        .iter()
        .map(|path| below_root(path, &root))
        .collect();
    checker.check(&CheckRequest {
        principal: &principal::resolve(),
        root: &root,
        paths: &paths,
    })
}

/// The path as the check wants it: an absolute path inside the root loses the root. One outside
/// stays absolute and therefore selects nothing; it must never widen the selection.
fn below_root(path: &Path, root: &FolderPath) -> String {
    let inside: PathBuf = path
        .strip_prefix(root.as_str())
        .map_or_else(|_| path.to_path_buf(), Path::to_path_buf);
    inside.to_string_lossy().into_owned()
}
