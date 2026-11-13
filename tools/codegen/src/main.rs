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

fn main() -> ExitCode {
    let root = repo_root();
    let registry = match codeskimr_codegen::load(&root) {
        Ok(registry) => registry,
        Err(problems) => {
            for problem in &problems {
                eprintln!("error: {problem}");
            }
            return ExitCode::FAILURE;
        }
    };
    let outputs = codeskimr_codegen::checked_in_outputs(&registry);
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
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
