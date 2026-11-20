//! Finding a tool and reading its version, without running project code.
//!
//! A tool is looked up in the `PATH` of the user only, never in the project, and a version probe
//! runs in a neutral folder: tools such as the `rustup` proxy or `dotnet` read files of the
//! working folder and may act on them.

use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

use codeskimr_application::cancel::CancelToken;
use codeskimr_application::ports::{ToolIssue, ToolProblem, ToolSeen, ToolSpec, ToolVersion};

use crate::consts::{
    ERROR_WORD, EXECUTABLE_SUFFIXES, PATH_ENV, PROBE_MAX_MEMORY, PROBE_MAX_OUTPUT, PROBE_TIMEOUT,
    VERSION_EXCERPT_CHARS, WORD_SEPARATOR,
};
use crate::env::{Environment, safe_dirs};
use crate::process::{Launch, Limits, run_guarded};
use crate::redact::{Redact, excerpt};

/// Finds `program` in `path`, skipping entries that are relative or inside `project_root`.
#[must_use]
pub fn resolve(program: &str, path: &OsStr, project_root: &Path) -> Option<PathBuf> {
    safe_dirs(path, project_root).into_iter().find_map(|dir| {
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
    /// Masks secrets in what a tool printed before it is kept.
    pub redact: Redact,
    /// A folder that belongs to no project, used as working folder.
    pub neutral_dir: &'a Path,
    /// The project folder, which `PATH` must not lead into.
    pub project_root: &'a Path,
}

/// Why `tool` is not usable, or the version it reported if it is installed and recent enough.
///
/// # Errors
/// The problem: the tool is missing, or its version is too old or cannot be read.
pub fn check_tool(tool: &ToolSpec, probe: &Probe<'_>) -> Result<ToolSeen, ToolProblem> {
    let problem = |issue| {
        Err(ToolProblem {
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
    // A program that cannot even print its version is not installed in a usable way, such as the
    // `rustup` shim of a component that was never added (#402) or an indexer that crashes on
    // start. It is reported as found but not runnable, with the line that tells why.
    let output = match version_output(&found, tool, probe) {
        Ok(output) => output,
        Err(failure) => {
            return problem(ToolIssue::NotRunnable {
                output: excerpt(&error_line(&failure), VERSION_EXCERPT_CHARS, probe.redact),
            });
        }
    };
    match ToolVersion::find_in(&output) {
        Some(version) if version >= tool.min_version => Ok(ToolSeen {
            program: tool.program.clone(),
            version,
        }),
        _ => problem(ToolIssue::TooOld {
            found: excerpt(output.trim(), VERSION_EXCERPT_CHARS, probe.redact),
            required: tool.min_version,
        }),
    }
}

fn path_of(env: &Environment) -> Option<OsString> {
    env.iter()
        .find(|(name, _)| name.to_string_lossy().eq_ignore_ascii_case(PATH_ENV))
        .map(|(_, value)| value.clone())
}

/// The line of a failure that says what went wrong: the first one that names an error (a crash
/// prints its path and source line first), else the first line that says anything.
fn error_line(failure: &str) -> String {
    let lines: Vec<&str> = failure
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();
    lines
        .iter()
        .copied()
        .find(|line| line.to_lowercase().contains(ERROR_WORD))
        .or_else(|| lines.first().copied())
        .unwrap_or_default()
        .to_owned()
}

/// What the tool prints when asked for its version, console streams joined (`java -version`
/// writes to standard error). The error is what it printed if the command ran but did not
/// succeed; it is empty if the command could not run at all.
fn version_output(program: &Path, tool: &ToolSpec, probe: &Probe<'_>) -> Result<String, String> {
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
        note_dir: None,
    };
    match run_guarded(&launch, &CancelToken::new()) {
        Ok(finished) => {
            let text = [finished.stdout_text(), finished.stderr_text()].join(WORD_SEPARATOR);
            if finished.success {
                Ok(text)
            } else {
                Err(text)
            }
        }
        Err(_) => Err(String::new()),
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;
    use std::path::Path;

    use super::{error_line, resolve};

    /// A file of the folder the tests run in must not be found through an empty or relative path.
    #[test]
    fn a_program_in_the_working_folder_is_not_found_through_an_empty_or_relative_path() {
        let root = std::env::temp_dir();
        let program = if cfg!(windows) { "Cargo" } else { "Cargo.toml" };
        for path in ["", ".", "./", ".."] {
            assert_eq!(resolve(program, OsStr::new(path), &root), None, "{path:?}");
        }
        assert!(Path::new("Cargo.toml").is_file());
    }

    /// A crash prints the path and the source line before the error: the error line is the one
    /// that is kept.
    #[test]
    fn the_error_line_of_a_crash_is_the_one_that_names_the_error() {
        let crash = "C:\\x\\Env.ts:4\nconst a = 1;\n   ^\n\nSyntaxError: Invalid regular expression\n    at new RegExp";
        assert_eq!(error_line(crash), "SyntaxError: Invalid regular expression");
        assert_eq!(error_line("\n first words \nsecond"), "first words");
        assert_eq!(error_line(""), "");
    }
}
