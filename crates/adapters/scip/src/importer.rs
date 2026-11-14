//! The `ReferenceImporter` port: reads an index, converts its columns and maps it onto the
//! snapshot (ADR 0033).

use std::sync::Arc;

use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{ImportRequest, Imported, ReferenceImporter, SourceText};
use codeskimr_domain::{FolderPath, Reference};

use crate::columns::convert_columns;
use crate::consts::FIRST_LINE;
use crate::decode::read_index;
use crate::mapping::{SnapshotView, import_references};

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
    fn lines(
        &self,
        root: &FolderPath,
        path: &codeskimr_domain::RelativePath,
    ) -> Option<Vec<String>> {
        self.text
            .excerpt(root, path, (FIRST_LINE, u32::MAX), (u32::MAX, u64::MAX))
            .ok()
            .map(|excerpt| excerpt.lines)
    }
}

impl std::fmt::Debug for ScipImporter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(std::any::type_name::<Self>()).finish()
    }
}

impl ReferenceImporter for ScipImporter {
    fn import(&self, request: &ImportRequest<'_>) -> Result<Imported, CodedError> {
        let mut index =
            read_index(request.index, request.max_index_bytes).map_err(CodedError::caused_by)?;
        convert_columns(&mut index, &mut |path| self.lines(request.root, path));
        let view = SnapshotView {
            files: request.files,
            symbols: request.symbols,
            references: request.references,
        };
        let outcome =
            import_references(&index, &view, request.min_mapped).map_err(CodedError::caused_by)?;
        let references: Vec<Reference> = outcome
            .references
            .into_iter()
            .filter(|reference| outcome.covered.binary_search(&reference.file).is_ok())
            .collect();
        Ok(Imported {
            files: outcome.covered,
            references,
            mapped: outcome.stats.mapped,
            dropped: outcome.stats.dropped(),
        })
    }
}
