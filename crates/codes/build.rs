//! Generates `ErrorCode` and `Group` from `errors/error-codes.toml` (blueprint 11.5).
//! The build fails with every problem listed when the registry or the messages are invalid.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// Name of the generated file inside `OUT_DIR`.
const OUTPUT_FILE: &str = "error_codes.rs";

/// Repository root relative to this crate.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn main() -> ExitCode {
    let root = repo_root();
    for watched in [
        codeskimr_codegen::consts::REGISTRY_PATH,
        codeskimr_codegen::consts::LOCALES_DIR,
    ] {
        println!("cargo::rerun-if-changed={}", root.join(watched).display());
    }
    let registry = match codeskimr_codegen::load(&root) {
        Ok(registry) => registry,
        Err(problems) => {
            for problem in &problems {
                eprintln!("error-codes: {problem}");
            }
            return ExitCode::FAILURE;
        }
    };
    let Some(out_dir) = std::env::var_os("OUT_DIR") else {
        eprintln!("error-codes: OUT_DIR is not set");
        return ExitCode::FAILURE;
    };
    let rendered = codeskimr_codegen::render_rust::render(&registry);
    match std::fs::write(Path::new(&out_dir).join(OUTPUT_FILE), rendered) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error-codes: {e}");
            ExitCode::FAILURE
        }
    }
}
