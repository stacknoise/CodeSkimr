//! Tests of the syntax fingerprints that the analyzer attaches to callables.

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::time::Duration;

    use codeskimr_application::config::Config;
    use codeskimr_application::ids::IdSequence;
    use codeskimr_application::ports::{
        AnalyzeRequest, DiscoveredFile, FileAnalyzer, FileFacts, SnippetFingerprinter,
    };
    use codeskimr_codes::Coded;
    use codeskimr_domain::{ContentHash, FolderPath, LanguageId, RelativePath, SnapshotId};
    use codeskimr_parsing::{LanguagePacks, TreeSitterAnalyzer};

    fn fixtures() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
    }

    fn analyzer(fingerprints: bool) -> TreeSitterAnalyzer {
        let packs = LanguagePacks::load(&fixtures().join("languages")).expect("packs");
        let analyzer = TreeSitterAnalyzer::new(packs, Duration::from_secs(5));
        if fingerprints {
            let config = Config::defaults().expect("defaults");
            analyzer
                .with_fingerprints(&config.similarity)
                .expect("params")
        } else {
            analyzer
        }
    }

    fn twins() -> DiscoveredFile {
        let language = LanguageId::new("rust").expect("language");
        let path = RelativePath::new("src/twins.rs").expect("path");
        DiscoveredFile {
            content_hash: ContentHash::new("twins").expect("hash"),
            changed: true,
            language,
            path,
        }
    }

    fn analyze(fingerprints: bool) -> FileFacts {
        let folder = fixtures().join("project");
        let ids = IdSequence::starting_at(1);
        let request = AnalyzeRequest {
            snapshot: SnapshotId::new(1),
            root: &FolderPath::new(folder.to_string_lossy()).expect("root"),
            file: &twins(),
            ids: &ids,
        };
        analyzer(fingerprints).analyze(&request).expect("analysis")
    }

    fn prints_of<'a>(
        facts: &'a FileFacts,
        name: &str,
    ) -> Option<&'a codeskimr_domain::Fingerprints> {
        let symbol = facts.symbols.iter().find(|s| s.name.as_str() == name)?;
        facts
            .fingerprints
            .iter()
            .find(|entry| entry.symbol == symbol.id)
            .map(|entry| &entry.fingerprints)
    }

    #[test]
    fn nothing_is_fingerprinted_unless_enabled() {
        assert!(matches!(analyze(false).fingerprints.as_slice(), []));
    }

    #[test]
    fn renamed_copies_are_identical_and_other_code_is_not() {
        let facts = analyze(true);
        let price = prints_of(&facts, "total_price").expect("total_price");
        let weights = prints_of(&facts, "sum_weights").expect("sum_weights");
        let describe = prints_of(&facts, "describe").expect("describe");
        assert!((price.similarity(weights) - 1.0).abs() < f64::EPSILON);
        assert!(price.similarity(describe) < 0.3);
    }

    #[test]
    fn short_bodies_and_non_callables_get_no_fingerprints() {
        let facts = analyze(true);
        assert!(prints_of(&facts, "tiny").is_none());
        assert!(prints_of(&facts, "twins").is_none());
        assert_eq!(facts.fingerprints.len(), 3);
    }

    fn snippet_of(
        text: &str,
    ) -> Result<codeskimr_domain::Fingerprints, codeskimr_application::error::CodedError> {
        analyzer(true).fingerprint(&LanguageId::new("rust").expect("language"), text)
    }

    #[test]
    fn a_snippet_of_a_function_matches_the_function_in_its_file() {
        let facts = analyze(true);
        let price = prints_of(&facts, "total_price").expect("total_price");
        let copy = snippet_of(
            "fn amount(rows: &[u32], pct: u32) -> u32 {\n    let mut acc = 1;\n    for row in rows {\n        \
             if *row > 4 {\n            acc += row * pct;\n        } else {\n            acc += row + 2;\n        }\n    }\n    acc\n}\n",
        )
        .expect("fingerprints");
        assert!(copy.similarity(price) > 0.95);
    }

    #[test]
    fn a_short_snippet_has_no_fingerprints_and_an_unknown_language_fails() {
        assert!(
            snippet_of("fn tiny() -> u32 { 1 }")
                .expect("fingerprints")
                .is_empty()
        );
        let error = analyzer(true)
            .fingerprint(&LanguageId::new("cobol").expect("language"), "x")
            .expect_err("no pack");
        assert_eq!(
            error.code(),
            codeskimr_codes::ErrorCode::LanguagePackMissing
        );
    }

    #[test]
    fn a_snippet_needs_fingerprinting_to_be_switched_on() {
        let error = analyzer(false)
            .fingerprint(&LanguageId::new("rust").expect("language"), "fn a() {}")
            .expect_err("off");
        assert_eq!(
            error.code(),
            codeskimr_codes::ErrorCode::UnexpectedInternalError
        );
    }
}
