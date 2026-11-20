//! The `ReferenceImporter` port: reads an index, converts its columns and maps it onto the
//! snapshot (ADR 0033).

use std::collections::HashSet;
use std::sync::Arc;

use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{ImportRequest, Imported, ReferenceImporter, SourceText};
use codeskimr_domain::{FolderPath, Reference, RelativePath};

use crate::columns::convert_columns;
use crate::consts::{FIRST_LINE, MAX_REFERENCES_PER_IMPORT};
use crate::create::Creation;
use crate::decode::read_index;
use crate::mapping::{SnapshotView, import_with_sites};

/// Imports SCIP indexes. `text` gives the lines of the project files, which the conversion of
/// UTF-16 and UTF-32 columns needs.
pub struct ScipImporter {
    text: Arc<dyn SourceText>,
}

impl ScipImporter {
    /// An importer that reads source lines through `text`.
    #[must_use]
    pub fn new(text: Arc<dyn SourceText>) -> Self {
        Self { text }
    }

    /// The lines of a file; `None` if it cannot be read, which leaves its occurrences unchanged.
    fn lines(&self, root: &FolderPath, path: &RelativePath, max_bytes: u64) -> Option<Vec<String>> {
        self.text
            .excerpt(root, path, (FIRST_LINE, u32::MAX), (u32::MAX, max_bytes))
            .ok()
            .map(|excerpt| excerpt.lines)
    }
}

impl std::fmt::Debug for ScipImporter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(std::any::type_name::<Self>()).finish()
    }
}

/// What the import may create and how many references it may produce, from the request.
fn creation_of<'a>(request: &ImportRequest<'a>) -> Creation<'a> {
    Creation {
        kinds: request.symbol_kinds,
        separator: request.separator,
        max_symbols: request.max_created_symbols,
        library_root: request.library_root,
        max_references: MAX_REFERENCES_PER_IMPORT,
    }
}

impl ReferenceImporter for ScipImporter {
    fn import(&self, request: &ImportRequest<'_>) -> Result<Imported, CodedError> {
        let mut index =
            read_index(request.index, request.max_index_bytes).map_err(CodedError::caused_by)?;
        let known: HashSet<&RelativePath> = request.files.iter().map(|file| &file.path).collect();
        // Only files of the snapshot are read, and only up to the limit: the paths come from a
        // process that ran project code.
        convert_columns(&mut index, &mut |path| {
            known
                .contains(path)
                .then(|| self.lines(request.root, path, request.max_file_bytes))
                .flatten()
        });
        let view = SnapshotView {
            files: request.files,
            symbols: request.symbols,
            references: request.references,
        };
        let creation = creation_of(request);
        let outcome = import_with_sites(
            &index,
            &view,
            (request.min_mapped, &creation),
            request.import_sites,
        )
        .map_err(CodedError::caused_by)?;
        let references: Vec<Reference> = outcome
            .references
            .into_iter()
            .filter(|reference| outcome.covered.binary_search(&reference.file).is_ok())
            .collect();
        Ok(Imported {
            files: outcome.covered,
            symbols: outcome.symbols,
            references,
            mapped: outcome.stats.mapped,
            dropped: outcome.stats.dropped(),
            outside_scan: outcome.stats.outside_scan,
        })
    }
}
