//! Embeds the built UI (`web/dist`, ADR 0030) and the language packs (`languages/`) into the program.
//! Without a UI build the list of UI files is empty and the server answers the API only, or serves a
//! UI folder. The language packs are always there; they are written to the data folder when no pack
//! folder exists (`embedded_packs.rs`).

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
/// The language packs, relative to this crate.
const PACKS_DIR: &str = "../../languages";
/// The generated table of the language packs inside `OUT_DIR`.
const PACKS_OUTPUT_FILE: &str = "embedded_packs.rs";
/// Test files of a pack; they are not needed to load it.
const PACK_TESTS_DIR: &str = "tests";
/// FNV-1a, 64 bit: start value and prime. A fingerprint of the embedded packs, not a security check.
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// Every file below `dir`, with the path relative to `base`, in a fixed order.
fn files_below(
    base: &Path,
    dir: &Path,
    keep: fn(&Path) -> bool,
    found: &mut Vec<PathBuf>,
) -> std::io::Result<()> {
    let mut entries = Vec::new();
    for entry in fs::read_dir(dir)? {
        entries.push(entry?.path());
    }
    entries.sort();
    for path in entries {
        if !keep(&path) {
            continue;
        }
        if path.is_dir() {
            files_below(base, &path, keep, found)?;
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

/// Whether `path` is part of a language pack that is needed to load it: no hidden files and no
/// folder of test files.
fn is_pack_file(path: &Path) -> bool {
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    !(name.starts_with(HIDDEN_PREFIX) || path.is_dir() && name == PACK_TESTS_DIR)
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

/// The fingerprint of the packs: names and contents, in the order of the table.
fn fingerprint(base: &Path, files: &[PathBuf]) -> std::io::Result<u64> {
    let mut hash = FNV_OFFSET;
    for relative in files {
        let content = fs::read(base.join(relative))?;
        let name = relative.to_string_lossy().into_owned();
        for byte in name.bytes().chain(content) {
            hash = (hash ^ u64::from(byte)).wrapping_mul(FNV_PRIME);
        }
    }
    Ok(hash)
}

/// Writes the table of the language packs and its fingerprint.
fn embed_packs(root: &Path, out_dir: &Path) -> std::io::Result<()> {
    let base = root.join(PACKS_DIR);
    println!("cargo:rerun-if-changed={}", base.display());
    let mut files = Vec::new();
    files_below(&base, &base, is_pack_file, &mut files)?;
    let mut source = render(&base, &files);
    let _ = writeln!(
        source,
        "pub const FINGERPRINT: &str = {:?};",
        format!("{:016x}", fingerprint(&base, &files)?)
    );
    fs::write(out_dir.join(PACKS_OUTPUT_FILE), source)
}

/// The app icon, relative to this crate (made by `tools/logo/make_icons.py`).
#[cfg(windows)]
const APP_ICON: &str = "assets/codeskimr.ico";

/// Embeds the app icon into the Windows exe, so Explorer and the task bar show the logo.
#[cfg(windows)]
fn embed_app_icon() -> std::io::Result<()> {
    println!("cargo:rerun-if-changed={APP_ICON}");
    let mut resource = winresource::WindowsResource::new();
    resource.set_icon(APP_ICON);
    resource.compile()
}

fn main() -> ExitCode {
    #[cfg(windows)]
    if let Err(error) = embed_app_icon() {
        eprintln!("app icon: {error}");
        return ExitCode::FAILURE;
    }
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
        if files_below(&base, &base, is_served, &mut files).is_err() {
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
    let out_dir = Path::new(&out_dir);
    let written = fs::write(out_dir.join(OUTPUT_FILE), render(&base, &files));
    if written.is_err() {
        eprintln!("embedded UI: cannot write the table");
        return ExitCode::FAILURE;
    }
    if embed_packs(root, out_dir).is_err() {
        eprintln!("embedded packs: cannot read {PACKS_DIR}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
