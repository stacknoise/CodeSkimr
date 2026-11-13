//! Helpers shared by the integration tests.

use codeskimr_application::ids::IdSequence;
use codeskimr_application::ports::{AnalyzeRequest, DiscoveredFile, FileAnalyzer, FileFacts};
use codeskimr_domain::{ContentHash, FolderPath, LanguageId, RelativePath, SnapshotId};
use codeskimr_parsing::TreeSitterAnalyzer;

/// Analyzes `file` (relative to `root`) as `language`; ids start at 1 for every call.
pub fn analyze_file(
    analyzer: &TreeSitterAnalyzer,
    ids: &IdSequence,
    root: &FolderPath,
    file: &str,
    language: &str,
) -> FileFacts {
    let discovered = DiscoveredFile {
        path: RelativePath::new(file).expect("path"),
        language: LanguageId::new(language).expect("language"),
        content_hash: ContentHash::new("hash").expect("hash"),
        changed: true,
    };
    let request = AnalyzeRequest {
        root,
        snapshot: SnapshotId::new(1),
        file: &discovered,
        ids,
    };
    analyzer.analyze(&request).expect("analyzes")
}
