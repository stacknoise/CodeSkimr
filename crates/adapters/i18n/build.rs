//! Embeds every Fluent file of `locales/` into the crate (blueprint section 13), so adding a
//! language needs no code change. The build fails when the translations are incomplete.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// Name of the generated file inside `OUT_DIR`.
const OUTPUT_FILE: &str = "embedded.rs";

/// Repository root relative to this crate.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

/// Prints every problem and returns the failing exit code.
fn fail(problems: &[String]) -> ExitCode {
    for problem in problems {
        eprintln!("i18n: {problem}");
    }
    ExitCode::FAILURE
}

fn main() -> ExitCode {
    let root = repo_root();
    let watched = root.join(codeskimr_codegen::consts::LOCALES_DIR);
    println!("cargo::rerun-if-changed={}", watched.display());
    let Some(out_dir) = std::env::var_os("OUT_DIR") else {
        return fail(&["OUT_DIR is not set".to_string()]);
    };
    let catalog = match codeskimr_codegen::i18n::load(&root) {
        Ok(catalog) => catalog,
        Err(problems) => return fail(&problems),
    };
    let rendered = codeskimr_codegen::i18n::render_embedded(&catalog, &root);
    match std::fs::write(Path::new(&out_dir).join(OUTPUT_FILE), rendered) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => fail(&[e.to_string()]),
    }
}
