//! The conversion of columns reads project files through the real `FolderText`: a document
//! whose path is a link that leaves the project must not be read (ADR 0033, blueprint S-05).

#[cfg(all(test, unix))]
mod cases {
    use std::sync::Arc;

    use codeskimr_application::ports::{ImportRequest, ReferenceImporter};
    use codeskimr_domain::{
        ContentHash, FileId, FolderPath, LanguageId, Loc, RelativePath, SnapshotId, SourceFile,
    };
    use codeskimr_scip::ScipImporter;
    use codeskimr_source::FolderText;

    const OCCURRENCE_RANGE: [u8; 3] = [0, 4, 7];
    const SYMBOL: &str = "scip-dotnet nuget . . ns/Svc#Run().";

    fn field(number: u8, payload: &[u8]) -> Vec<u8> {
        let length = u8::try_from(payload.len()).unwrap_or(u8::MAX);
        let mut bytes = vec![(number << 3) | 2, length];
        bytes.extend_from_slice(payload);
        bytes
    }

    /// An index with one document `a.cs` that holds one usage of `SYMBOL`.
    fn index_of_a() -> Vec<u8> {
        let mut occurrence = field(1, &OCCURRENCE_RANGE);
        occurrence.extend(field(2, SYMBOL.as_bytes()));
        let mut document = field(1, b"a.cs");
        document.extend(field(2, &occurrence));
        let mut index = field(1, &field(3, b"file:///p"));
        index.extend(field(2, &document));
        index
    }

    #[test]
    fn a_document_that_is_a_link_out_of_the_project_is_not_read() {
        let outside = tempfile::tempdir().unwrap();
        let project = tempfile::tempdir().unwrap();
        std::fs::write(outside.path().join("secret.txt"), "ä secret text\n").unwrap();
        std::os::unix::fs::symlink(
            outside.path().join("secret.txt"),
            project.path().join("a.cs"),
        )
        .unwrap();
        let index = project.path().join("index.scip");
        std::fs::write(&index, index_of_a()).unwrap();
        let files = [SourceFile {
            id: FileId::new(1),
            snapshot: SnapshotId::new(1),
            path: RelativePath::new("a.cs").unwrap(),
            language: LanguageId::new("csharp").unwrap(),
            content_hash: ContentHash::new("h").unwrap(),
            loc: Loc(1),
        }];
        let root = FolderPath::new(project.path().to_str().unwrap()).unwrap();
        let scip = ScipImporter::new(Arc::new(FolderText));
        // The text cannot be read, so the document keeps its encoding; the import then fails below the
        // threshold or maps nothing, but it never reads the file behind the link.
        let outcome = scip.import(&ImportRequest {
            index: &index,
            root: &root,
            files: &files,
            symbols: &[],
            references: &[],
            max_index_bytes: u64::from(u32::MAX),
            max_file_bytes: u64::from(u32::MAX),
            min_mapped: 0.0,
        });
        let found = outcome.ok();
        assert!(found.is_none_or(|imported| imported.references.is_empty()));
        assert!(text_behind_link_is_refused(&root, &files[0].path));
    }

    /// Asks `FolderText` directly: the link is refused as not found.
    fn text_behind_link_is_refused(root: &FolderPath, path: &RelativePath) -> bool {
        use codeskimr_application::ports::SourceText;
        FolderText.excerpt(root, path, (1, 10), (10, 1000)).is_err()
    }
}
