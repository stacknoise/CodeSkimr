//! Embeds the built UI (`web/dist`) into the program (ADR 0030). Without a build the list of
//! files is empty and the server answers the API only, or serves a UI folder.

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// The built UI, relative to this crate.
const UI_DIST: &str = "../../web/dist";
/// The generated file inside `OUT_DIR`.
const OUTPUT_FILE: &str = "embedded_ui.rs";

/// Every file below `dir`, with the path relative to `base`, in a fixed order.
fn files_below(base: &Path, dir: &Path, found: &mut Vec<PathBuf>) -> std::io::Result<()> {
    let mut entries: Vec<PathBuf> = fs::read_dir(dir)?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            files_below(base, &path, found)?;
        } else if let Ok(relative) = path.strip_prefix(base) {
            found.push(relative.to_path_buf());
        }
    }
    Ok(())
}

/// The Rust source of the table of files.
fn render(base: &Path, files: &[PathBuf]) -> String {
    let mut source = String::from("pub static FILES: &[(&str, &[u8])] = &[\n");
    for relative in files {
        let name: Vec<String> = relative
            .components()
            .map(|part| part.as_os_str().to_string_lossy().into_owned())
            .collect();
        let absolute = base.join(relative);
        let _ = writeln!(
            source,
            "    ({:?}, include_bytes!({:?})),",
            name.join("/"),
            absolute.display().to_string()
        );
    }
    source.push_str("];\n");
    source
}

fn main() -> ExitCode {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let base = root.join(UI_DIST);
    println!("cargo:rerun-if-changed={}", base.display());
    let mut files = Vec::new();
    if base.join("index.html").is_file() && files_below(&base, &base, &mut files).is_err() {
        eprintln!("embedded UI: cannot read {}", base.display());
        return ExitCode::FAILURE;
    }
    let Some(out_dir) = std::env::var_os("OUT_DIR") else {
        return ExitCode::FAILURE;
    };
    match fs::write(Path::new(&out_dir).join(OUTPUT_FILE), render(&base, &files)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
}
