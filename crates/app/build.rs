//! Embeds the built UI (`web/dist`) into the program (ADR 0030). Without a build the list of
//! files is empty and the server answers the API only, or serves a UI folder.

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// The built UI, relative to this crate.
const UI_DIST: &str = "../../web/dist";
/// Names with this start are never served (strict path rules of the server).
const HIDDEN_PREFIX: &str = ".";
/// Source maps are not served and only make the program larger.
const SOURCE_MAP_SUFFIX: &str = ".map";
/// The start page; without it the UI is not built.
const INDEX_FILE: &str = "index.html";
/// The generated file inside `OUT_DIR`.
const OUTPUT_FILE: &str = "embedded_ui.rs";

/// Every file below `dir`, with the path relative to `base`, in a fixed order.
fn files_below(base: &Path, dir: &Path, found: &mut Vec<PathBuf>) -> std::io::Result<()> {
    let mut entries = Vec::new();
    for entry in fs::read_dir(dir)? {
        entries.push(entry?.path());
    }
    entries.sort();
    for path in entries {
        if !is_served(&path) {
            continue;
        }
        if path.is_dir() {
            files_below(base, &path, found)?;
        } else if let Ok(relative) = path.strip_prefix(base) {
            found.push(relative.to_path_buf());
        }
    }
    Ok(())
}

/// Whether the server could ever serve `path`: no name starting with a dot (hidden files, such as
/// `.vite`) and no source maps.
fn is_served(path: &Path) -> bool {
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    !name.starts_with(HIDDEN_PREFIX) && !name.ends_with(SOURCE_MAP_SUFFIX)
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
    // A path that does not exist makes Cargo build the crate again every time; the empty folder
    // makes the UI build (which fills it) the only reason to build again.
    if fs::create_dir_all(&base).is_err() {
        eprintln!("embedded UI: cannot create {}", base.display());
        return ExitCode::FAILURE;
    }
    println!("cargo:rerun-if-changed={}", base.display());
    let mut files = Vec::new();
    if base.join(INDEX_FILE).is_file() {
        if files_below(&base, &base, &mut files).is_err() {
            eprintln!("embedded UI: cannot read {}", base.display());
            return ExitCode::FAILURE;
        }
    } else {
        println!(
            "cargo:warning=the UI is not built (web/dist); run `npm run build` in web/ first, the program will have no UI"
        );
    }
    let Some(out_dir) = std::env::var_os("OUT_DIR") else {
        return ExitCode::FAILURE;
    };
    match fs::write(Path::new(&out_dir).join(OUTPUT_FILE), render(&base, &files)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
}
