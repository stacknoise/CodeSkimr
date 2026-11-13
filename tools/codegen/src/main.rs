//! Writes (or with `--check` verifies) the checked-in generated files.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use codeskimr_codegen::consts::CHECK_FLAG;

/// Repository root: two levels above this crate's manifest folder.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Writes every output file, creating folders as needed.
fn write_all(root: &Path, outputs: &[codeskimr_codegen::Output]) -> Result<(), String> {
    for (rel, content) in outputs {
        let path = root.join(rel);
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        }
        std::fs::write(&path, content).map_err(|e| format!("{}: {e}", path.display()))?;
    }
    Ok(())
}

/// Lists the files whose on-disk content differs from the generated content.
fn stale(root: &Path, outputs: &[codeskimr_codegen::Output]) -> Vec<String> {
    outputs
        .iter()
        .filter(|(rel, content)| {
            std::fs::read_to_string(root.join(rel)).ok().as_deref() != Some(content.as_str())
        })
        .map(|(rel, _)| (*rel).to_string())
        .collect()
}

/// Prints every problem and returns the failing exit code.
fn fail(problems: &[String]) -> ExitCode {
    for problem in problems {
        eprintln!("error: {problem}");
    }
    ExitCode::FAILURE
}

/// Loads and validates registry and translations, then renders the checked-in files.
fn render(root: &Path) -> Result<Vec<codeskimr_codegen::Output>, Vec<String>> {
    let registry = codeskimr_codegen::load(root)?;
    let catalog = codeskimr_codegen::i18n::load(root)?;
    Ok(codeskimr_codegen::checked_in_outputs(&registry, &catalog).into())
}

fn main() -> ExitCode {
    let root = repo_root();
    let outputs = match render(&root) {
        Ok(outputs) => outputs,
        Err(problems) => return fail(&problems),
    };
    if std::env::args().any(|a| a == CHECK_FLAG) {
        let files = stale(&root, &outputs);
        for file in &files {
            eprintln!("error: {file} is out of date, run `cargo run -p codeskimr_codegen`");
        }
        return if files.is_empty() {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        };
    }
    match write_all(&root, &outputs) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => fail(&[e]),
    }
}
