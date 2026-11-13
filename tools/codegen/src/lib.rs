//! Generator for the error code registry and the i18n keys (blueprint sections 11 and 13).
//!
//! Reads `errors/error-codes.toml` and `locales/`, validates both, and renders Rust,
//! TypeScript and the Markdown catalogue. `build.rs` of `codeskimr_codes` uses the Rust
//! output; the binary writes (or with `--check` verifies) the checked-in files.

pub mod consts;
pub mod ftl;
pub mod i18n;
pub mod registry;
pub mod render_md;
pub mod render_rust;
pub mod render_ts;
pub mod validate;

use std::path::Path;

pub use registry::Registry;

/// Loads and fully validates the registry under the repository `root`.
///
/// # Errors
/// Returns every problem found as one message each.
pub fn load(root: &Path) -> Result<Registry, Vec<String>> {
    let path = root.join(consts::REGISTRY_PATH);
    let text =
        std::fs::read_to_string(&path).map_err(|e| vec![format!("{}: {e}", path.display())])?;
    let registry =
        Registry::parse(&text).map_err(|e| vec![format!("{}: {e}", consts::REGISTRY_PATH)])?;
    validate::validate_registry(&registry)?;
    validate::validate_locales(&registry, root)?;
    Ok(registry)
}

/// A generated file: path relative to the repository root and its content.
pub type Output = (&'static str, String);

/// The files that are checked in (everything except the Rust output).
#[must_use]
pub fn checked_in_outputs(registry: &Registry, catalog: &i18n::Catalog) -> [Output; 3] {
    [
        (consts::TS_PATH, render_ts::render(registry)),
        (consts::DOCS_PATH, render_md::render(registry)),
        (consts::I18N_TS_PATH, i18n::render_ts(catalog)),
    ]
}
