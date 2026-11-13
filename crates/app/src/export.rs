//! `codeskimr export`: writes the HTML export of a scanned project to a file.

use std::fs;
use std::path::Path;

use codeskimr_application::error::CodedError;
use codeskimr_application::registry::Registry;
use codeskimr_cli::{ExportArgs, GlobalOptions};
use codeskimr_codes::ErrorCode;

use crate::consts::BYTES_PER_KILOBYTE;
use crate::locations::Locations;
use crate::source_arg::classify;
use crate::ui_commands::{ExportInput, HtmlExport, export_command};
use crate::wiring::open_store;
use crate::{config_files, principal};
use codeskimr_storage::SqliteStore;

/// What was written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Written {
    /// Number of symbols in the page.
    pub symbols: usize,
    /// Size of the file, rounded up to whole kilobytes.
    pub kilobytes: usize,
}

/// Runs the registered command `export.html` as the local owner and writes the page to the file.
///
/// # Errors
/// - `PRJ-201` if the project is not registered, `MCP-203` if it has not been scanned.
/// - `EXP-201` if the built export page is missing, `EXP-601` if the file cannot be written.
/// - Any coded failure of the configuration or the storage.
pub fn run(
    args: &ExportArgs,
    global: &GlobalOptions,
    locations: &Locations,
) -> Result<Written, CodedError> {
    let config = config_files::load(locations, global, None)?;
    let store = open_store(locations, &config)?;
    let project = resolve(&args.project, &store)?;
    let mut registry = Registry::new();
    registry
        .register(export_command(&store, &config, locations))
        .map_err(CodedError::caused_by)?;
    let out = registry.execute::<HtmlExport>(&principal::resolve(), ExportInput { project })?;
    write_file(&args.html, &out.html)?;
    Ok(Written {
        symbols: out.symbols,
        kilobytes: out.html.len().div_ceil(BYTES_PER_KILOBYTE),
    })
}

/// A number names a project by its id; anything else is the source it was scanned from.
fn resolve(argument: &str, store: &SqliteStore) -> Result<u64, CodedError> {
    if let Ok(id) = argument.parse::<u64>() {
        return Ok(id);
    }
    let spec = classify(argument, None)?;
    store
        .project_by_source(&spec.source)?
        .map(|project| project.id.get())
        .ok_or_else(|| CodedError::new(ErrorCode::ProjectNotFound))
}

fn write_file(path: &Path, html: &str) -> Result<(), CodedError> {
    fs::write(path, html).map_err(|_| CodedError::new(ErrorCode::ExportWriteFailed))
}
