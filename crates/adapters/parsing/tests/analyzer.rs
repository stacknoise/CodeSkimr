//! Tests of the Tree-sitter analyzer against the fixture pack.

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::time::Duration;

    use codeskimr_application::ids::IdSequence;
    use codeskimr_application::ports::{AnalyzeRequest, DiscoveredFile, FileAnalyzer, FileFacts};
    use codeskimr_codes::{Coded, ErrorCode};
    use codeskimr_domain::{
        ContentHash, FolderPath, LanguageId, Loc, ReferenceKind, RelativePath, SnapshotId,
        SymbolKind, Visibility,
    };
    use codeskimr_parsing::{LanguagePacks, TreeSitterAnalyzer};

    fn fixtures() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
    }

    fn analyzer(timeout: Duration) -> TreeSitterAnalyzer {
        let packs = LanguagePacks::load(&fixtures().join("languages")).expect("fixture packs load");
        TreeSitterAnalyzer::new(packs, timeout)
    }

    fn analyze_in(
        root: &Path,
        path: &str,
        language: &str,
        timeout: Duration,
    ) -> Result<FileFacts, ErrorCode> {
        let file = DiscoveredFile {
            path: RelativePath::new(path).expect("path"),
            language: LanguageId::new(language).expect("language"),
            content_hash: ContentHash::new("hash").expect("hash"),
            changed: true,
        };
        let root = FolderPath::new(root.to_string_lossy()).expect("root");
        let ids = IdSequence::starting_at(1);
        let request = AnalyzeRequest {
            root: &root,
            snapshot: SnapshotId::new(1),
            file: &file,
            ids: &ids,
        };
        analyzer(timeout).analyze(&request).map_err(|e| e.code())
    }

    fn analyze(path: &str) -> Result<FileFacts, ErrorCode> {
        analyze_in(
            &fixtures().join("project"),
            path,
            "rust",
            Duration::from_secs(5),
        )
    }

    fn names(facts: &FileFacts) -> Vec<(String, SymbolKind)> {
        facts
            .symbols
            .iter()
            .map(|s| (s.qualified_name.to_string(), s.kind))
            .collect()
    }

    #[test]
    fn extracts_symbols_with_hierarchy_and_qualified_names() {
        let facts = analyze("src/sample.rs").expect("analyzes");
        let found = names(&facts);
        assert_eq!(
            found.first(),
            Some(&("src::sample".to_owned(), SymbolKind::Module))
        );
        for expected in [
            ("src::sample::Point", SymbolKind::Struct),
            ("src::sample::Shape", SymbolKind::Enum),
            ("src::sample::count", SymbolKind::Function),
            ("src::sample::helper", SymbolKind::Function),
            ("src::sample::origin", SymbolKind::Method),
        ] {
            assert!(
                found.contains(&(expected.0.to_owned(), expected.1)),
                "{expected:?} in {found:?}"
            );
        }
        let module = &facts.symbols[0];
        assert!(
            facts
                .symbols
                .iter()
                .skip(1)
                .all(|s| s.parent == Some(module.id))
        );
    }

    #[test]
    fn reads_visibility_doc_and_spans() {
        let facts = analyze("src/sample.rs").expect("analyzes");
        let point = facts
            .symbols
            .iter()
            .find(|s| s.name.as_str() == "Point")
            .expect("Point");
        assert_eq!(point.visibility, Some(Visibility::Public));
        assert_eq!(point.doc.as_deref(), Some("A point in the plane."));
        let shape = facts
            .symbols
            .iter()
            .find(|s| s.name.as_str() == "Shape")
            .expect("Shape");
        assert_eq!(shape.visibility, Some(Visibility::Private));
        let origin = facts
            .symbols
            .iter()
            .find(|s| s.name.as_str() == "origin")
            .expect("origin");
        assert_eq!(origin.visibility, Some(Visibility::Internal));
        assert_eq!(point.span.start().line, 3);
        assert_eq!(point.span.end().line, 5);
    }

    #[test]
    fn collects_pending_references_with_their_enclosing_symbol() {
        let facts = analyze("src/sample.rs").expect("analyzes");
        let count = facts
            .symbols
            .iter()
            .find(|s| s.name.as_str() == "count")
            .expect("count");
        let call = facts
            .pending
            .iter()
            .find(|p| p.target.as_str() == "helper")
            .expect("call of helper");
        assert_eq!(call.kind, ReferenceKind::Call);
        assert_eq!(call.from, count.id);
        assert_eq!(call.file, facts.file.id);
        assert!(
            facts
                .pending
                .iter()
                .any(|p| p.kind == ReferenceKind::Import)
        );
        assert!(
            facts
                .pending
                .iter()
                .any(|p| p.kind == ReferenceKind::TypeUse && p.target.as_str() == "Point")
        );
    }

    #[test]
    fn counts_only_code_lines() {
        let facts = analyze("src/sample.rs").expect("analyzes");
        // 29 lines: 5 blank, 2 line comments, 2 block comment lines.
        assert_eq!(facts.file.loc, Loc(20));
    }

    #[test]
    fn empty_file_has_only_its_module() {
        let facts = analyze("src/empty.rs").expect("analyzes");
        assert_eq!(
            names(&facts),
            vec![("src::empty".to_owned(), SymbolKind::Module)]
        );
        assert_eq!(facts.file.loc, Loc(0));
    }

    #[test]
    fn syntax_errors_do_not_fail_the_analysis() {
        let facts = analyze("src/broken.rs").expect("error tolerant");
        assert_eq!(facts.symbols.len(), 1);
    }

    #[test]
    fn unknown_language_is_pack_missing() {
        let result = analyze_in(
            &fixtures().join("project"),
            "src/sample.rs",
            "cobol",
            Duration::from_secs(5),
        );
        assert_eq!(result.err(), Some(ErrorCode::LanguagePackMissing));
    }

    #[test]
    fn missing_file_is_reported_with_a_code() {
        assert_eq!(
            analyze("src/nothing.rs").err(),
            Some(ErrorCode::SourcePathNotFound)
        );
    }

    #[test]
    fn zero_timeout_is_parse_timeout() {
        let result = analyze_in(
            &fixtures().join("project"),
            "src/sample.rs",
            "rust",
            Duration::ZERO,
        );
        assert_eq!(result.err(), Some(ErrorCode::ParseTimeout));
    }

    #[test]
    fn invalid_bytes_are_an_encoding_error_and_utf16_is_accepted() {
        let dir = std::env::temp_dir().join(format!("codeskimr-parsing-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        std::fs::write(dir.join("bad.rs"), [0xFF_u8, 0x80, 0x81]).expect("write");
        let utf16: Vec<u8> = [0xFF_u8, 0xFE]
            .into_iter()
            .chain("fn a() {}\n".encode_utf16().flat_map(u16::to_le_bytes))
            .collect();
        std::fs::write(dir.join("wide.rs"), utf16).expect("write");
        let timeout = Duration::from_secs(5);
        assert_eq!(
            analyze_in(&dir, "bad.rs", "rust", timeout).err(),
            Some(ErrorCode::FileEncodingUnsupported)
        );
        let wide = analyze_in(&dir, "wide.rs", "rust", timeout).expect("utf16");
        assert!(names(&wide).contains(&("wide::a".to_owned(), SymbolKind::Function)));
        std::fs::remove_dir_all(dir).ok();
    }

    fn scratch_file(name: &str, bytes: &[u8]) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("codeskimr-an-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        std::fs::write(dir.join("a.rs"), bytes).expect("write");
        dir
    }

    #[test]
    fn reads_utf8_bom_and_utf16_big_endian() {
        let timeout = Duration::from_secs(5);
        let mut bom = vec![0xEF_u8, 0xBB, 0xBF];
        bom.extend_from_slice(b"fn a() {}\n");
        let dir = scratch_file("bom", &bom);
        assert!(analyze_in(&dir, "a.rs", "rust", timeout).is_ok());
        let big: Vec<u8> = [0xFE_u8, 0xFF]
            .into_iter()
            .chain("fn b() {}\n".encode_utf16().flat_map(u16::to_be_bytes))
            .collect();
        let dir_be = scratch_file("be", &big);
        let facts = analyze_in(&dir_be, "a.rs", "rust", timeout).expect("utf16 be");
        assert!(names(&facts).contains(&("a::b".to_owned(), SymbolKind::Function)));
        let odd = scratch_file("odd", &[0xFF_u8, 0xFE, 0x61]);
        assert_eq!(
            analyze_in(&odd, "a.rs", "rust", timeout).err(),
            Some(ErrorCode::FileEncodingUnsupported)
        );
        for d in [dir, dir_be, odd] {
            std::fs::remove_dir_all(d).ok();
        }
    }

    #[test]
    fn plain_comments_are_not_documentation_and_declarations_are_not_duplicated() {
        let source = b"// plain\n/// first\n/// second\npub struct S;\n";
        let dir = scratch_file("doc", source);
        let facts = analyze_in(&dir, "a.rs", "rust", Duration::from_secs(5)).expect("analyzes");
        let structs: Vec<_> = facts
            .symbols
            .iter()
            .filter(|s| s.kind == SymbolKind::Struct)
            .collect();
        assert_eq!(structs.len(), 1);
        assert_eq!(structs[0].doc.as_deref(), Some("first\nsecond"));
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn references_outside_declarations_belong_to_the_module() {
        let dir = scratch_file("top", b"use std::fmt;\n");
        let facts = analyze_in(&dir, "a.rs", "rust", Duration::from_secs(5)).expect("analyzes");
        assert_eq!(facts.pending.len(), 1);
        assert_eq!(facts.pending[0].from, facts.symbols[0].id);
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn a_long_signature_is_cut_and_marked() {
        let root = std::env::temp_dir().join(format!("codeskimr-signature-{}", std::process::id()));
        std::fs::create_dir_all(root.join("src")).expect("temp dir");
        std::fs::write(
            root.join("src/long.rs"),
            "pub const LIST: &[u32] = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10];\npub const N: u32 = 3;\n",
        )
        .expect("write");
        let file = DiscoveredFile {
            path: RelativePath::new("src/long.rs").expect("path"),
            language: LanguageId::new("rust").expect("language"),
            content_hash: ContentHash::new("hash").expect("hash"),
            changed: true,
        };
        let folder = FolderPath::new(root.to_string_lossy()).expect("root");
        let ids = IdSequence::starting_at(1);
        let request = AnalyzeRequest {
            root: &folder,
            snapshot: SnapshotId::new(1),
            file: &file,
            ids: &ids,
        };
        let facts = analyzer(Duration::from_secs(5))
            .with_signature_limit(20)
            .analyze(&request)
            .expect("analyzes");
        let signature = |name: &str| {
            facts
                .symbols
                .iter()
                .find(|symbol| symbol.qualified_name.to_string().ends_with(name))
                .and_then(|symbol| symbol.signature.clone())
        };
        assert_eq!(signature("LIST").as_deref(), Some("pub const LIST: &[u…"));
        assert!(signature("LIST").is_some_and(|text| text.chars().count() <= 20));
        assert_eq!(signature("N").as_deref(), Some("pub const N: u32 = 3;"));
        std::fs::remove_dir_all(root).ok();
    }
}
