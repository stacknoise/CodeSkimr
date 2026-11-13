//! Generates `ErrorCode`, `Group` and the i18n key constants from `errors/error-codes.toml` and
//! `locales/` (blueprint 11.5 and 13). The build fails with every problem listed when the
//! registry, the messages or the translations are invalid.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// Name of the generated error code file inside `OUT_DIR`.
const ERROR_CODES_FILE: &str = "error_codes.rs";
/// Name of the generated i18n key file inside `OUT_DIR`.
const I18N_KEYS_FILE: &str = "i18n_keys.rs";
/// Label in front of every reported problem.
const PROBLEM_LABEL: &str = "codegen";

/// Repository root relative to this crate.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Prints every problem and returns the failing exit code.
fn fail(problems: &[String]) -> ExitCode {
    for problem in problems {
        eprintln!("{PROBLEM_LABEL}: {problem}");
    }
    ExitCode::FAILURE
}

/// Loads and validates the inputs and writes both generated files.
fn generate(root: &Path, out_dir: &Path) -> Result<(), Vec<String>> {
    let registry = codeskimr_codegen::load(root)?;
    let catalog = codeskimr_codegen::i18n::load(root)?;
    let outputs = [
        (
            ERROR_CODES_FILE,
            codeskimr_codegen::render_rust::render(&registry),
        ),
        (
            I18N_KEYS_FILE,
            codeskimr_codegen::i18n::render_rust(&catalog),
        ),
    ];
    for (name, content) in outputs {
        std::fs::write(out_dir.join(name), content).map_err(|e| vec![e.to_string()])?;
    }
    Ok(())
}

fn main() -> ExitCode {
    let root = repo_root();
    for watched in [
        codeskimr_codegen::consts::REGISTRY_PATH,
        codeskimr_codegen::consts::LOCALES_DIR,
    ] {
        println!("cargo::rerun-if-changed={}", root.join(watched).display());
    }
    let Some(out_dir) = std::env::var_os("OUT_DIR") else {
        return fail(&["OUT_DIR is not set".to_string()]);
    };
    match generate(&root, Path::new(&out_dir)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(problems) => fail(&problems),
    }
}
