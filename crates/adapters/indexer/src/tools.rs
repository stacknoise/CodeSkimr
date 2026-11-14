//! Finding a tool and reading its version, without running project code.
//!
//! A tool is looked up in the `PATH` of the user only, never in the project, and a version probe
//! runs in a neutral folder: tools such as the `rustup` proxy or `dotnet` read files of the
//! working folder and may act on them.

use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

use codeskimr_application::cancel::CancelToken;
use codeskimr_application::ports::{ToolIssue, ToolProblem, ToolSpec, ToolVersion};

use crate::consts::{
    EXECUTABLE_SUFFIXES, PATH_ENV, PROBE_MAX_MEMORY, PROBE_MAX_OUTPUT, PROBE_TIMEOUT,
    WORD_SEPARATOR,
};
use crate::env::{Environment, safe_path};
use crate::process::{Launch, Limits, run_guarded};

/// Finds `program` in `path`, skipping entries that are relative or inside `project_root`.
#[must_use]
pub fn resolve(program: &str, path: &OsStr, project_root: &Path) -> Option<PathBuf> {
    let safe = safe_path(path, project_root);
    std::env::split_paths(&safe).find_map(|dir| {
        EXECUTABLE_SUFFIXES
            .iter()
            .map(|suffix| dir.join([program, suffix].concat()))
            .find(|candidate| candidate.is_file())
    })
}

/// Everything a check needs besides the tool.
#[derive(Clone, Copy)]
pub struct Probe<'a> {
    /// The environment the tool will run with.
    pub env: &'a Environment,
    /// A folder that belongs to no project, used as working folder.
    pub neutral_dir: &'a Path,
    /// The project folder, which `PATH` must not lead into.
    pub project_root: &'a Path,
}

/// Why `tool` is not usable, or `None` if it is installed and recent enough.
#[must_use]
pub fn check_tool(tool: &ToolSpec, probe: &Probe<'_>) -> Option<ToolProblem> {
    let problem = |issue| {
        Some(ToolProblem {
            program: tool.program.clone(),
            issue,
        })
    };
    let Some(path) = path_of(probe.env) else {
        return problem(ToolIssue::Missing);
    };
    let Some(found) = resolve(&tool.program, &path, probe.project_root) else {
        return problem(ToolIssue::Missing);
    };
    let output = version_output(&found, tool, probe);
    match ToolVersion::find_in(&output) {
        Some(version) if version >= tool.min_version => None,
        _ => problem(ToolIssue::TooOld {
            found: output.trim().to_owned(),
            required: tool.min_version,
        }),
    }
}

fn path_of(env: &Environment) -> Option<OsString> {
    env.iter()
        .find(|(name, _)| name.to_string_lossy().eq_ignore_ascii_case(PATH_ENV))
        .map(|(_, value)| value.clone())
}

/// What the tool prints when asked for its version, console streams joined (`java -version`
/// writes to standard error).
fn version_output(program: &Path, tool: &ToolSpec, probe: &Probe<'_>) -> String {
    let launch = Launch {
        program: program.to_path_buf(),
        args: tool.version_args.iter().map(OsString::from).collect(),
        env: probe.env.clone(),
        cwd: probe.neutral_dir.to_path_buf(),
        limits: Limits {
            timeout: PROBE_TIMEOUT,
            max_memory_bytes: PROBE_MAX_MEMORY,
            max_output_bytes: PROBE_MAX_OUTPUT,
        },
        watch_dir: None,
    };
    match run_guarded(&launch, &CancelToken::new()) {
        Ok(finished) if finished.success => {
            [finished.stdout_text(), finished.stderr_text()].join(WORD_SEPARATOR)
        }
        _ => String::new(),
    }
}
