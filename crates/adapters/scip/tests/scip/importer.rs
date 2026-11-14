//! The importer of the application port: reads the file and uses the lines of the source.

use std::sync::{Arc, Mutex};

use codeskimr_application::error::CodedError;
use codeskimr_application::ports::{Excerpt, ImportRequest, ReferenceImporter, SourceText};
use codeskimr_codes::{Coded, ErrorCode};
use codeskimr_domain::{FolderPath, RelativePath};
use codeskimr_scip::ScipImporter;

use crate::common::{LIMIT, definition, document, file, index, run_index, scip, symbol, usage};
use crate::mapping::service_world;

const FILE_LIMIT: u64 = 4096;

/// Gives no source: columns of UTF-8 indexes need none.
struct NoText;

impl SourceText for NoText {
    fn excerpt(
        &self,
        _: &FolderPath,
        _: &RelativePath,
        _: (u32, u32),
        _: (u32, u64),
    ) -> Result<Excerpt, CodedError> {
        Err(CodedError::new(ErrorCode::SourcePathNotFound))
    }
}

fn import(
    bytes: &[u8],
    min_mapped: f64,
) -> Result<codeskimr_application::ports::Imported, CodedError> {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("index.scip");
    std::fs::write(&path, bytes).expect("writes");
    let (files, symbols) = service_world();
    let root = FolderPath::new("/work").expect("root");
    ScipImporter::new(Arc::new(NoText)).import(&ImportRequest {
        index: &path,
        root: &root,
        files: &files,
        symbols: &symbols,
        references: &[],
        max_index_bytes: LIMIT,
        max_file_bytes: FILE_LIMIT,
        min_mapped,
    })
}

#[test]
fn the_importer_returns_the_new_references_of_the_covered_files() {
    let imported = import(&run_index(&[[3, 12, 15]]), 0.5).expect("imports");
    assert_eq!(imported.mapped, 1);
    assert_eq!(imported.references.len(), 1);
    assert!(
        imported
            .references
            .iter()
            .all(|reference| reference.verified)
    );
    assert_ne!(imported.files.len(), 0);
}

#[test]
fn a_missing_index_is_a_coded_failure_and_the_threshold_is_kept() {
    let missing = ScipImporter::new(Arc::new(NoText)).import(&ImportRequest {
        index: std::path::Path::new("does-not-exist.scip"),
        root: &FolderPath::new("/work").expect("root"),
        files: &[file(1, "src/a.cs")],
        symbols: &[symbol(
            1,
            1,
            codeskimr_domain::SymbolKind::Class,
            "Svc",
            ((0, 0), (3, 1)),
        )],
        references: &[],
        max_index_bytes: LIMIT,
        max_file_bytes: FILE_LIMIT,
        min_mapped: 0.5,
    });
    assert_eq!(
        missing.expect_err("fails").code(),
        ErrorCode::ScipIndexInvalid
    );
    let too_strict = import(&run_index(&[[3, 12, 15]]), 1.5).expect_err("fails");
    assert_eq!(too_strict.code(), ErrorCode::ScipMappingTooLow);
}

/// Remembers what it was asked for and answers with a line of non-ASCII text.
struct Recording(Mutex<Vec<(String, u64)>>);

impl SourceText for Recording {
    fn excerpt(
        &self,
        _: &FolderPath,
        path: &RelativePath,
        _: (u32, u32),
        limits: (u32, u64),
    ) -> Result<Excerpt, CodedError> {
        self.0
            .lock()
            .expect("lock")
            .push((path.as_str().to_owned(), limits.1));
        Ok(Excerpt {
            lines: vec!["ä".to_owned()],
            truncated: false,
        })
    }
}

#[test]
fn only_files_of_the_snapshot_are_read_and_only_up_to_the_limit() {
    let symbol_text = scip("ns/Svc#Run().");
    let bytes = index(
        "file:///p",
        &[
            document("src/a.cs", &[definition(&[2, 16, 19], &symbol_text)]),
            document("src/b.cs", &[usage(&[3, 12, 15], &symbol_text)]),
            document("logs/huge.log", &[usage(&[0, 0, 1], &symbol_text)]),
        ],
    );
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("index.scip");
    std::fs::write(&path, bytes).expect("writes");
    let (files, symbols) = service_world();
    let recording = Arc::new(Recording(Mutex::default()));
    let root = FolderPath::new("/work").expect("root");
    let _ = ScipImporter::new(recording.clone()).import(&ImportRequest {
        index: &path,
        root: &root,
        files: &files,
        symbols: &symbols,
        references: &[],
        max_index_bytes: LIMIT,
        max_file_bytes: FILE_LIMIT,
        min_mapped: 0.0,
    });
    let asked = recording.0.lock().expect("lock").clone();
    assert_ne!(asked.len(), 0);
    assert!(
        asked.iter().all(|(path, limit)| {
            (path == "src/a.cs" || path == "src/b.cs") && *limit == FILE_LIMIT
        }),
        "{asked:?}"
    );
}
